# Product brief

fstime shows which observable filesystem traversal choices make a local tree
slow or incomplete without pretending to be a kernel profiler.

Target users are maintainers of large source trees and local build tools who
need counts for traversal, metadata, directories, symlinks, ignores, and
permission failures.

The first commands are:

```text
fstime profile ./repo
fstime compare cold.json warm.json
fstime report run.json
```

Current alternatives include ad hoc timing around recursive scripts, general
system profilers, and synchronization tools. Those options can obscure which
tree paths were visited or ignored and can imply observations outside a
userspace walk. The switching wedge is a bounded report that explains the
operations this program actually invoked.

Evidence and inference are separate. Filesystem API behavior is grounded in
the primary sources linked in [`docs/research.md`](research.md). The
description of alternative weaknesses and the switching wedge is a product
inference, not a market-size or adoption claim.

Non-goals are GUI work, kernel instrumentation, cache flushing, general
monitoring, and build-system integration.
