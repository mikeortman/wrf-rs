# Minimum WRF NetCDF and restart schema

This is the first dependency-closed WRF I/O slice. It targets WRF v4.7.1's
classic NetCDF path and enough ARW state to prove typed schema, stagger,
metadata, field-bit, and restart-equivalence behavior. It is not yet a complete
`wrfinput` or `wrfrst` implementation.

## Upstream sources

The inventory is derived from the pinned files below:

- `share/output_wrf.F` defines global dates, grid dimensions, restart markers,
  and alarm metadata;
- `external/io_netcdf/wrf_io.F90` defines classic 64-bit-offset creation and
  variable storage;
- `Registry/Registry.EM_COMMON` defines ARW data names, descriptions, units,
  memory order, and staggering; and
- `tools/gen_allocs.c` uppercases Registry data names before runtime I/O.

WRF's legacy `testWRFWrite.F90` does not exercise the current low-level
signature. The independent parity fixture therefore calls NetCDF-C directly,
which is the same storage API used by `wrf_io.F90`, using the inventoried WRF
schema rather than asking the Rust implementation to generate its own oracle.

## Dimensions

| File order | Name | Length | Role |
|---:|---|---:|---|
| 1 | `Time` | unlimited, one record in the fixture | restart time record |
| 2 | `DateStrLen` | 19 | `YYYY-MM-DD_HH:MM:SS` |
| 3 | `west_east` | 4 | mass-grid x axis |
| 4 | `south_north` | 3 | mass-grid y axis |
| 5 | `bottom_top` | 2 | mass-grid vertical axis |
| 6 | `west_east_stag` | 5 | x-staggered axis |
| 7 | `south_north_stag` | 4 | y-staggered axis |
| 8 | `bottom_top_stag` | 3 | vertically staggered axis |

Runtime constructors accept other positive grid lengths and derive each
staggered length with checked arithmetic.

## Variables

All numerical fields in this slice are `NC_FLOAT`. Dimensions are listed in
NetCDF file order.

| Name | Dimensions | Memory order | Stagger | Meaning |
|---|---|---|---|---|
| `Times` | `Time, DateStrLen` | — | — | timestamp characters |
| `U` | `Time, bottom_top, south_north, west_east_stag` | `XYZ` | `X` | x wind |
| `V` | `Time, bottom_top, south_north_stag, west_east` | `XYZ` | `Y` | y wind |
| `W` | `Time, bottom_top_stag, south_north, west_east` | `XYZ` | `Z` | vertical wind |
| `PH`, `PHB` | `Time, bottom_top_stag, south_north, west_east` | `XYZ` | `Z` | perturbation/base geopotential |
| `THM` | `Time, bottom_top, south_north, west_east` | `XYZ` | empty | moist or dry perturbation potential temperature |
| `MU`, `MUB` | `Time, south_north, west_east` | `XY ` | empty | perturbation/base column dry-air mass |
| `P`, `PB`, `QVAPOR` | `Time, bottom_top, south_north, west_east` | `XYZ` | empty | pressure and water-vapor state |
| `XTIME` | `Time` | `0  ` | empty | model minutes |

Each numerical variable carries WRF's `FieldType = 104`, `MemoryOrder`,
`description`, `units`, and `stagger` attributes. Text, including trailing
spaces and empty attributes, is compared exactly. Float attributes use raw-bit
equality, preserving signed zero and NaN payload distinctions.

## Global metadata

The typed schema includes `TITLE`, `START_DATE`, `SIMULATION_START_DATE`, the
three `*-GRID_DIMENSION` attributes, `DX`, `DY`, and `GRIDTYPE`. Restart files
also require `FLAG_RESTART = 1`. WRF's alarm attributes and the much larger
Registry-selected restart state are explicitly deferred.

## Rust boundary

`WrfFileSchema` validates metadata and dimensions. `WrfDatasetView` borrows all
field buffers and verifies variable membership, type, and length before the
output path is opened. `WrfNetcdfWriter` emits classic 64-bit-offset files with
no field-sized clone. `WrfNetcdfReader` accepts NetCDF-3 and NetCDF-4 through
the safe GeoRust API and reads into caller-owned buffers.

NetCDF-C is not thread-safe, so the reader dependency serializes C calls. This
does not change compute-kernel parallelism. Concurrent callers should perform
large, bounded reads rather than many tiny calls.

`WrfRestartComparer` requires restart markers, exact ordered schema and
metadata, and exact raw variable bits. It compares chunks with at most one MiB
per side, so scratch memory is bounded independently of field size.

## Evidence and limitations

`scripts/run-netcdf-restart-oracle.sh` builds the C fixture with the installed
NetCDF library, writes an independent restart, writes the Rust restart in
release mode, and compares every typed schema element and field bit. Unit tests
also cover malformed dimensions and timestamps, missing variables, wrong
primitive/length buffers, float-metadata bit equality, and first differing
restart element reporting.

The committed oracle also requires the complete classic-format files to be
byte-identical. The semantic comparison remains because it gives a useful
variable and element location when future differences appear.

The writer currently targets only classic 64-bit-offset output. The reader's
typed inventory intentionally rejects dimensions and primitive types outside
this minimum slice. Full WRF restart support still requires arbitrary Registry
dimensions, all selected fields and attributes, multiple time records,
NetCDF-4 chunking/compression policy, alarm state, and an end-to-end resumed
trajectory comparison.
