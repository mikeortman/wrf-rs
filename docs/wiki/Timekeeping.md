# Timekeeping

## Why model time is separate

Atmospheric integration uses a deterministic model timeline, not a wall clock.
Time zones, daylight-saving transitions, and operating-system clock changes are
irrelevant. The WRF time compatibility layer uses a proleptic Gregorian
calendar and exact rational seconds.

## Representation

`ModelTime` combines a civil date-time with a rational fractional second.
`TimeInterval` represents a signed rational duration. Fractions are normalized
to a positive denominator and reduced by their greatest common divisor. Exact
rational arithmetic prevents cumulative drift from repeatedly adding a
fractional timestep in binary floating point.

The calendar supports year zero and negative years because the mathematical
proleptic calendar is broader than many civil-date libraries. Conversion to and
from a civil-day count makes addition and subtraction uniform across month,
year, century, and leap-year boundaries.

## Compatibility details

Formatting follows the bundled WRF time manager, including its signed
fractional interval convention. Negative fractions are emitted as `-NN/DD`;
assuming an otherwise plausible plus sign produced an early false expectation
that was corrected against the upstream golden output.

The current compatibility evidence covers all 93 active cases in WRF's
`Test1.F90` and byte-for-byte output from both its `ESMF_` and `WRFU_` build
interfaces. Leap seconds are not modeled because the pinned compatibility
surface does not require them; that policy must be revisited if a later caller
does.
