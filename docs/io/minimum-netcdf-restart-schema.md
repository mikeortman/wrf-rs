# Registry-driven WRF NetCDF restart schema

This dependency-closed I/O slice targets WRF v4.7.1's classic 64-bit-offset
NetCDF path. `WrfRegistryRestartSchemaBuilder` consumes a borrowed
`wrf_registry::RegistryDocument` and produces a typed restart schema from the
selected `dimspec` and `state` metadata. The original fixed minimum-ARW
constructor remains available as a compatibility fixture, but Registry state is
no longer limited to its 13 variables.

This is not yet a complete `wrfrst` implementation. Alarm metadata, boundary
companion arrays, subgrid dimensions, package and four-dimensional scalar
bundling, processor-transposed state, NetCDF-4 policy, and resumed-trajectory
parity remain separate gates.

## Pinned upstream sources

The implementation is derived from WRF v4.7.1 commit
`f52c197ed39d12e087d02c50f412d90d418f6186`, as pinned in `UPSTREAM.toml`:

- `tools/reg_parse.c` parses the Registry I/O string and marks restart state;
- `tools/gen_allocs.c` creates external data names, time-level suffixes,
  dimension names, descriptions, units, memory order, and stagger metadata;
- `tools/type.c` implements `set_mem_order`;
- `tools/set_dim_strs.c` resolves standard-domain, namelist, and constant
  inclusive bounds with standard-domain staggering;
- `share/output_wrf.F` selects every restart time level and dispatches the
  correct WRF field type;
- `external/io_netcdf/wrf_io.F90` implements `ExtOrder`, dimension first use,
  classic file creation, `FieldType`, and three-byte `MemoryOrder`; and
- `external/ioapi_share/wrf_io_flags.h` pins `WRF_REAL = 104`,
  `WRF_DOUBLE = 105`, `WRF_INTEGER = 106`, and `WRF_LOGICAL = 107`.

WRF's legacy `testWRFWrite.F90` does not exercise the current low-level
signature. The differential fixture therefore calls NetCDF-C directly, the
same storage API used by `wrf_io.F90`, and defines its expected schema manually
instead of asking the Rust builder to generate its oracle.

The same gate also extracts the pinned `GetDim`, `ExtOrder`, `ExtOrderStr`,
`LowerCase`, and `reorder` routines directly from `wrf_io.F90`, compiles them
with a focused Fortran driver, and compares their XZY, YX, and scalar ordering
summary against the Rust-built schema. The executable Fortran differential and
the independent NetCDF-C file differential cover separate parts of the path.

## Registry transformation

Restart membership comes from a lowercase or uppercase `r` in the Registry I/O
string. Braced stream numbers and `f=`, `d=`, `u=`, or `s=` interpolation
payloads are opaque, matching `reg_parse.c`; unmatched braces fail with a typed
error. Unselected state does not constrain the restart schema.

Selected states support:

- standard-domain X, Y, and Z dimensions, with mass or staggered length;
- one- or two-name namelist bounds, including literal integer bounds;
- named constant bounds and anonymous constant-axis bounds;
- scalar, vertical/constant one-dimensional, horizontal two-dimensional, and
  all six X/Y/Z three-dimensional Registry orders; and
- one external variable per time level, using uppercase `DataName`, then
  `_1`, `_2`, and so on when `ntl` is greater than one.

WRF's generated fixed-character metadata is normalized at the runtime call
boundary: an empty or leading-blank `DataName` falls back to the state name,
and trailing ASCII blanks are trimmed from external names, descriptions, and
units.

WRF reorders Registry dimensions into external X/Y/Z order before calling the
Fortran NetCDF API. The Fortran API reverses those IDs in the physical file, so
the typed C-order schema is `Time, Z, Y, X`; X remains the contiguous dimension.
Horizontal fields are `Time, Y, X`. Scalars still carry the unlimited `Time`
record dimension.

Dimension definitions follow WRF first use. `Time` and `DateStrLen` are created
first. Named dimensions reuse an existing name only at the same length.
Constant-coordinate axes are anonymous because `gen_allocs.c` only emits
dimension names for X, Y, and Z. Anonymous axes reuse the first dimension of
equal length or claim the next `DIMnnnn` slot. The builder also models WRF's
2,000 pre-seeded placeholder slots, including order-sensitive `DIMnnnn` name
collisions and exhaustion.

Each selected field carries exactly this ordered attribute set:

1. `FieldType`
2. `MemoryOrder`
3. `description`
4. `units`
5. `stagger`

`MemoryOrder` is uppercase and blank-padded to exactly three bytes. Real,
double-precision, integer, and logical Registry state maps to `NC_FLOAT`,
`NC_DOUBLE`, `NC_INT`, and `NC_INT`, respectively; logical state retains
`FieldType = 107`. WRF encodes logically empty text attributes as one NUL byte;
the typed Rust schema normalizes that representation back to an empty string.

## Differential fixture

`scripts/run-netcdf-restart-oracle.sh` first runs the extracted pinned-Fortran
ordering differential, then covers these representative fields with the
independent NetCDF-C fixture:

| Variables | Coverage |
|---|---|
| `T`, `U`, `V`, `W` | mass and X/Y/Z-staggered standard-domain 3D fields |
| `LANDMASK` | YX Registry order reordered to external XY; integer storage |
| `ENERGY` | double-precision 3D storage |
| `ACTIVE` | logical metadata with exact `NC_INT` data |
| `SOIL`, `SOILSTAG` | namelist Z dimension, both names and staggering |
| `MODE`, `MODESTAG` | constant Z dimension, both names and staggering |
| `CATEGORY`, `ANON` | constant-axis dimensions with anonymous first-use slots |
| `XTIME` | zero-dimensional Registry scalar written on `Time` |
| `TEND_1`, `TEND_2` | multi-time-level data names |

The fixture includes signed zero, explicit NaN payloads, and infinity. The
script requires byte-identical complete files and then uses
`WrfRestartComparer` to compare ordered dimensions, variables, attributes, and
raw field bits.

## Validation, allocation, and failure behavior

The Registry builder and `WrfFileSchema` validate generated names, dimension
consistency, supported memory orders and field types, and duplicate variables.
`WrfDatasetView` borrows caller buffers and validates complete membership,
primitive type, and element count. `WrfNetcdfWriter` builds the complete NetCDF
definition before opening or truncating the output path, so malformed schema
leaves an existing file unchanged.

The writer uses a fixed one-MiB buffered output and never clones a field.
`WrfRestartComparer` reads bounded chunks with at most one MiB per comparison
side, independent of field size. Schema building is deterministic across
repeated and concurrent callers; it does not mutate the parsed Registry.

Malformed and degenerate coverage includes missing or inverted namelist bounds,
conflicting dimension lengths, reserved names, invalid NetCDF names, unsupported
axis combinations, selected character state, subgrid dimensions,
boundary arrays, three-dimensional logical state, processor-transposed state,
four-dimensional scalar members, malformed I/O specifications, and an empty
restart selection.
