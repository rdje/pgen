// `.87`(d) risk measurement: at what thread-stack reservation does spawn start failing
// on this host? Standalone so it needs nothing from the pgen build.
fn try_spawn(bytes: usize) -> Result<(), std::io::Error> {
    let h = std::thread::Builder::new()
        .name("probe".to_string())
        .stack_size(bytes)
        .spawn(|| {
            // Touch a little of it so the mapping is not purely theoretical.
            let buf = [0u8; 4096];
            std::hint::black_box(&buf);
        })?;
    let _ = h.join();
    Ok(())
}

fn main() {
    const GIB: usize = 1024 * 1024 * 1024;
    let sizes: [(&str, usize); 12] = [
        ("96 MiB (floor)", 96 * 1024 * 1024),
        ("256 MiB (reference)", 256 * 1024 * 1024),
        ("384 MiB (this host, .87(d))", 384 * 1024 * 1024),
        ("1 GiB (64 GiB host)", GIB),
        ("4 GiB (256 GiB host)", 4 * GIB),
        ("16 GiB (1 TiB host)", 16 * GIB),
        ("64 GiB (4 TiB host)", 64 * GIB),
        ("384 GiB (24 TiB host)", 384 * GIB),
        ("1 TiB", 1024 * GIB),
        ("16 TiB (1 PiB host)", 16 * 1024 * GIB),
        ("64 TiB", 64 * 1024 * GIB),
        ("128 TiB", 128 * 1024 * GIB),
    ];
    for (label, bytes) in sizes {
        let t = std::time::Instant::now();
        match try_spawn(bytes) {
            Ok(()) => println!("{:>28}  {:>18} B  OK      {:?}", label, bytes, t.elapsed()),
            Err(e) => println!("{:>28}  {:>18} B  FAILED  {} (kind {:?})", label, bytes, e, e.kind()),
        }
    }
}
