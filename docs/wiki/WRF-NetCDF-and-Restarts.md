# WRF NetCDF and restarts

## Why restart parity is stricter than history parity

A history file is an observation of a trajectory. Small, explained floating-
point differences may sometimes be acceptable in diagnostics. A restart file
is executable model state: reading it changes the initial condition of the
next process. For that reason, this project requires exact schema, metadata,
primitive representation, ordering, and field bits for restart equivalence.

The first I/O slice does not claim a complete restart. It establishes the
storage contract with a dependency-closed set of ARW dynamical fields.

## NetCDF structure

WRF's ARW grid uses an Arakawa C stagger. Mass variables occupy
`west_east × south_north × bottom_top`. `U` adds the upper x face, `V` adds the
upper y face, and vertical velocity and geopotential use
`bottom_top_stag`. NetCDF dimensions describe these locations explicitly; the
`stagger` attribute records the semantic axis.

The record dimension is `Time`. `Times` stores the fixed 19-character WRF
timestamp. `XTIME` stores model minutes. The selected restart state includes
wind (`U`, `V`, `W`), geopotential (`PH`, `PHB`), thermodynamic state (`THM`,
`P`, `PB`, `QVAPOR`), and column mass (`MU`, `MUB`). `THM` is important: WRF
v4.7.1's Registry data name is `thm`, and the generator uppercases it to
`THM`; using historical `T` would be schema-incompatible.

The detailed ordered tables live in the repository's
`docs/io/minimum-netcdf-restart-schema.md`.

## Typed Rust design

Schema and data are separate. A `WrfFileSchema` owns small metadata. A
`WrfDatasetView` borrows caller fields and validates the complete dataset
before any filesystem mutation. The one-shot writer can therefore stream
caller storage without cloning domain-sized arrays or exposing lifetime-heavy
builder state.

The writer uses a safe pure-Rust NetCDF-3 implementation for WRF's
64-bit-offset mode. The reader uses the mature GeoRust wrapper around
NetCDF-C, allowing both NetCDF-3 and NetCDF-4 input while keeping all project
code under `forbid(unsafe_code)`. Trusted dependencies may encapsulate audited
unsafe internals; the port does not reproduce that machinery locally.

## Restart comparison

Comparison happens in two phases:

1. ordered dimensions, global attributes, variables, variable dimensions,
   primitive types, and variable attributes must match exactly;
2. each variable is read and compared as raw bytes in bounded multidimensional
   chunks.

The chunk planner preserves row-major coverage and reports the first different
logical element. Two one-MiB scratch buffers are reused, avoiding a second copy
of the complete restart in memory.

## Independent oracle

The C oracle uses `NC_64BIT_OFFSET`, defines the WRF-derived schema directly,
and writes deterministic values. The Rust side cannot silently approve its own
metadata because the two definitions are independent. CI then opens both and
requires both whole-file byte identity and exact typed restart equivalence.

This is storage-layer parity, not yet WRF integration parity. The decisive
future test is to stop a pinned WRF case, resume once from WRF state and once
from Rust-written state, and compare the subsequent trajectory.
