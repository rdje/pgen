use std::hint::black_box;

#[repr(C)]
struct ProbeParser {
    position: usize,
    furthest_position: usize,
}

#[inline(never)]
#[no_mangle]
fn baseline_update(parser: &mut ProbeParser) -> usize {
    if parser.position > parser.furthest_position {
        parser.furthest_position = parser.position;
    }
    parser.furthest_position
}

#[inline(never)]
#[no_mangle]
fn forwarded_ref_update(position: usize, furthest_position: &mut usize) -> usize {
    if position > *furthest_position {
        *furthest_position = position;
    }
    *furthest_position
}

#[inline(never)]
#[no_mangle]
fn value_update(position: usize, furthest_position: usize) -> usize {
    position.max(furthest_position)
}

fn main() {
    let mut baseline = ProbeParser {
        position: 7,
        furthest_position: 3,
    };
    let mut forwarded = 3;
    let baseline_result = baseline_update(black_box(&mut baseline));
    let forwarded_result = forwarded_ref_update(black_box(7), black_box(&mut forwarded));
    let value_result = value_update(black_box(7), black_box(3));
    println!(
        "baseline={} forwarded_ref={} value={} equal={}",
        baseline_result,
        forwarded_result,
        value_result,
        baseline_result == forwarded_result && forwarded_result == value_result,
    );
}
