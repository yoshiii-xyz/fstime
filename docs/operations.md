# Operations

## Local use

Run `fstime profile ./repo --cache-label cold` or use the default
`unspecified` label. Add `--ignore NAME` for each ignored name or path prefix.
Redirect JSON output to a report file, then compare it with
`fstime compare cold.json warm.json`.

## Safety and retention

The tool is read-only. It does not open regular-file contents, execute files,
follow symlink directories, or use a network service. Reports contain the
root path, counts, errors, and directory names, so apply the local retention
and access policy.

## Troubleshooting

- `complete: false` means errors or a traversal bound prevented a complete
  walk. Read the `errors` array.
- A symlink loop should appear as symlink entries and should not increase
  directory traversal because links are not followed.
- `elapsed_ms` varies with system load and should be compared only under a
  controlled run setup.
- `fstime report run.json` explains a saved report without touching its root.

## Recovery

No recovery is needed for the profiler itself because it does not write to the
profiled tree. If another process changes the tree, rerun after quiescing it.
