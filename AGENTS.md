# Repository instructions

These instructions apply to the whole repository. More specific `AGENTS.md`
files may tighten them for a subtree.

## Mandatory Rust contract

**MUST follow [`RUST_BACKEND_STYLE_GUIDE.md`](RUST_BACKEND_STYLE_GUIDE.md) for
all Rust design, implementation, review, testing, and documentation work.** Read
it before changing Rust code. Treat its rules as the repository contract, not
optional guidance. A more specific `AGENTS.md` may add stricter requirements
but may not weaken the style guide unless the user explicitly directs an
exception.

## Sources of truth

- GitHub Issues, sub-issues, and the **WRF Rust Port** Project own mutable work
  state, priorities, dependencies, and long-horizon plans.
- `tracking/port-status.json` owns the current coarse parity state.
- `tracking/benchmarks.json` owns matched benchmark definitions and change
  routing.
- `docs/generated/` is generated. Run `python3 tools/tracking.py render`; never
  hand-edit generated files.
- GitHub Actions benchmark artifacts, job summaries, and merged-PR receipt
  comments own measurements from CI. Detailed `docs/performance/` pages remain
  immutable engineering records, not the current-value database.
- `CURRENT_STATE.md`, `TEST_COVERAGE.md`, and `PERFORMANCE_PARITY.md` are legacy
  narrative references. Do not add mutable totals, TODO lists, or benchmark
  tables to them.

Start work by querying GitHub, not by reconstructing a backlog from Markdown:

```sh
gh issue list --repo mikeortman/wrf-rs --state open --limit 100
gh project item-list 1 --owner mikeortman
```

## Change workflow

1. Every dependency-closed slice starts with one issue. Use parent issues and
   sub-issues for objectives that span several independently mergeable slices.
   Add active issues to the **WRF Rust Port** Project and set Area, Work type,
   Horizon, and Evidence gate. Repository issue forms add the Project
   automatically; CLI-created issues should use `--project "WRF Rust Port"`.
2. Fetch `main`, branch from the fetched tip as `codex/issue-N-short-name`, and
   do not stack a new branch on an unmerged pull request.
3. Keep one issue-linked pull request per slice. Its body must close the issue.
4. Enable auto-merge only after required checks are present. Until this
   user-owned repository can use GitHub's native merge queue, the strict
   up-to-date branch rule and unstacked branches are the queue substitute.
5. Commit focused checkpoints. Preserve unrelated user changes.

## Scientific acceptance gates

- Pin the exact WRF source routine or upstream regression test before porting.
- Preserve discrete behavior. Use exact bits by default; any ULP or tolerance
  policy must be explicit, justified, and tested at intermediate states.
- Add a direct Fortran oracle, deterministic fixture, malformed-input tests,
  boundary cases, and seeded randomized coverage where useful.
- Test failure atomicity, one-worker behavior, and multithreaded determinism.
- A slice is not complete because Rust-only tests pass.

## Rust design and performance

- Use safe, idiomatic Rust; repository code must not contain `unsafe`.
- Prefer clear ownership, typed units/errors, borrowed field views, reusable
  scratch, and bounded allocations. Lightweight clones are acceptable when
  they materially improve clarity without cloning large fields.
- Organize code by scientific subsystem. Large subsystems own nested modules;
  keep stable public facades and avoid flat collections of unrelated files.
- CPU execution is the default and should use host parallelism sensibly.
  Future accelerators belong behind narrow capability traits.
- Benchmark only after parity. Compare the same workload with release-like
  Rust and `-O3 -flto` Fortran, without fast-math unless separately justified.
- Treat close results as close. Do not keep complex SIMD, target-specific
  tuning, or less readable code for a small or noisy gain.
- SIMD requires scalar/Fortran parity and representative benchmark evidence.
  Prefer maintained, well-trusted crates over local intrinsics.

## Documentation and verification

- Public APIs need useful rustdoc with equations, units, indexing, invariants,
  errors, and upstream provenance where relevant.
- Update the wiki for algorithms or architecture; update
  `UPSTREAM_FINDINGS.md` for reproducible upstream bugs or performance issues.
- Update structured tracking records instead of hand-maintained totals.
- Before requesting merge, run the applicable subset and then the full gates:

```sh
python3 tools/tracking.py check
cargo fmt --all --check
cargo test --workspace --all-targets
cargo test --workspace --release
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Run every affected Fortran oracle. Review `git diff --check` and the complete
diff before committing.
