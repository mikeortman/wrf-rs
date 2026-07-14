/// Value policy used when coupled velocity identifies boundary inflow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpecifiedBoundaryInflowPolicy {
    /// Write positive zero, matching WRF `flow_dep_bdy`.
    Zero,
    /// Write one caller-supplied concentration, matching `flow_dep_bdy_qnn`.
    Constant(f32),
    /// Retain the current destination, matching `flow_dep_bdy_fixed_inflow`.
    Preserve,
}

impl SpecifiedBoundaryInflowPolicy {
    pub(crate) fn apply(self, destination: &mut f32) {
        match self {
            Self::Zero => *destination = 0.0,
            Self::Constant(value) => *destination = value,
            Self::Preserve => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policies_clear_replace_or_preserve_exact_bits() {
        let mut value = -0.0_f32;
        SpecifiedBoundaryInflowPolicy::Zero.apply(&mut value);
        assert_eq!(value.to_bits(), 0.0_f32.to_bits());

        SpecifiedBoundaryInflowPolicy::Constant(f32::from_bits(0x7FC0_1234)).apply(&mut value);
        assert_eq!(value.to_bits(), 0x7FC0_1234);

        SpecifiedBoundaryInflowPolicy::Preserve.apply(&mut value);
        assert_eq!(value.to_bits(), 0x7FC0_1234);
    }
}
