// RGX-0078.5.j.4 BATCH-1 DESIGN — G1-B carrier-layout probe.
//
// WHAT: measures the four candidate `ParseError` carrier shapes so the G1-B
// design is grounded in measured layout rather than in the estimate banked by
// `-0164` ("~80 bytes ... FIX = box the cold payload => small + trivially
// droppable").
//
// WHY: `ParseResult<T> = Result<T, ParseError>` is the return type of EVERY
// fused cascade match function (`cascade.rs:590`, `cascade.rs:725` emit
// `fn #match_fn(&mut self) -> ParseResult<()>`), so the carrier's width and
// drop-glue status are paid on the executed SUCCESS path of every rule.
//
// The field types are copied verbatim from
// `rust/src/ast_pipeline/mod.rs:654-681`, so the layout is exact.
//
// HOW:  rustc -O -o carrier_layout_probe carrier_layout_probe.rs && ./carrier_layout_probe
// OUT:  carrier_layout_probe.txt

struct Data {
    _m: String,
    _p: usize,
    _r: Vec<&'static str>,
    _i: String,
}

// (A) CURRENT — `mod.rs:654-681`, inline `ContextualError` payload.
enum A {
    E { p: usize },
    T { e: &'static str, f: char, p: usize },
    I { m: &'static str, p: usize },
    B { p: usize },
    R { p: usize, d: usize },
    C { m: String, p: usize, r: Vec<&'static str>, i: String },
}

// (B) BOXED cold payload — the shape `-0164` proposed.
enum B_ {
    E { p: usize },
    T { e: &'static str, f: char, p: usize },
    I { m: &'static str, p: usize },
    B { p: usize },
    R { p: usize, d: usize },
    C(Box<Data>),
}

// (C) INDEX cold payload — the payload moves to a parser-owned side table and
// the variant carries only its index, so the carrier owns no heap at all.
#[derive(Clone, Copy)]
enum C_ {
    E { p: usize },
    T { e: &'static str, f: char, p: usize },
    I { m: &'static str, p: usize },
    B { p: usize },
    R { p: usize, d: usize },
    C { idx: u32 },
}

// (D) INDEX + interned `&'static str` -> u32 symbol, positions narrowed to u32.
#[derive(Clone, Copy)]
enum D_ {
    E { p: u32 },
    T { e: u32, f: char, p: u32 },
    I { m: u32, p: u32 },
    B { p: u32 },
    R { p: u32, d: u32 },
    C { idx: u32 },
}

macro_rules! r {
    ($n:literal, $t:ty) => {
        println!(
            "{:<36} size={:>3}  Result<(),_>={:>3}  needs_drop={}",
            $n,
            std::mem::size_of::<$t>(),
            std::mem::size_of::<Result<(), $t>>(),
            std::mem::needs_drop::<$t>()
        );
    };
}

fn main() {
    println!("RGX-0078.5.j.4 BATCH-1 — ParseError carrier layout");
    println!("field types copied verbatim from rust/src/ast_pipeline/mod.rs:654-681\n");
    r!("A current (inline ContextualError)", A);
    r!("B boxed cold payload (-0164 proposal)", B_);
    r!("C index cold payload (side table)", C_);
    r!("D index + interned strs", D_);
    println!("\nNOTE: only (C)/(D) achieve needs_drop=false. Boxing (B) shrinks the");
    println!("carrier 80->32 but KEEPS drop glue, because Box is itself an owner.");
}
