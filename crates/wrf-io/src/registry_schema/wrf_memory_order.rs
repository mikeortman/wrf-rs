use wrf_registry::CoordinateAxis;

use crate::{WrfIoError, WrfIoResult};

/// A Registry memory order with a supported WRF external reordering.
///
/// WRF's `ExtOrder`/`reorder` in `external/io_netcdf/wrf_io.F90` accept only
/// scalars, one vertical or constant dimension, the two horizontal orders, and
/// the six three-dimensional permutations of X, Y, and Z. The external file
/// order always sorts axes as X fastest, then Y, then Z, and the stored
/// `MemoryOrder` attribute is that sorted order blank-padded to three bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WrfMemoryOrder {
    /// No spatial dimensions; WRF writes memory order `0`.
    Scalar,
    /// One vertical dimension; WRF writes memory order `Z`.
    Vertical,
    /// One constant-axis dimension; WRF writes memory order `C`.
    Constant,
    /// Two horizontal dimensions in Registry order.
    Horizontal {
        /// Position of the X dimension in Registry memory order.
        x_position: usize,
        /// Position of the Y dimension in Registry memory order.
        y_position: usize,
    },
    /// Three spatial dimensions in Registry order.
    Volume {
        /// Position of the X dimension in Registry memory order.
        x_position: usize,
        /// Position of the Y dimension in Registry memory order.
        y_position: usize,
        /// Position of the Z dimension in Registry memory order.
        z_position: usize,
    },
}

impl WrfMemoryOrder {
    /// Classifies Registry dimension axes, rejecting unsupported orders.
    pub(crate) fn try_new(state: &str, axes: &[CoordinateAxis]) -> WrfIoResult<Self> {
        let position = |axis: CoordinateAxis| {
            axes.iter()
                .position(|&candidate| candidate == axis)
                .filter(|_| axes.iter().filter(|&&candidate| candidate == axis).count() == 1)
        };

        match axes.len() {
            0 => Ok(Self::Scalar),
            1 if axes[0] == CoordinateAxis::Z => Ok(Self::Vertical),
            1 if axes[0] == CoordinateAxis::Constant => Ok(Self::Constant),
            2 => match (position(CoordinateAxis::X), position(CoordinateAxis::Y)) {
                (Some(x_position), Some(y_position)) => Ok(Self::Horizontal {
                    x_position,
                    y_position,
                }),
                _ => Err(Self::unsupported(state, axes)),
            },
            3 => match (
                position(CoordinateAxis::X),
                position(CoordinateAxis::Y),
                position(CoordinateAxis::Z),
            ) {
                (Some(x_position), Some(y_position), Some(z_position)) => Ok(Self::Volume {
                    x_position,
                    y_position,
                    z_position,
                }),
                _ => Err(Self::unsupported(state, axes)),
            },
            _ => Err(Self::unsupported(state, axes)),
        }
    }

    /// Returns Registry-order positions sorted into external X, Y, Z order.
    pub(crate) fn external_permutation(&self) -> Vec<usize> {
        match *self {
            Self::Scalar => Vec::new(),
            Self::Vertical | Self::Constant => vec![0],
            Self::Horizontal {
                x_position,
                y_position,
            } => vec![x_position, y_position],
            Self::Volume {
                x_position,
                y_position,
                z_position,
            } => vec![x_position, y_position, z_position],
        }
    }

    /// Returns the file `MemoryOrder` attribute, blank-padded to three bytes.
    pub(crate) const fn attribute_text(&self) -> &'static str {
        match self {
            Self::Scalar => "0  ",
            Self::Vertical => "Z  ",
            Self::Constant => "C  ",
            Self::Horizontal { .. } => "XY ",
            Self::Volume { .. } => "XYZ",
        }
    }

    fn unsupported(state: &str, axes: &[CoordinateAxis]) -> WrfIoError {
        let memory_order = axes
            .iter()
            .map(|axis| match axis {
                CoordinateAxis::X => 'X',
                CoordinateAxis::Y => 'Y',
                CoordinateAxis::Z => 'Z',
                CoordinateAxis::Constant => 'C',
            })
            .collect();
        WrfIoError::UnsupportedMemoryOrder {
            state: state.to_owned(),
            memory_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_sorts_every_supported_volume_order_into_external_xyz() {
        let cases = [
            (
                [CoordinateAxis::X, CoordinateAxis::Y, CoordinateAxis::Z],
                vec![0, 1, 2],
            ),
            (
                [CoordinateAxis::X, CoordinateAxis::Z, CoordinateAxis::Y],
                vec![0, 2, 1],
            ),
            (
                [CoordinateAxis::Y, CoordinateAxis::X, CoordinateAxis::Z],
                vec![1, 0, 2],
            ),
            (
                [CoordinateAxis::Y, CoordinateAxis::Z, CoordinateAxis::X],
                vec![2, 0, 1],
            ),
            (
                [CoordinateAxis::Z, CoordinateAxis::X, CoordinateAxis::Y],
                vec![1, 2, 0],
            ),
            (
                [CoordinateAxis::Z, CoordinateAxis::Y, CoordinateAxis::X],
                vec![2, 1, 0],
            ),
        ];

        for (axes, expected_permutation) in cases {
            let order = WrfMemoryOrder::try_new("field", &axes).unwrap();
            assert_eq!(order.external_permutation(), expected_permutation);
            assert_eq!(order.attribute_text(), "XYZ");
        }
    }

    #[test]
    fn try_new_supports_horizontal_vertical_constant_and_scalar_orders() {
        let horizontal =
            WrfMemoryOrder::try_new("mu", &[CoordinateAxis::Y, CoordinateAxis::X]).unwrap();
        assert_eq!(horizontal.external_permutation(), vec![1, 0]);
        assert_eq!(horizontal.attribute_text(), "XY ");

        assert_eq!(
            WrfMemoryOrder::try_new("profile", &[CoordinateAxis::Z])
                .unwrap()
                .attribute_text(),
            "Z  "
        );
        assert_eq!(
            WrfMemoryOrder::try_new("category", &[CoordinateAxis::Constant])
                .unwrap()
                .attribute_text(),
            "C  "
        );

        assert_eq!(
            WrfMemoryOrder::try_new("xtime", &[])
                .unwrap()
                .attribute_text(),
            "0  "
        );
    }

    #[test]
    fn try_new_rejects_orders_without_a_wrf_external_reordering() {
        for axes in [
            vec![CoordinateAxis::X],
            vec![CoordinateAxis::X, CoordinateAxis::Z],
            vec![CoordinateAxis::X, CoordinateAxis::X, CoordinateAxis::Y],
            vec![
                CoordinateAxis::X,
                CoordinateAxis::Y,
                CoordinateAxis::Constant,
            ],
        ] {
            assert!(matches!(
                WrfMemoryOrder::try_new("bad", &axes),
                Err(WrfIoError::UnsupportedMemoryOrder { .. })
            ));
        }
    }
}
