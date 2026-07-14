use std::path::Path;

use crate::corpus_writer::CorpusWriter;
use crate::deterministic_random::DeterministicRandom;
use crate::generator_error::GeneratorResult;

const CASE_COUNT: usize = 12;
const QUIET_NAN_BITS: u32 = 0x7FC5_4321;

pub(crate) struct HeldSuarezCorpus;

impl HeldSuarezCorpus {
    pub(crate) fn write(output_directory: &Path) -> GeneratorResult<()> {
        let mut writer = CorpusWriter::create(&output_directory.join("held_suarez.in"))?;
        writer.write_metadata(&[CASE_COUNT as i64])?;

        for case_index in 0..CASE_COUNT {
            let seed = 3_901_111_u64 + case_index as u64 * 154_858;
            let mut random = DeterministicRandom::new(seed);
            let memory_west_east_start = -3 + case_index as i32 % 3;
            let memory_south_north_start = -4 + case_index as i32 % 4;
            let memory_bottom_top_start = -1 + case_index as i32 % 2;
            let west_east_points = random.usize_inclusive(9, 15) as i32;
            let south_north_points = random.usize_inclusive(8, 14) as i32;
            let bottom_top_points = random.usize_inclusive(6, 8) as i32;
            let memory_west_east_end = memory_west_east_start + west_east_points - 1;
            let memory_south_north_end = memory_south_north_start + south_north_points - 1;
            let memory_bottom_top_end = memory_bottom_top_start + bottom_top_points - 1;
            let domain_west_east_start = memory_west_east_start + 1;
            let domain_west_east_end = memory_west_east_end - 1;
            let domain_south_north_start = memory_south_north_start + 1;
            let domain_south_north_end = memory_south_north_end - 1;
            let domain_bottom_top_start = 1;
            let domain_bottom_top_end = memory_bottom_top_end;
            let tile_west_east_start = domain_west_east_start + case_index as i32 % 2;
            let tile_west_east_end = domain_west_east_end - (case_index as i32 / 2) % 2;
            let tile_south_north_start = domain_south_north_start + case_index as i32 % 2;
            let tile_south_north_end = domain_south_north_end - (case_index as i32 / 3) % 2;
            let tile_bottom_top_start = if case_index + 1 == CASE_COUNT {
                1
            } else {
                1 + case_index as i32 % 2
            };
            let tile_bottom_top_end = domain_bottom_top_end;
            writer.write_metadata(&[
                seed as i64,
                domain_west_east_start as i64,
                domain_west_east_end as i64,
                domain_south_north_start as i64,
                domain_south_north_end as i64,
                domain_bottom_top_start as i64,
                domain_bottom_top_end as i64,
                memory_west_east_start as i64,
                memory_west_east_end as i64,
                memory_south_north_start as i64,
                memory_south_north_end as i64,
                memory_bottom_top_start as i64,
                memory_bottom_top_end as i64,
                tile_west_east_start as i64,
                tile_west_east_end as i64,
                tile_south_north_start as i64,
                tile_south_north_end as i64,
                tile_bottom_top_start as i64,
                tile_bottom_top_end as i64,
            ])?;

            let value_count = west_east_points as usize
                * south_north_points as usize
                * bottom_top_points as usize;
            let mut west_east_tendency = Vec::with_capacity(value_count);
            let mut south_north_tendency = Vec::with_capacity(value_count);
            let mut west_east_momentum = Vec::with_capacity(value_count);
            let mut south_north_momentum = Vec::with_capacity(value_count);
            let mut perturbation_pressure = Vec::with_capacity(value_count);
            let mut base_pressure = Vec::with_capacity(value_count);

            for south_north_index in memory_south_north_start..=memory_south_north_end {
                for bottom_top_index in memory_bottom_top_start..=memory_bottom_top_end {
                    for west_east_index in memory_west_east_start..=memory_west_east_end {
                        west_east_tendency.push(random.moderate_f32_bits());
                        south_north_tendency.push(random.moderate_f32_bits());
                        west_east_momentum.push(random.moderate_f32_bits());
                        south_north_momentum.push(random.moderate_f32_bits());
                        perturbation_pressure
                            .push((random.i32_inclusive(-750, 750) as f32).to_bits());
                        base_pressure.push(
                            ((100_000 - (bottom_top_index - 1) * 12_000
                                + random.i32_inclusive(-500, 500)
                                + west_east_index * 3
                                + south_north_index * 5)
                                .max(20_000) as f32)
                                .to_bits(),
                        );
                    }
                }
            }

            if case_index == 0 {
                let active_index = Self::linear_index(
                    tile_west_east_start,
                    tile_bottom_top_start,
                    tile_south_north_start,
                    memory_west_east_start,
                    memory_bottom_top_start,
                    memory_south_north_start,
                    west_east_points as usize,
                    bottom_top_points as usize,
                );
                west_east_momentum[active_index] = (-0.0_f32).to_bits();
                west_east_tendency[active_index] = (-0.0_f32).to_bits();
                if active_index + 1 < value_count {
                    west_east_momentum[active_index + 1] = 1.0e30_f32.to_bits();
                }
            }
            if case_index + 1 == CASE_COUNT {
                let active_index = Self::linear_index(
                    tile_west_east_start,
                    1,
                    tile_south_north_start,
                    memory_west_east_start,
                    memory_bottom_top_start,
                    memory_south_north_start,
                    west_east_points as usize,
                    bottom_top_points as usize,
                );
                west_east_momentum[active_index] = QUIET_NAN_BITS;
                if active_index + 1 < value_count {
                    west_east_momentum[active_index + 1] = f32::INFINITY.to_bits();
                }
                let south_north_active_index = Self::linear_index(
                    tile_west_east_start,
                    1,
                    (domain_south_north_start + 1).max(tile_south_north_start),
                    memory_west_east_start,
                    memory_bottom_top_start,
                    memory_south_north_start,
                    west_east_points as usize,
                    bottom_top_points as usize,
                );
                south_north_momentum[south_north_active_index] = QUIET_NAN_BITS;
                if south_north_active_index + 1 < value_count {
                    south_north_momentum[south_north_active_index + 1] =
                        f32::NEG_INFINITY.to_bits();
                }
            }

            writer.write_bits(&west_east_tendency)?;
            writer.write_bits(&south_north_tendency)?;
            writer.write_bits(&west_east_momentum)?;
            writer.write_bits(&south_north_momentum)?;
            writer.write_bits(&perturbation_pressure)?;
            writer.write_bits(&base_pressure)?;
        }

        writer.finish()
    }

    #[allow(clippy::too_many_arguments)]
    fn linear_index(
        west_east_index: i32,
        bottom_top_index: i32,
        south_north_index: i32,
        memory_west_east_start: i32,
        memory_bottom_top_start: i32,
        memory_south_north_start: i32,
        west_east_points: usize,
        bottom_top_points: usize,
    ) -> usize {
        let west_east_offset = (west_east_index - memory_west_east_start) as usize;
        let bottom_top_offset = (bottom_top_index - memory_bottom_top_start) as usize;
        let south_north_offset = (south_north_index - memory_south_north_start) as usize;
        (south_north_offset * bottom_top_points + bottom_top_offset) * west_east_points
            + west_east_offset
    }
}
