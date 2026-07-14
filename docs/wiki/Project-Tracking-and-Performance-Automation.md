# Project tracking and performance automation

## Why state is split by kind

The port is too large and long-lived for one continuously edited handoff file.
It also produces several different kinds of information: proposed work,
accepted scientific facts, benchmark definitions, measurements, and design
explanations. Treating one Markdown file as the database makes those facts hard
to query, easy to duplicate, and prone to merge conflicts.

The repository therefore uses four authoritative layers:

1. GitHub Issues, sub-issues, and the **WRF Rust Port** Project contain mutable
   work state, dependencies, horizons, and priority.
2. Versioned JSON under `tracking/` contains scientific status and benchmark
   definitions that must change atomically with code.
3. GitHub Actions contains measured evidence. Raw logs and normalized JSON are
   artifacts; job summaries and merged-PR comments are durable receipts; Pages
   presents the latest aggregate, historical distributions, and rendered
   project documentation.
4. Rustdoc, this wiki, and dated performance reports explain algorithms,
   contracts, findings, and decisions.

Generated Markdown under `docs/generated/` is a projection of JSON. It is
reviewable in pull requests but is never edited directly.

## Issue hierarchy and Project views

A dependency-closed code slice is one issue and one pull request. A scientific
objective that requires several slices is a parent issue whose sub-issues can
land independently. The Project adds fields for status, area, work type, and
horizon so contributors can query “next ARW work,” “all upstream findings,” or
“performance work after parity” without reconstructing that state from prose.

Closing a pull request closes its slice issue. Parent progress is derived from
sub-issue completion. Project views supply the current queue and roadmap; they
do not replace the issue discussion and evidence trail.

## Post-merge benchmark flow

The `Verification` workflow proves Rust quality and pinned-Fortran parity. A
successful run on `main` triggers `Performance`:

1. The verified merge SHA is compared with its parent.
2. `tracking/benchmarks.json` maps changed paths to matched suites.
3. Each matrix job fetches the checksum-pinned WRF source, runs `-O3 -flto`
   Fortran and the release-like Criterion case on the same runner, and converts
   both outputs to one JSON schema.
4. Raw output and normalized results are uploaded as Actions artifacts.
5. A serialized aggregate job produces Markdown and cumulative JSON, calculates
   p50, p90, and p99 latency from the raw samples, adds the matrix to the Actions
   summary, and comments on the merged pull request.
6. The Pages builder renders the current matrix, per-suite historical ratio
   charts, and canonical repository Markdown into one static project site.

Documentation-only merges select no suites but still rebuild and publish the
site. Shared executor, compiler profile, catalog, or benchmark normalization
changes select every suite. Manual dispatch can force a full refresh.

Runner state can vary. Same-runner Rust/Fortran ratios are the useful
signal; small absolute changes between workflow runs are not treated as
regressions. If absolute trend detection becomes important, the same schema can
run on a pinned self-hosted benchmark machine.

## Merge safety

Required checks currently enforce a strict up-to-date branch before merge, and
contributors branch only from fresh `main` without stacking on open pull
requests. The verification workflow also accepts GitHub's `merge_group` event,
so it is ready for a native merge queue.

GitHub does not offer a native merge queue to a public repository owned by an
individual account. If the repository moves to an organization, branch rules
can require the queue immediately; until then, strict checks and unstacked
branches provide the safest available behavior.
