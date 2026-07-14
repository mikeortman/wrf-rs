/// Borrowed forcing and smoothing weights indexed by boundary distance.
#[derive(Clone, Copy)]
pub struct SpecifiedBoundaryRelaxationCoefficients<'a> {
    pub(crate) forcing: &'a [f32],
    pub(crate) smoothing: &'a [f32],
}

impl<'a> SpecifiedBoundaryRelaxationCoefficients<'a> {
    /// Captures WRF's `fcx` forcing and `gcx` smoothing arrays.
    pub const fn new(forcing: &'a [f32], smoothing: &'a [f32]) -> Self {
        Self { forcing, smoothing }
    }

    /// Returns the forcing weights in boundary-distance order.
    pub const fn forcing(self) -> &'a [f32] {
        self.forcing
    }

    /// Returns the smoothing weights in boundary-distance order.
    pub const fn smoothing(self) -> &'a [f32] {
        self.smoothing
    }
}
