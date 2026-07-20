use std::hint::black_box;

#[repr(C)]
struct ProbeParser<'input> {
    input: &'input str,
    position: usize,
}

#[inline(never)]
#[no_mangle]
fn baseline_input_read(parser: &mut ProbeParser<'_>) -> u8 {
    let position = parser.position;
    if position < parser.input.len() {
        parser.input.as_bytes()[position]
    } else {
        0
    }
}

#[inline(never)]
#[no_mangle]
fn forwarded_len_read(parser: &mut ProbeParser<'_>, input_len: usize) -> u8 {
    let position = parser.position;
    if position < input_len {
        parser.input.as_bytes()[position]
    } else {
        0
    }
}

#[inline(never)]
#[no_mangle]
fn safe_len_orchestrator(parser: &mut ProbeParser<'_>) -> u8 {
    let input_len = parser.input.len();
    forwarded_len_read(parser, input_len)
}

#[inline(never)]
#[no_mangle]
fn forwarded_view_read(parser: &mut ProbeParser<'_>, input: &[u8]) -> u8 {
    let position = parser.position;
    if position < input.len() {
        input[position]
    } else {
        0
    }
}

#[inline(never)]
#[no_mangle]
fn safe_view_orchestrator(parser: &mut ProbeParser<'_>) -> u8 {
    // `&str` is Copy. This copies the external reference out of the parser;
    // the resulting borrow does not keep the parser field borrowed, so a
    // simultaneous `&mut ProbeParser` argument remains safe Rust.
    let input = parser.input.as_bytes();
    forwarded_view_read(parser, input)
}

fn main() {
    let mut parser = ProbeParser {
        input: "abc",
        position: 1,
    };
    let baseline = baseline_input_read(black_box(&mut parser));
    let forwarded_len = safe_len_orchestrator(black_box(&mut parser));
    let forwarded_view = safe_view_orchestrator(black_box(&mut parser));
    println!(
        "baseline={} forwarded_len={} forwarded_view={} equal={}",
        baseline,
        forwarded_len,
        forwarded_view,
        baseline == forwarded_len && baseline == forwarded_view
    );
}
