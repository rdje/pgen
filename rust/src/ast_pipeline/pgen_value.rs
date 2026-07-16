//! `PgenValue<'input>` — the arena-`Copy` committed-value representation
//! (RGX-0078.5.i.7 REPRESENTATION, `PGEN-RGX-0078-0104`).
//!
//! RE-PROFILE #13 (`PGEN-RGX-0078-0102`) pinned the committed-value serde
//! machinery (`serde_json::Value` construction, `BTreeMap` inserts, `String`
//! key clones, subtree deep-clones, `Value` drops) as the dominant attackable
//! compute surface of the parse. This type replaces `serde_json::Value` INSIDE
//! the parse: every composite is an arena slice and every reference is `Copy`,
//! so putting a child value under a key copies machine words instead of
//! deep-cloning a serde tree, and teardown is the arena's single mass free.
//!
//! ## Byte-identity contract (the load-bearing invariant)
//!
//! The released typed-AST JSON carrier must not change by a single byte. Two
//! mechanisms guarantee that:
//!
//! 1. **The `Serialize` impl mirrors `serde_json::Value`'s** variant for
//!    variant (`Null`→`serialize_unit`, numbers→`serialize_i64/u64/f64`,
//!    strings→`serialize_str`, arrays→sequences, objects→maps). All actual
//!    formatting (string escaping, itoa/ryu number rendering) is performed by
//!    the SAME `serde_json` serializer that renders `Value` today, so equal
//!    logical values produce equal bytes by construction.
//! 2. **Object semantics mirror `BTreeMap`** — entries are key-SORTED and
//!    duplicate keys resolve last-write-wins (see [`insert_object_pair`]),
//!    exactly `serde_json::Map`'s insert + iteration behavior.
//!
//! The unit tests below are the oracle: each case serializes a `PgenValue`
//! and the equivalent `serde_json::Value` and asserts byte equality.
//!
//! ## Number semantics
//!
//! `Int`/`UInt`/`Float` mirror `serde_json::Number`'s internal split
//! (`NegInt`/`PosInt`/`Float`). Equality is variant-strict like serde's
//! (`Float(1.0) != Int(1)`). Conversions that mirror `Value::from(f64)` map
//! non-finite floats to `Null` ([`PgenValue::from_f64`]); the `Serialize`
//! impl defensively renders a non-finite `Float` as `null` as well, so both
//! paths agree.

use serde::ser::{Serialize, SerializeMap, Serializer};

/// The arena-`Copy` committed value. Composite variants borrow arena slices
/// (see `NodeArena`); string payloads borrow the input or an arena-interned
/// rendered string. The single `'input` lifetime is unified to the arena scope
/// exactly like `ParseNode`'s (the `.5.d.4.i` precedent).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PgenValue<'input> {
    Null,
    Bool(bool),
    /// Mirrors `serde_json::Number` `NegInt`/small `PosInt` (constructed from
    /// `i64` literals and integer transforms).
    Int(i64),
    /// Mirrors `serde_json::Number` `PosInt` beyond `i64::MAX` (reachable via
    /// the `TransformedTerminal` JSON-text conversion path). CANONICALIZATION
    /// RULE: a value representable as `i64` is ALWAYS constructed as `Int`,
    /// never `UInt` (serde's `Number` collapses the overlap range into one
    /// internal variant, so without this rule `PartialEq` across the two
    /// spellings would diverge from serde's). Construction sites must uphold
    /// it; [`PgenValue::from_u64`] encodes it.
    UInt(u64),
    /// Finite by discipline: construction sites mirror `Value::from(f64)` by
    /// mapping non-finite to `Null` (use [`PgenValue::from_f64`]).
    Float(f64),
    Str(&'input str),
    Array(&'input [PgenValue<'input>]),
    /// Key-sorted, duplicate-free (last-write-wins) — the `BTreeMap` mirror.
    /// Build through [`insert_object_pair`] to preserve that invariant.
    Object(&'input [(&'input str, PgenValue<'input>)]),
}

impl<'input> PgenValue<'input> {
    /// Mirror of `serde_json::Value::from(f64)`: non-finite maps to `Null`.
    pub fn from_f64(value: f64) -> Self {
        if value.is_finite() {
            PgenValue::Float(value)
        } else {
            PgenValue::Null
        }
    }

    /// Canonicalizing `u64` constructor: the `i64`-representable range becomes
    /// `Int` (the `UInt` doc's canonicalization rule), so numeric `PartialEq`
    /// never depends on which integer spelling a construction site used.
    pub fn from_u64(value: u64) -> Self {
        match i64::try_from(value) {
            Ok(as_i64) => PgenValue::Int(as_i64),
            Err(_) => PgenValue::UInt(value),
        }
    }

    /// Convert to an owned `serde_json::Value` (the boundary escape hatch —
    /// dump paths, fact rendering, and any consumer that needs an owned tree).
    /// Produces exactly the `Value` today's eager construction would have
    /// built, so serializing either representation yields identical bytes.
    pub fn to_serde_value(&self) -> serde_json::Value {
        match *self {
            PgenValue::Null => serde_json::Value::Null,
            PgenValue::Bool(value) => serde_json::Value::Bool(value),
            PgenValue::Int(value) => serde_json::Value::from(value),
            PgenValue::UInt(value) => serde_json::Value::from(value),
            PgenValue::Float(value) => serde_json::Value::from(value),
            PgenValue::Str(text) => serde_json::Value::String(text.to_string()),
            PgenValue::Array(items) => {
                serde_json::Value::Array(items.iter().map(|item| item.to_serde_value()).collect())
            }
            PgenValue::Object(pairs) => {
                let mut map = serde_json::Map::new();
                for (key, value) in pairs {
                    map.insert((*key).to_string(), value.to_serde_value());
                }
                serde_json::Value::Object(map)
            }
        }
    }
}

/// Insert one `(key, value)` pair into an under-construction object pair list,
/// mirroring `serde_json::Map` (`BTreeMap`) `insert` semantics byte-exactly:
/// the list stays SORTED by key bytes, and inserting an existing key replaces
/// its value in place (last write wins). Objects in return-annotation
/// templates are small (a handful of keys), so binary-search + `Vec::insert`
/// beats any map structure and produces the final slice layout directly.
pub fn insert_object_pair<'input>(
    pairs: &mut Vec<(&'input str, PgenValue<'input>)>,
    key: &'input str,
    value: PgenValue<'input>,
) {
    match pairs.binary_search_by(|(existing, _)| existing.as_bytes().cmp(key.as_bytes())) {
        Ok(index) => pairs[index].1 = value,
        Err(index) => pairs.insert(index, (key, value)),
    }
}

impl Serialize for PgenValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            // `serde_json::Value::Null` serializes through `serialize_unit`.
            PgenValue::Null => serializer.serialize_unit(),
            PgenValue::Bool(value) => serializer.serialize_bool(value),
            PgenValue::Int(value) => serializer.serialize_i64(value),
            PgenValue::UInt(value) => serializer.serialize_u64(value),
            PgenValue::Float(value) => {
                if value.is_finite() {
                    serializer.serialize_f64(value)
                } else {
                    // Defensive parity: a `Value` can never hold a non-finite
                    // number (`from(f64)` yields `Null`), so render `null`.
                    serializer.serialize_unit()
                }
            }
            PgenValue::Str(text) => serializer.serialize_str(text),
            PgenValue::Array(items) => serializer.collect_seq(items.iter()),
            PgenValue::Object(pairs) => {
                let mut map = serializer.serialize_map(Some(pairs.len()))?;
                for (key, value) in pairs {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The byte-identity oracle: serialize the `PgenValue` and the equivalent
    /// `serde_json::Value` (compact AND pretty) and assert equal bytes, plus
    /// assert `to_serde_value` reproduces the oracle `Value` structurally.
    fn assert_byte_identical(pgen: &PgenValue<'_>, oracle: &serde_json::Value) {
        assert_eq!(
            serde_json::to_string(pgen).unwrap(),
            serde_json::to_string(oracle).unwrap(),
            "compact serialization diverged"
        );
        assert_eq!(
            serde_json::to_string_pretty(pgen).unwrap(),
            serde_json::to_string_pretty(oracle).unwrap(),
            "pretty serialization diverged"
        );
        assert_eq!(
            &pgen.to_serde_value(),
            oracle,
            "to_serde_value diverged from the oracle Value"
        );
    }

    #[test]
    fn scalars_are_byte_identical() {
        assert_byte_identical(&PgenValue::Null, &serde_json::Value::Null);
        assert_byte_identical(&PgenValue::Bool(true), &serde_json::Value::Bool(true));
        assert_byte_identical(&PgenValue::Bool(false), &serde_json::Value::Bool(false));
        assert_byte_identical(&PgenValue::Int(0), &serde_json::Value::from(0i64));
        assert_byte_identical(&PgenValue::Int(i64::MIN), &serde_json::Value::from(i64::MIN));
        assert_byte_identical(&PgenValue::Int(i64::MAX), &serde_json::Value::from(i64::MAX));
        assert_byte_identical(&PgenValue::UInt(u64::MAX), &serde_json::Value::from(u64::MAX));
        // The u64/i64 overlap range must render identically from either variant.
        assert_eq!(
            serde_json::to_string(&PgenValue::UInt(3)).unwrap(),
            serde_json::to_string(&PgenValue::Int(3)).unwrap()
        );
    }

    #[test]
    fn floats_are_byte_identical_including_whole_and_signed_zero() {
        for f in [
            0.0f64,
            -0.0,
            1.0,
            -1.0,
            0.5,
            -2.75,
            1e300,
            5e-324,
            std::f64::consts::PI,
        ] {
            assert_byte_identical(&PgenValue::Float(f), &serde_json::Value::from(f));
        }
        // The emitter-documented case: `from(0.0_f64)` serializes as `0.0`,
        // never `0` — the Float variant must preserve that.
        assert_eq!(serde_json::to_string(&PgenValue::Float(0.0)).unwrap(), "0.0");
    }

    #[test]
    fn non_finite_floats_mirror_value_from_f64_as_null() {
        for f in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(PgenValue::from_f64(f), PgenValue::Null);
            assert_eq!(serde_json::Value::from(f), serde_json::Value::Null);
            // Defensive serialize parity for a directly-held non-finite Float.
            assert_eq!(serde_json::to_string(&PgenValue::Float(f)).unwrap(), "null");
        }
        assert_eq!(PgenValue::from_f64(2.5), PgenValue::Float(2.5));
    }

    #[test]
    fn strings_escape_byte_identically() {
        for s in [
            "",
            "plain",
            "with \"quotes\" and \\backslashes\\",
            "control:\u{0}\u{1}\u{1f}\ttab\nnewline\rcr",
            "unicode: éüλ🎯 \u{2028}\u{2029}",
            "solidus / stays bare",
        ] {
            assert_byte_identical(
                &PgenValue::Str(s),
                &serde_json::Value::String(s.to_string()),
            );
        }
    }

    #[test]
    fn arrays_and_nesting_are_byte_identical() {
        let inner = [PgenValue::Int(1), PgenValue::Str("two"), PgenValue::Null];
        let outer = [
            PgenValue::Array(&inner),
            PgenValue::Bool(true),
            PgenValue::Array(&[]),
        ];
        let oracle = serde_json::json!([[1, "two", null], true, []]);
        assert_byte_identical(&PgenValue::Array(&outer), &oracle);
    }

    #[test]
    fn objects_sort_keys_exactly_like_btreemap() {
        // Insert deliberately out of order; BTreeMap sorts on iteration.
        let mut pairs = Vec::new();
        insert_object_pair(&mut pairs, "zeta", PgenValue::Int(1));
        insert_object_pair(&mut pairs, "alpha", PgenValue::Str("a"));
        insert_object_pair(&mut pairs, "Middle", PgenValue::Bool(false));
        insert_object_pair(&mut pairs, "middle", PgenValue::Bool(true));
        let pgen = PgenValue::Object(&pairs);

        let mut map = serde_json::Map::new();
        map.insert("zeta".to_string(), serde_json::Value::from(1i64));
        map.insert("alpha".to_string(), serde_json::Value::String("a".into()));
        map.insert("Middle".to_string(), serde_json::Value::Bool(false));
        map.insert("middle".to_string(), serde_json::Value::Bool(true));
        assert_byte_identical(&pgen, &serde_json::Value::Object(map));
    }

    #[test]
    fn duplicate_keys_resolve_last_write_wins_like_btreemap_insert() {
        let mut pairs = Vec::new();
        insert_object_pair(&mut pairs, "k", PgenValue::Int(1));
        insert_object_pair(&mut pairs, "other", PgenValue::Null);
        insert_object_pair(&mut pairs, "k", PgenValue::Str("replaced"));
        assert_eq!(pairs.len(), 2);
        let pgen = PgenValue::Object(&pairs);

        let mut map = serde_json::Map::new();
        map.insert("k".to_string(), serde_json::Value::from(1i64));
        map.insert("other".to_string(), serde_json::Value::Null);
        map.insert("k".to_string(), serde_json::Value::String("replaced".into()));
        assert_byte_identical(&pgen, &serde_json::Value::Object(map));
    }

    #[test]
    fn deep_mixed_tree_is_byte_identical() {
        let inner_items = [PgenValue::Float(2.5), PgenValue::UInt(9223372036854775808)];
        let mut inner_pairs = Vec::new();
        insert_object_pair(&mut inner_pairs, "list", PgenValue::Array(&inner_items));
        insert_object_pair(&mut inner_pairs, "empty", PgenValue::Object(&[]));
        let mut outer_pairs = Vec::new();
        insert_object_pair(&mut outer_pairs, "type", PgenValue::Str("regex"));
        insert_object_pair(&mut outer_pairs, "inner", PgenValue::Object(&inner_pairs));
        let pgen = PgenValue::Object(&outer_pairs);

        let oracle = serde_json::json!({
            "type": "regex",
            "inner": { "list": [2.5, 9223372036854775808u64], "empty": {} }
        });
        assert_byte_identical(&pgen, &oracle);
    }

    #[test]
    fn equality_is_variant_strict_and_from_u64_canonicalizes() {
        // Mirrors serde: `Number::from(1i64) != Number::from(1.0f64)`.
        assert_ne!(PgenValue::Int(1), PgenValue::Float(1.0));
        assert_eq!(PgenValue::Float(1.0), PgenValue::Float(1.0));
        // The canonicalization rule: i64-representable u64s become `Int`, so
        // the overlap range has ONE spelling (serde collapses it internally).
        assert_eq!(PgenValue::from_u64(3), PgenValue::Int(3));
        assert_eq!(
            PgenValue::from_u64(u64::MAX),
            PgenValue::UInt(u64::MAX)
        );
        assert_eq!(
            PgenValue::from_u64(i64::MAX as u64),
            PgenValue::Int(i64::MAX)
        );
        assert_eq!(
            PgenValue::from_u64(i64::MAX as u64 + 1),
            PgenValue::UInt(i64::MAX as u64 + 1)
        );
    }
}
