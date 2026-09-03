# Design

## Measurements

The profiler counts traversal, `symlink_metadata`, `read_dir`, and
`read_link` calls that it invokes. `open_count` and `realpath_count` are
explicitly zero in this implementation because it does not open file content
or call canonicalization. Directory metrics record direct entry counts and
the sum of regular-file sizes below each visited directory.

## Input and output

`profile_tree` accepts a local path, ignore names or prefixes, and an operator
cache label. It emits a versioned JSON report. `compare_profiles` produces
numeric deltas from two saved reports. `explain_profile` renders a report
without touching the filesystem.

## Symlinks and failures

Traversal uses symlink metadata and never follows a symlink as a directory.
The stored link target is read to count and observe readlink behavior. Missing
roots, permission failures, read errors, depth limits, and entry limits make a
report incomplete and preserve bounded error text.

## Resource limits

Traversal is capped at 100,000 entries and depth 256. Directory metrics and
errors are capped. JSON output is capped at 1 MiB. The report's elapsed time is
a volatile wall-clock measurement, not a monotonic kernel trace.

## Portability boundary

The implementation uses portable Rust filesystem APIs and is tested on Linux.
It does not claim to measure every kernel operation, cache event, scheduler
decision, mount behavior, or filesystem-specific metadata.
