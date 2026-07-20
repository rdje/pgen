# Raw/timeline sampler qualification — `spindump` is root-only on this host

`PGEN-RGX-0078-0180`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Short instrument qualification only; no full-band profile, build,
parser change, emitter change, or generated-artifact change.

## Result

The local `spindump` candidate cannot be used by this campaign under the
available authority. Its help surface advertises exactly the retention features
worth testing—target-only microsecond intervals, timeline order, leaf-frame
timestamps, binary output, and later re-rendering—but a live capture exits
before sampling with:

```text
spindump must be run as root when sampling the live system
```

That result reproduced twice: once in the normal sandbox and once through the
approved unsandboxed execution path. Both returned exit 77. This is a tool
privilege requirement, not a filesystem-sandbox artifact.

The unchanged preserved probe was under full custody before attachment:

- full SHA-256
  `1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8`;
- live PID 94599;
- `lsof` text image resolved to the exact repository-relative preserved-probe
  path;
- 38,640 KiB RSS at the identity check;
- no competing build or profiler job, 64 GiB disk free, and no throttled memory
  pages.

No capture file was created, so the pre-registered **>=95% one-PC coverage**
bar is not met. The workload was interrupted after refusal, both sampler and
probe were confirmed absent, and the empty temporary directory was removed.

**Decision: REFUSE `spindump` on this host.** The campaign will not request or
assume root authority merely to rescue a candidate instrument. BATCH-1 + G3
remains **84.385 ns / HOLD**; no performance number changes.

## Next instrument

`PGEN-RGX-0078-0181` will qualify a small in-process profiler injected into the
unchanged probe. The design boundary is:

1. A preload dynamic library installs an `ITIMER_PROF`/`SIGPROF` handler.
2. On arm64, the handler reads the interrupted PC from `ucontext_t` into a
   preallocated fixed buffer; the same signal is blocked during its handler, so
   a simple index is sufficient and no allocation or I/O occurs in signal
   context.
3. Normal process teardown disables the timer and emits raw PCs plus the main
   image ASLR slide; offline code normalizes every address against the same
   full-SHA probe and `otool` disassembly.
4. Qualification again requires **>=95%** one-PC retention and exact re-summing
   of the target self population. It must also quantify handler/out-of-image
   samples rather than silently discard them.

This approach requires no debugger attach, root access, parser mutation, or
generated-artifact change. The injected profiler source/build/run and its
signal-safety audit belong to the separately owned next leaf.
