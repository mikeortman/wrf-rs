# Domain decomposition and halo exchange

## Purpose

WRF divides a physical grid twice. MPI processes own **patches**; threads work
on smaller **tiles** inside each patch. A patch also allocates **memory bounds**
outside its owned points so neighboring values can be received into halos.
These are different contracts and must not share an untyped integer tuple.

The implementation is split accordingly:

- `wrf-domain` owns bounds, decomposition, transfer plans, one-patch storage,
  and the deterministic in-process reference exchange;
- `wrf-domain-mpi` owns MPI calls only; and
- numerical crates consume domain meaning without depending on MPI types.

Both crates forbid local unsafe code.

## Index representation

WRF passes inclusive Fortran ranges such as `ids:ide`, usually with one-based
origins. Rust converts these once into signed, zero-based, half-open ranges:

```text
Fortran  1:10   -> Rust  0..10
Fortran -2:4    -> Rust -3..4
```

Half-open ranges make lengths and adjacency unambiguous. Signed indices are
necessary because physical and periodic halos can be outside the physical
domain. `IndexRange` rejects empty ranges and checked construction reports
overflow instead of wrapping.

## RSL_LITE patch decomposition

For one axis with `N` points and `P` processes, RSL_LITE starts with
`base = N / P` and `remainder = N % P`. Remainder points do not all go to the
first processes. They are split between both ends because boundary processes
usually have less work.

For 13 points on five process columns, the lengths are:

```text
3 | 2 | 2 | 3 | 3
```

`DomainTopology` calculates these ranges directly and assigns row-major patch
IDs, matching MPI Cartesian rank order when reordering is disabled. The direct
oracle compiles WRF's pinned `external/RSL_LITE/task_for_point.c` and compares
all patch assignments for three uneven grids.

WRF's routine silently limits some invalid process counts. Rust rejects a
process dimension larger than its grid dimension before any field exists. This
turns a later partial or empty patch failure into a typed setup error.

## Patch, memory, and tile bounds

Each `PatchBounds` contains:

- the physical points owned by one rank;
- the process-grid coordinate and stable `PatchId`; and
- derived `MemoryBounds` including halos and WRF's extra guard point.

The memory formula follows `compute_memory_dims_rsl_lite`. On one axis it is:

```text
memory start = max(patch start - halo, domain start - boundary width) - 1
memory end   = min(patch end   + halo, domain end   + boundary width) + 1
```

Here `end` is already half-open. Boundary width reserves storage outside the
physical domain for periodic fields.

`create_tiles` splits owned patch axes with centered remainder placement.
Requested execution may extend into allocated halo memory. Only edge tiles
inherit that extension, and final execution is clipped to the physical domain,
matching the intent of WRF `set_tiles2`. Invalid execution outside memory is
rejected before tiles are returned.

## Halo plans

`HaloExchangePlan` contains rectangular `HaloTransfer` descriptors. A transfer
names its source and destination patches, source and destination ranges,
vertical range, axis, and direction. It does not contain an MPI communicator,
CPU closure, or storage pointer, so another transport or GPU backend can
consume the same scientific plan.

WRF exchanges south-north first and west-east second:

```text
pack Y -> exchange Y -> unpack Y -> pack X -> exchange X -> unpack X
```

The second phase includes the updated south-north halo in its transverse
range. This is how diagonal corner values propagate without separate corner
messages. The local executor packs every message in a phase before mutation,
then unpacks in deterministic order. It copies boundary-sized buffers, never a
complete field.

## Periodic endpoints and staggering

WRF periodic exchange is not a simple modulo operation. `period.c` treats the
last physical endpoint as the periodic duplicate. For a width `w`:

- the high-edge patch sends the `w` points immediately before its last
  endpoint to the low halo;
- the low-edge patch sends its first `w` points to the high endpoint and halo;
  and
- staggering adds one point to the low-to-high message only.

Periodic transfers do not clip the transverse axis because doubly periodic
corners must travel through the Y-then-X sequence. The plan validates that
physical-boundary storage is wide enough for `w + stagger` before a field can
be changed.

The periodic oracle compiles the pinned WRF `period.c`, `buf_for_proc.c`, and
`f_pack.F90`, runs four MPI ranks, and compares every selected physical
periodic destination and doubly periodic corner with Rust. The fixture covers
width two, two vertical levels, X staggering, and both periodic axes.

## Local and MPI execution

`PatchField<T>` owns one rank's contiguous XZY buffer. `LocalPatchField<T>` is
an all-patch test container used only by the deterministic reference executor.
The production MPI adapter receives a single `PatchField<T>`, so each process
allocates only its own patch plus boundary message buffers.

The MPI adapter:

1. verifies communicator size and rank-to-patch ownership;
2. packs outbound boundary regions;
3. allocates exact-sized inbound buffers;
4. posts all non-blocking receives before non-blocking sends;
5. waits for the phase; and
6. unpacks before starting the next axis.

Message tags distinguish axis and direction. A four-rank fixture compares the
complete memory of every MPI patch with the local executor for nonperiodic and
doubly periodic staggered plans.

## Performance and memory policy

Topology construction is setup work. It favors explicit validation and small
immutable descriptors. Halo execution is linear in the transferred boundary
area, and temporary memory is linear in in-flight boundary messages. No
field-sized clone is used.

A loopback MPI timing is not yet recorded as a Fortran/Rust speed ratio because
WRF's generated multi-field aggregation is not ported; such a number would
mostly measure the local MPI runtime. The benchmark gate becomes meaningful
when both implementations exchange the same aggregated field set.

## Evidence and remaining scope

Implemented evidence:

- exact pinned `task_for_point.c` patch assignments;
- WRF guard-point memory formula tests;
- invalid topology and storage rejection before mutation;
- clipped edge-tile tests;
- internal edge and corner propagation;
- exact pinned `period.c` periodic and staggered destinations; and
- complete local-versus-four-rank MPI memory parity.

Still required are a direct `set_tiles2` oracle, Registry-generated asymmetric
halo descriptors, multi-field aggregation, larger process grids, nested and
intermediate domains, and end-to-end ARW fields.
