# Queryable project tracking

The project deliberately separates policy, work state, scientific facts, and
measurements so an agent does not need a long conversation or a hand-maintained
state dump to recover context.

| Information | Authoritative home |
|---|---|
| Work, priority, dependencies, horizon | GitHub Issues, sub-issues, and the **WRF Rust Port** Project |
| Coarse port/parity state | `port-status.json` |
| Matched benchmark commands and change routing | `benchmarks.json` |
| Current generated views | `docs/generated/` |
| CI performance measurements | Actions artifacts, run summaries, Pages, and merged-PR receipt comments |
| Algorithm explanations and decisions | Rustdoc, `docs/wiki/`, and dated `docs/performance/` records |
| Upstream defects and opportunities | `UPSTREAM_FINDINGS.md` |

- Project: <https://github.com/users/mikeortman/projects/1>
- Project documentation and performance history: <https://mikeortman.github.io/wrf-rs/>

## Updating state

Edit the relevant JSON record, then run:

```sh
python3 tools/tracking.py render
python3 tools/tracking.py check
```

`check` validates paths and schemas, ensures every Criterion bench and matched
Fortran benchmark is catalogued, and fails when generated views drift.

Do not put TODO lists, changing totals, or current benchmark values back into a
narrative Markdown ledger. Create or update an issue instead. Generated
Markdown is a convenient projection, not another database.

## Benchmark lifecycle

After the required parity workflow succeeds on `main`, the performance
workflow selects suites affected by the merged diff. Each matrix job runs the
catalogued Fortran and Rust commands on the same runner, normalizes their raw
outputs into JSON, and uploads both evidence and logs. A serialized aggregate
job calculates p50, p90, and p99 latency, appends each observation to the
published per-suite history, posts the current matrix to the merged pull
request, and publishes the project site. Documentation-only merges rebuild the
site without running numerical suites. Manual dispatch can refresh every suite.

Same-runner measurements are suitable for relative Rust/Fortran comparisons
and coarse trends. Runner state can still vary, so small cross-run deltas are
not regression verdicts. The runner classification remains attached to every
observation in case the benchmark host changes.

The Pages artifact also renders the repository Markdown under `docs/`, the
root project records, and this tracking guide into a shared, searchable
documentation shell. Markdown remains canonical; generated HTML is disposable.
