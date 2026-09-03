# Research notes

Research date: 2026-09-03.

Primary sources:

- Rust [`read_dir`](https://doc.rust-lang.org/std/fs/fn.read_dir.html)
  documents directory iteration and entry errors.
- Rust [`symlink_metadata`](https://doc.rust-lang.org/std/fs/fn.symlink_metadata.html)
  documents metadata inspection without following the final symlink.
- Rust [`read_link`](https://doc.rust-lang.org/std/fs/fn.read_link.html)
  documents reading the stored symlink target.
- Rust [`Metadata`](https://doc.rust-lang.org/std/fs/struct.Metadata.html)
  documents portable file metadata access.

Issue and discussion review:
[Rust filesystem issues](https://github.com/rust-lang/rust/issues?q=is%3Aissue+filesystem+metadata)
and [Rust users filesystem discussions](https://users.rust-lang.org/search?q=filesystem%20profiling)
were treated as context only, not as normative behavior.

Distribution signal: the Cargo package and standalone CLI are prepared for
crates.io. Package availability or download counts are distribution signals,
not evidence of willingness to pay.

Evidence grade: the cited documentation supports the filesystem operations
and symlink boundary. The operation counters, limits, cold or warm labels, and
report structure are design inferences for a bounded userspace MVP.

Rejected alternatives include a kernel tracer, which would exceed the stated
scope and portability evidence, and a GUI, which would not improve the
machine-readable measurement boundary. Decision: count invoked userspace
operations and label unobserved conditions explicitly.
