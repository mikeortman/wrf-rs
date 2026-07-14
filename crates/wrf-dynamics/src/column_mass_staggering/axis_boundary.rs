#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColumnMassStaggeringAxisBoundary {
    Interior,
    Lower,
    Upper,
    Both,
}

impl ColumnMassStaggeringAxisBoundary {
    pub(crate) const fn from_contacts(lower: bool, upper: bool) -> Self {
        match (lower, upper) {
            (false, false) => Self::Interior,
            (true, false) => Self::Lower,
            (false, true) => Self::Upper,
            (true, true) => Self::Both,
        }
    }

    pub(crate) const fn touches_lower(self) -> bool {
        matches!(self, Self::Lower | Self::Both)
    }

    pub(crate) const fn touches_upper(self) -> bool {
        matches!(self, Self::Upper | Self::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_every_boundary_state_from_independent_contacts() {
        let cases = [
            ((false, false), ColumnMassStaggeringAxisBoundary::Interior),
            ((true, false), ColumnMassStaggeringAxisBoundary::Lower),
            ((false, true), ColumnMassStaggeringAxisBoundary::Upper),
            ((true, true), ColumnMassStaggeringAxisBoundary::Both),
        ];

        for ((lower, upper), expected) in cases {
            let boundary = ColumnMassStaggeringAxisBoundary::from_contacts(lower, upper);
            assert_eq!(boundary, expected);
            assert_eq!(boundary.touches_lower(), lower);
            assert_eq!(boundary.touches_upper(), upper);
        }
    }
}
