# Rust module structure

The physical source tree follows scientific ownership rather than accumulating
one file per type at each crate root. A crate root is a stable public facade;
algorithm families own their implementation details beneath named directories.

For example, `wrf-dynamics` currently has this shape:

```text
src/
├── lib.rs
├── held_suarez/
│   ├── mod.rs
│   ├── cpu.rs
│   ├── kernels.rs
│   ├── fields.rs
│   ├── region.rs
│   └── ...
└── positive_definite/
    ├── mod.rs
    ├── cpu.rs
    ├── kernels.rs
    ├── slab_region.rs
    └── ...
```

## Placement rules

1. A scientifically coherent algorithm family owns one directory.
2. The family `mod.rs` is a small internal facade: it declares children and
   re-exports only the types needed by the crate facade.
3. `lib.rs` documents the crate, declares top-level families, and preserves the
   public API. It does not list every implementation file.
4. Backend implementations live within the family they implement. A CPU file
   may be split further by operation when implementation code—not tests—becomes
   difficult to navigate.
5. Domain concepts retain focused files with one primary type where practical:
   errors, validated regions, field bundles, and capability traits do not need
   to share a large catch-all file.
6. Tests remain at the bottom of the implementation file they exercise, in
   accordance with `RUST_BACKEND_STYLE_GUIDE.md`.

This hierarchy balances discoverability with the style guide's focused-file
rule. It also scales toward future nested families such as dynamics, physics,
I/O, and registry generation without exposing internal file layout as public
API.

`wrf-registry` applies the same rule to build-time infrastructure:

```text
src/
├── lib.rs
├── model/
│   ├── mod.rs
│   ├── dimension.rs
│   ├── state_variable.rs
│   └── ...
├── parser/
│   ├── mod.rs
│   ├── logical_line.rs
│   ├── tokenizer.rs
│   └── registry_parser.rs
└── generated_state/
    ├── mod.rs
    ├── registry_artifact_generator.rs
    └── ...
```

The parser owns syntax and typed metadata. `generated_state` consumes that
model but does not become runtime domain storage. Future runtime domain types
must live in a runtime crate and consume generated declarations through an
explicit boundary.
