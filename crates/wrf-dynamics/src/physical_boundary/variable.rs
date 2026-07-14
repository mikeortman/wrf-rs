/// Field class passed to WRF `set_physical_bc3d`/`set_physical_bc2d`.
///
/// The variants cover the variable characters the acoustic small-step window
/// passes: `'u'`, `'v'`, `'w'`, and the interchangeable mass-point codes
/// `'t'`/`'p'`. The class fixes the horizontal stagger and, for volume fields,
/// the vertical extent of every boundary copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalBoundaryVariable {
    /// West-east staggered momentum (`'u'`).
    WestEastFace,
    /// South-north staggered momentum (`'v'`).
    SouthNorthFace,
    /// Vertically staggered full-level field (`'w'`).
    FullLevel,
    /// Unstaggered mass-point field (`'t'` or `'p'`).
    MassHalfLevel,
}

impl PhysicalBoundaryVariable {
    /// Fortran `istag`: zero when the west-east axis stores one extra point.
    pub(crate) const fn west_east_stagger(self) -> isize {
        match self {
            Self::WestEastFace => 0,
            _ => -1,
        }
    }

    /// Fortran `jstag`: zero when the south-north axis stores one extra point.
    pub(crate) const fn south_north_stagger(self) -> isize {
        match self {
            Self::SouthNorthFace => 0,
            _ => -1,
        }
    }

    /// True for the vertically staggered class whose copies reach `kde`.
    pub(crate) const fn is_full_level(self) -> bool {
        matches!(self, Self::FullLevel)
    }

    /// True for the west-east staggered class with sign-flipped symmetry.
    pub(crate) const fn is_west_east_face(self) -> bool {
        matches!(self, Self::WestEastFace)
    }

    /// True for the south-north staggered class with sign-flipped symmetry.
    pub(crate) const fn is_south_north_face(self) -> bool {
        matches!(self, Self::SouthNorthFace)
    }
}
