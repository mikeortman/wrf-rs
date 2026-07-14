# Acoustic mass, omega, and potential temperature

WRF's `advance_mu_t` advances three coupled parts of the small acoustic step:
perturbation column mass, vertically integrated mass flux (`ww`, often called
small-step omega), and mass-coupled perturbation potential temperature.

## Continuity and column mass

At every mass point and half level, the routine calculates horizontal flux
divergence from current U/V momentum plus saved large-step velocity weighted by
hybrid-coordinate column mass. U uses its Y map factor; V uses its inverse X
map factor. Mass-point X and Y map factors couple the combined divergence.

The divergence is integrated bottom-to-top with `dnw`. The resulting column
tendency is added to the large-step `mu_tend`, advancing perturbation column
mass. The same tendency is saved for divergence damping; base full mass is
added to form `muts`; and `epssm` produces the time-centered `muave` consumed by
the following vertical momentum solve.

## Vertical mass flux

The lower boundary value starts a bottom-to-top recurrence. Each next full
level subtracts the half-level divergence, column-integrated tendency, and
large-step mass tendency, scaled by `dnw`, `c1h`, and the mass-point Y map
factor. Saved large-step `ww_1` is then removed from every half level. The
recurrence requires the complete physical column even though the Fortran
interface accepts vertical tile bounds.

## Potential-temperature transport

The large-step theta tendency is first uncoupled with the mass-point Y map
factor and added to the mass-coupled state. Horizontal transport uses centered
neighbor sums at U and V faces. Vertical transport interpolates saved half-level
theta to full levels with `fnm` and `fnp`, multiplies by the updated acoustic
mass flux, and applies the difference through `rdnw`. Both vertical boundary
fluxes are zero.

## Safe parallel execution and memory

South-north planes are independent and run on the backend's persistent worker
pool. Every phase owns disjoint mutable blocks, so the implementation requires
no locks or unsafe code. It preserves WRF's level-major, contiguous west-east
operation order.

The Fortran routine allocates `dvdxi`, `wdtn`, and `dmdt` scratch. Rust briefly
uses required diagnostic outputs before overwriting them with their final
values: `t_ave` holds flux divergence and `muts` holds prior column mass.
Vertical theta fluxes are evaluated locally. No numerical allocation or field
clone is required.

## Boundary and parity contracts

Global tiles clip to the mass domain. Specified/nested domains exclude one
point on each lateral edge; periodic X restores the global west-east range.
The typed region requires west/south scalar neighbors, east/north momentum
neighbors, and the complete half-level column plus upper full level.

The direct oracle compiles pinned WRF v4.7.1 `advance_mu_t`. Global, nested,
periodic-X, and partial-horizontal cases match all 3,168 stored values by raw
IEEE-754 bits, including inactive sentinels. One- and four-worker results are
identical. See WRF-049 through WRF-052 in `UPSTREAM_FINDINGS.md` for dead
arguments, partial `INTENT(OUT)` definitions, the hidden complete-column
requirement, and scratch/test opportunities.
