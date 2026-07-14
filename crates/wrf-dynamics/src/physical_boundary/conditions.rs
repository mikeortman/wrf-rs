/// Lateral boundary-condition flags mirrored from WRF `grid_config_rec_type`.
///
/// Only the combinations WRF itself can configure are meaningful: a periodic
/// axis suppresses that axis's symmetric and open branches exactly as in
/// `set_physical_bc3d`/`set_physical_bc2d`. The open-boundary copy also fires
/// when `specified` or `nested` is set, and `polar` joins the south-north open
/// copy. All flags default to `false`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalBoundaryConditions {
    /// WRF `periodic_x`: wrap the west-east halo from the opposite edge.
    pub periodic_x: bool,
    /// WRF `symmetric_xs`: reflect about the west domain edge.
    pub symmetric_xs: bool,
    /// WRF `symmetric_xe`: reflect about the east domain edge.
    pub symmetric_xe: bool,
    /// WRF `open_xs`: copy the west edge value outward.
    pub open_xs: bool,
    /// WRF `open_xe`: copy the east edge value outward.
    pub open_xe: bool,
    /// WRF `periodic_y`: wrap the south-north halo from the opposite edge.
    pub periodic_y: bool,
    /// WRF `symmetric_ys`: reflect about the south domain edge.
    pub symmetric_ys: bool,
    /// WRF `symmetric_ye`: reflect about the north domain edge.
    pub symmetric_ye: bool,
    /// WRF `open_ys`: copy the south edge value outward.
    pub open_ys: bool,
    /// WRF `open_ye`: copy the north edge value outward.
    pub open_ye: bool,
    /// WRF `polar`: polar caps join the south-north open copies.
    pub polar: bool,
    /// WRF `specified`: specified lateral boundaries join every open copy.
    pub specified: bool,
    /// WRF `nested`: nest-forced boundaries join every open copy.
    pub nested: bool,
}

impl PhysicalBoundaryConditions {
    /// Doubly periodic configuration used by idealized channel cases.
    pub const fn periodic_xy() -> Self {
        Self {
            periodic_x: true,
            periodic_y: true,
            symmetric_xs: false,
            symmetric_xe: false,
            open_xs: false,
            open_xe: false,
            symmetric_ys: false,
            symmetric_ye: false,
            open_ys: false,
            open_ye: false,
            polar: false,
            specified: false,
            nested: false,
        }
    }

    /// Specified lateral boundaries as configured by real-data cases.
    pub const fn specified_lateral() -> Self {
        Self {
            specified: true,
            periodic_x: false,
            periodic_y: false,
            symmetric_xs: false,
            symmetric_xe: false,
            open_xs: false,
            open_xe: false,
            symmetric_ys: false,
            symmetric_ye: false,
            open_ys: false,
            open_ye: false,
            polar: false,
            nested: false,
        }
    }

    /// Nest-forced lateral boundaries of a child domain.
    pub const fn nested_lateral() -> Self {
        Self {
            nested: true,
            periodic_x: false,
            periodic_y: false,
            symmetric_xs: false,
            symmetric_xe: false,
            open_xs: false,
            open_xe: false,
            symmetric_ys: false,
            symmetric_ye: false,
            open_ys: false,
            open_ye: false,
            polar: false,
            specified: false,
        }
    }

    /// True when the west open-boundary copy is active.
    pub(crate) const fn copies_open_west(self) -> bool {
        self.open_xs || self.specified || self.nested
    }

    /// True when the east open-boundary copy is active.
    pub(crate) const fn copies_open_east(self) -> bool {
        self.open_xe || self.specified || self.nested
    }

    /// True when the south open-boundary copy is active.
    pub(crate) const fn copies_open_south(self) -> bool {
        self.open_ys || self.polar || self.specified || self.nested
    }

    /// True when the north open-boundary copy is active.
    pub(crate) const fn copies_open_north(self) -> bool {
        self.open_ye || self.polar || self.specified || self.nested
    }
}
