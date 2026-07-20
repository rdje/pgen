#![allow(dead_code)]

use std::marker::PhantomData;
use std::mem::{align_of, needs_drop, size_of};
use std::ops::Range;

#[derive(Clone, Copy)]
enum DerivEvent {
    OrWinner(usize),
    QuantCount(usize),
    OptPresent(bool),
    TokStart(usize),
    TokEnd(usize),
}

struct ParseNode<'input> {
    rule_name: &'static str,
    content: &'input str,
    span: Range<usize>,
}

#[derive(Clone, Copy)]
enum TaggedCurrent<'input> {
    Event(DerivEvent),
    Boundary(&'input ParseNode<'input>),
}

#[derive(Clone, Copy)]
enum TaggedPacked<'input> {
    Event(usize),
    Boundary(&'input ParseNode<'input>),
}

union UntaggedWord<'input> {
    event: usize,
    boundary: &'input ParseNode<'input>,
}

#[derive(Clone, Copy)]
struct TaggedAddressWord<'input> {
    raw: *const ParseNode<'input>,
    lifetime: PhantomData<&'input ParseNode<'input>>,
}

const TAG_MASK: usize = 0b111;
const TAG_ESCAPE: usize = 0b111;
const NARROW_PAYLOAD_MAX: usize = usize::MAX >> 3;

impl<'input> TaggedAddressWord<'input> {
    fn from_address(address: usize) -> Self {
        Self {
            raw: std::ptr::without_provenance(address),
            lifetime: PhantomData,
        }
    }

    fn from_boundary(node: &'input ParseNode<'input>) -> Self {
        assert_eq!(node as *const ParseNode<'input> as usize & TAG_MASK, 0);
        Self {
            raw: node,
            lifetime: PhantomData,
        }
    }

    fn tag(self) -> usize {
        self.raw.addr() & TAG_MASK
    }

    fn boundary(self) -> Option<&'input ParseNode<'input>> {
        if self.tag() != 0 {
            return None;
        }
        // The sole constructor of a tag-zero word stores an unmodified live
        // reference. Event constructors always set a nonzero tag, including
        // the wide payload word, which is consumed only after an escape tag.
        unsafe { self.raw.as_ref() }
    }
}

fn encode_event<'input>(tag: usize, payload: usize) -> ([TaggedAddressWord<'input>; 2], usize) {
    assert!((1..TAG_ESCAPE).contains(&tag));
    if payload <= NARROW_PAYLOAD_MAX {
        (
            [
                TaggedAddressWord::from_address((payload << 3) | tag),
                TaggedAddressWord::from_address(1),
            ],
            1,
        )
    } else {
        (
            [
                TaggedAddressWord::from_address((tag << 3) | TAG_ESCAPE),
                TaggedAddressWord::from_address(payload),
            ],
            2,
        )
    }
}

fn decode_event(words: &[TaggedAddressWord<'_>]) -> (usize, usize, usize) {
    let address = words[0].raw.addr();
    let tag = address & TAG_MASK;
    if tag == TAG_ESCAPE {
        (address >> 3, words[1].raw.addr(), 2)
    } else {
        (tag, address >> 3, 1)
    }
}

fn print_layout<T>(name: &str) {
    println!(
        "{name} size={} align={} needs_drop={}",
        size_of::<T>(),
        align_of::<T>(),
        needs_drop::<T>(),
    );
}

fn main() {
    let node = ParseNode {
        rule_name: "probe",
        content: "x",
        span: 0..1,
    };
    let boundary = TaggedAddressWord::from_boundary(&node);
    assert!(std::ptr::eq(boundary.boundary().unwrap(), &node));
    for tag in 1..TAG_ESCAPE {
        for payload in [0, NARROW_PAYLOAD_MAX, NARROW_PAYLOAD_MAX + 1, usize::MAX] {
            let (encoded, width) = encode_event(tag, payload);
            assert_eq!(decode_event(&encoded), (tag, payload, width));
            assert!(encoded[0].boundary().is_none());
        }
    }
    print_layout::<DerivEvent>("DerivEvent");
    print_layout::<ParseNode>("ParseNodeStandIn");
    print_layout::<&ParseNode>("BoundaryRef");
    print_layout::<TaggedCurrent>("TaggedCurrent");
    print_layout::<TaggedPacked>("TaggedPacked");
    print_layout::<UntaggedWord>("UntaggedWord");
    print_layout::<TaggedAddressWord>("TaggedAddressWord");
    print_layout::<[TaggedAddressWord; 2]>("WideEventRecord");
    println!("usize_bits={}", usize::BITS);
    println!("narrow_payload_max={NARROW_PAYLOAD_MAX}");
    println!("codec_roundtrip=PASS");
}
