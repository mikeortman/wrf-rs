use pulp::Simd;

const SIGMA_BOUNDARY: f32 = 0.7;
const DAY_LENGTH_SECONDS: f32 = 60.0 * 60.0 * 24.0;
const FRICTION_RATE: f32 = 1.0 / DAY_LENGTH_SECONDS;

pub(super) struct MomentumDampingLine<'a> {
    pub(super) tendency: &'a mut [f32],
    pub(super) momentum: &'a [f32],
    pub(super) current_pressure: &'a [f32],
    pub(super) current_base_pressure: &'a [f32],
    pub(super) adjacent_pressure: &'a [f32],
    pub(super) adjacent_base_pressure: &'a [f32],
    pub(super) current_surface_pressure: &'a [f32],
    pub(super) current_surface_base_pressure: &'a [f32],
    pub(super) adjacent_surface_pressure: &'a [f32],
    pub(super) adjacent_surface_base_pressure: &'a [f32],
}

#[inline(always)]
pub(super) fn damp_momentum_line<S: Simd>(simd: S, line: MomentumDampingLine<'_>) {
    let MomentumDampingLine {
        tendency,
        momentum,
        current_pressure,
        current_base_pressure,
        adjacent_pressure,
        adjacent_base_pressure,
        current_surface_pressure,
        current_surface_base_pressure,
        adjacent_surface_pressure,
        adjacent_surface_base_pressure,
    } = line;
    let (tendency_vectors, tendency_tail) = S::as_mut_simd_f32s(tendency);
    let (momentum_vectors, momentum_tail) = S::as_simd_f32s(momentum);
    let (current_pressure_vectors, current_pressure_tail) = S::as_simd_f32s(current_pressure);
    let (current_base_pressure_vectors, current_base_pressure_tail) =
        S::as_simd_f32s(current_base_pressure);
    let (adjacent_pressure_vectors, adjacent_pressure_tail) = S::as_simd_f32s(adjacent_pressure);
    let (adjacent_base_pressure_vectors, adjacent_base_pressure_tail) =
        S::as_simd_f32s(adjacent_base_pressure);
    let (current_surface_pressure_vectors, current_surface_pressure_tail) =
        S::as_simd_f32s(current_surface_pressure);
    let (current_surface_base_pressure_vectors, current_surface_base_pressure_tail) =
        S::as_simd_f32s(current_surface_base_pressure);
    let (adjacent_surface_pressure_vectors, adjacent_surface_pressure_tail) =
        S::as_simd_f32s(adjacent_surface_pressure);
    let (adjacent_surface_base_pressure_vectors, adjacent_surface_base_pressure_tail) =
        S::as_simd_f32s(adjacent_surface_base_pressure);

    debug_assert_eq!(tendency_vectors.len(), momentum_vectors.len());
    debug_assert_eq!(tendency_tail.len(), momentum_tail.len());
    let zero = simd.splat_f32s(0.0);
    let sigma_boundary = simd.splat_f32s(SIGMA_BOUNDARY);
    let sigma_transition_width = simd.splat_f32s(1.0 - SIGMA_BOUNDARY);
    let friction_rate = simd.splat_f32s(FRICTION_RATE);
    for vector_index in 0..tendency_vectors.len() {
        let numerator = simd.add_f32s(
            adjacent_pressure_vectors[vector_index],
            adjacent_base_pressure_vectors[vector_index],
        );
        let numerator = simd.add_f32s(numerator, current_pressure_vectors[vector_index]);
        let numerator = simd.add_f32s(numerator, current_base_pressure_vectors[vector_index]);
        let denominator = simd.add_f32s(
            adjacent_surface_pressure_vectors[vector_index],
            adjacent_surface_base_pressure_vectors[vector_index],
        );
        let denominator =
            simd.add_f32s(denominator, current_surface_pressure_vectors[vector_index]);
        let denominator = simd.add_f32s(
            denominator,
            current_surface_base_pressure_vectors[vector_index],
        );
        let sigma = simd.div_f32s(numerator, denominator);
        let sigma_term =
            simd.div_f32s(simd.sub_f32s(sigma, sigma_boundary), sigma_transition_width);
        let sigma_term =
            simd.select_f32s(simd.greater_than_f32s(sigma_term, zero), sigma_term, zero);
        let vertical_damping = simd.mul_f32s(friction_rate, sigma_term);
        tendency_vectors[vector_index] = simd.sub_f32s(
            tendency_vectors[vector_index],
            simd.mul_f32s(vertical_damping, momentum_vectors[vector_index]),
        );
    }

    damp_scalar_tail(MomentumDampingLine {
        tendency: tendency_tail,
        momentum: momentum_tail,
        current_pressure: current_pressure_tail,
        current_base_pressure: current_base_pressure_tail,
        adjacent_pressure: adjacent_pressure_tail,
        adjacent_base_pressure: adjacent_base_pressure_tail,
        current_surface_pressure: current_surface_pressure_tail,
        current_surface_base_pressure: current_surface_base_pressure_tail,
        adjacent_surface_pressure: adjacent_surface_pressure_tail,
        adjacent_surface_base_pressure: adjacent_surface_base_pressure_tail,
    });
}

#[inline(always)]
fn damp_scalar_tail(line: MomentumDampingLine<'_>) {
    for index in 0..line.tendency.len() {
        let sigma = (line.adjacent_pressure[index]
            + line.adjacent_base_pressure[index]
            + line.current_pressure[index]
            + line.current_base_pressure[index])
            / (line.adjacent_surface_pressure[index]
                + line.adjacent_surface_base_pressure[index]
                + line.current_surface_pressure[index]
                + line.current_surface_base_pressure[index]);
        let sigma_term = 0.0_f32.max((sigma - SIGMA_BOUNDARY) / (1.0 - SIGMA_BOUNDARY));
        let vertical_damping = FRICTION_RATE * sigma_term;
        line.tendency[index] -= vertical_damping * line.momentum[index];
    }
}

#[cfg(test)]
mod tests {
    use pulp::{Arch, WithSimd};

    use super::*;

    #[test]
    fn runtime_simd_matches_scalar_bits_across_vector_and_tail_lengths() {
        for length in 1..=257 {
            let inputs = create_inputs(length);
            let mut expected = inputs.tendency.clone();
            let mut actual = inputs.tendency.clone();

            damp_momentum_line(pulp::Scalar, inputs.line(&mut expected));
            Arch::new().dispatch(ApplyRuntimeSimd(inputs.line(&mut actual)));

            assert_eq!(
                actual
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "line length {length}"
            );
        }
    }

    struct ApplyRuntimeSimd<'a>(MomentumDampingLine<'a>);

    impl WithSimd for ApplyRuntimeSimd<'_> {
        type Output = ();

        fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
            damp_momentum_line(simd, self.0);
        }
    }

    struct LineInputs {
        tendency: Vec<f32>,
        momentum: Vec<f32>,
        current_pressure: Vec<f32>,
        current_base_pressure: Vec<f32>,
        adjacent_pressure: Vec<f32>,
        adjacent_base_pressure: Vec<f32>,
        current_surface_pressure: Vec<f32>,
        current_surface_base_pressure: Vec<f32>,
        adjacent_surface_pressure: Vec<f32>,
        adjacent_surface_base_pressure: Vec<f32>,
    }

    impl LineInputs {
        fn line<'a>(&'a self, tendency: &'a mut [f32]) -> MomentumDampingLine<'a> {
            MomentumDampingLine {
                tendency,
                momentum: &self.momentum,
                current_pressure: &self.current_pressure,
                current_base_pressure: &self.current_base_pressure,
                adjacent_pressure: &self.adjacent_pressure,
                adjacent_base_pressure: &self.adjacent_base_pressure,
                current_surface_pressure: &self.current_surface_pressure,
                current_surface_base_pressure: &self.current_surface_base_pressure,
                adjacent_surface_pressure: &self.adjacent_surface_pressure,
                adjacent_surface_base_pressure: &self.adjacent_surface_base_pressure,
            }
        }
    }

    fn create_inputs(length: usize) -> LineInputs {
        let indices = (0..length).map(|index| index as f32).collect::<Vec<_>>();
        LineInputs {
            tendency: indices.iter().map(|index| 0.1 + index * 0.001).collect(),
            momentum: indices.iter().map(|index| -8.0 + index * 0.02).collect(),
            current_pressure: indices.iter().map(|index| index * 0.125).collect(),
            current_base_pressure: indices
                .iter()
                .enumerate()
                .map(|(index, _)| if index % 3 == 0 { 65_000.0 } else { 85_000.0 })
                .collect(),
            adjacent_pressure: indices.iter().map(|index| index * 0.25).collect(),
            adjacent_base_pressure: vec![0.0; length],
            current_surface_pressure: indices.iter().map(|index| index * 0.125).collect(),
            current_surface_base_pressure: vec![100_000.0; length],
            adjacent_surface_pressure: indices.iter().map(|index| index * 0.25).collect(),
            adjacent_surface_base_pressure: vec![0.0; length],
        }
    }
}
