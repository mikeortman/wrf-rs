//! C-grid domain axes, physical ranges, and active tile derivation.

mod axis;
mod region;

pub use axis::AcousticHorizontalMomentumAxis;
pub(crate) use region::AcousticHorizontalMomentumActiveRanges;
pub use region::AcousticHorizontalMomentumRegion;
