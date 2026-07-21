use std::mem::size_of;
use std::ops::Range;
// today's shapes (replica)
enum PgenValue<'a> { Null, Bool(bool), Int(i64), UInt(u64), Float(f64), Str(&'a str), Array(&'a [PgenValue<'a>]), Object(&'a [(&'a str, PgenValue<'a>)]) }
enum ContentOld<'a> { Terminal(&'a str), TransformedTerminal(String), Shaped(PgenValue<'a>), Sequence(Vec<&'a NodeOld<'a>>), Alternative(&'a NodeOld<'a>), Quantified(Vec<&'a NodeOld<'a>>, &'static str) }
struct NodeOld<'a> { rule_name: &'static str, content: ContentOld<'a>, span: Range<usize> }
// candidate shapes
#[derive(Clone, Copy)] struct Span { start: u32, end: u32 }
enum ContentNew<'a> { Terminal(&'a str), TransformedTerminal(String), Shaped(PgenValue<'a>), Sequence(Vec<&'a NodeNew<'a>>), Alternative(&'a NodeNew<'a>), Quantified(Vec<&'a NodeNew<'a>>, &'static &'static str) }
struct NodeNew<'a> { rule_name: &'static &'static str, content: ContentNew<'a>, span: Span }
enum ParseError { A { position: usize }, B { expected: &'static str, found: char, position: usize }, C { message: &'static str, position: usize }, D { position: usize } }
fn main() {
    println!("PgenValue={}", size_of::<PgenValue>());
    println!("ContentOld={} NodeOld={} Result<NodeOld,PE>={}", size_of::<ContentOld>(), size_of::<NodeOld>(), size_of::<Result<NodeOld, ParseError>>());
    println!("ContentNew={} NodeNew={} Result<NodeNew,PE>={}", size_of::<ContentNew>(), size_of::<NodeNew>(), size_of::<Result<NodeNew, ParseError>>());
    // static-promotion proof for the thin-ref emission spelling:
    let r: &'static &'static str = &"letter";
    println!("promoted={} deref_eq={}", r, *r == "letter");
}
