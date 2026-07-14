/// Vertical geopotential-advection discretization selected by WRF `phi_adv_z`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcousticVerticalAdvection {
    /// Staggers the geopotential gradient before multiplying by omega (`phi_adv_z = 2`).
    StaggeredGeopotentialGradient,
    /// Staggers the omega-gradient product (`phi_adv_z /= 2`).
    StaggeredTransportProduct,
}
