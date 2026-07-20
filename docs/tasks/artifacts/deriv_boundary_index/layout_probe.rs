use std::mem::{align_of, needs_drop, size_of};

struct ParseNode<'a> {
    _input: std::marker::PhantomData<&'a str>,
}

fn main() {
    type BoundaryRef = &'static ParseNode<'static>;
    println!(
        "BoundaryRef size={} align={} needs_drop={}",
        size_of::<BoundaryRef>(),
        align_of::<BoundaryRef>(),
        needs_drop::<BoundaryRef>()
    );
    println!(
        "BoundaryIndex32 size={} align={} needs_drop={}",
        size_of::<u32>(),
        align_of::<u32>(),
        needs_drop::<u32>()
    );
    println!(
        "BoundaryIndexUsize size={} align={} needs_drop={}",
        size_of::<usize>(),
        align_of::<usize>(),
        needs_drop::<usize>()
    );
    println!("usize_bits={}", usize::BITS);
}
