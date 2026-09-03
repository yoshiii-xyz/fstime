# Limits and non-goals

The 0.1.0 MVP has these limits:

- Linux is the only platform covered by release evidence.
- The profiler reads local directory metadata and symlink targets. It does
  not open regular-file contents, execute programs, or modify files.
- `open_count` and `realpath_count` are zero because those calls are not part
  of the MVP. A zero count does not prove the operating system made no such
  internal calls.
- The cold or warm label is user supplied. The profiler does not flush or
  inspect filesystem caches.
- Traversal is capped at 100,000 entries and depth 256. JSON reports are
  capped at 1 MiB.
- Permission errors and concurrent filesystem changes can make a report
  incomplete. The tool does not provide a race-free snapshot.
- The tool is not a GUI, kernel profiler, general monitoring platform, or
  build-system performance oracle.
