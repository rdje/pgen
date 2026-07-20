use pgen::generated_parsers::regex::RegexParser;
use std::mem::{align_of, size_of};

#[inline(never)]
fn type_anchor(parser: *const RegexParser<'static>) {
    std::hint::black_box(parser);
}

fn main() {
    type_anchor(std::ptr::null());
    println!(
        "RegexParser<'static> size={} align={}",
        size_of::<RegexParser<'static>>(),
        align_of::<RegexParser<'static>>(),
    );
}
