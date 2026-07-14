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
- Latest performance dashboard: <https://mikeortman.github.io/wrf-rs/>

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
job builds the matrix, posts it to the merged pull request, and publishes the
latest dashboard. Manual dispatch can refresh every suite.

GitHub-hosted hardware is suitable for relative Rust/Fortran comparisons made
on the same runner and for coarse trends. It is not a stable laboratory for
small cross-run deltas. A future pinned self-hosted runner may be marked as the
authoritative absolute baseline without changing the result schema.
