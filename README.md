# fstime

fstime profiles observable local filesystem traversal work so large-tree
performance reports do not imply kernel-level measurements.

Status: released v0.1.0.

CI: https://github.com/joshiii-xyz/fstime/actions

## Install

```text
cargo install fstime
```

## Quick start

```text
fstime profile ./repo --cache-label cold
fstime compare cold.json warm.json
fstime report run.json
```

## What it solves

Maintainers can see traversal, metadata, directory, symlink, ignored-path,
and permission measurements from one bounded local walk.

## How it works

fstime sorts directory entries, counts the filesystem operations it invokes,
records direct and subtree directory metrics, and never follows symlinks.
`elapsed_ms` is a volatile wall-clock measurement. The cold or warm label is
operator supplied and does not manipulate filesystem cache state.

## Commands and library API

The CLI provides `profile`, `compare`, and `report`. The library exposes
`profile_tree`, `compare_profiles`, `profile_json`, and explanation helpers.
Use `fstime --help` for the complete option list.

## Output and exit codes

- Profile exit code 0 means the walk completed without recorded errors.
- Profile exit code 1 means the report is incomplete or hit a bound.
- Exit code 2 means a saved report could not be read or encoded.

JSON output is capped at 1 MiB. Traversal is capped at 100,000 entries and
depth 256. The report states ignored paths, failures, and the operations that
were not invoked.

## Safety and data handling

The profiler reads metadata and symlink targets only. It does not open file
contents, execute programs, modify trees, flush caches, or upload data.
