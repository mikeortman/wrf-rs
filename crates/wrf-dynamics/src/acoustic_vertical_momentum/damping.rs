/// Optional upper-level damping applied after the implicit solve.
#[derive(Clone, Copy, Debug)]
pub enum AcousticVerticalDamping {
    /// Disables WRF `damp_opt = 3` for this acoustic step.
    Disabled,
    /// Applies the sinusoidal upper damping layer.
    UpperLayer {
        /// WRF damping coefficient before multiplication by the acoustic timestep.
        coefficient: f32,
        /// Geometric depth of the damping layer in meters.
        depth: f32,
    },
}
