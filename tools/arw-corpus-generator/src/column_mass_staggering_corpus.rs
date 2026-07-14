use std::path::Path;

use crate::corpus_writer::CorpusWriter;
use crate::deterministic_random::DeterministicRandom;
use crate::generator_error::GeneratorResult;

const CASE_COUNT: usize = 16;
const QUIET_NAN_BITS: u32 = 0x7FC2_468A;

pub(crate) struct ColumnMassStaggeringCorpus;

impl ColumnMassStaggeringCorpus {
    pub(crate) fn write(output_directory: &Path) -> GeneratorResult<()> {
        let mut writer = CorpusWriter::create(&output_directory.join("column_mass_staggering.in"))?;
        writer.write_metadata(&[CASE_COUNT as i64])?;

        for case_index in 0..CASE_COUNT {
            let seed = 5_100_007_u64 + case_index as u64 * 179_429;
            let mut random = DeterministicRandom::new(seed);
            let memory_west_east_start = -4 + case_index as i32 % 6;
            let memory_south_north_start = -3 + case_index as i32 % 5;
            let west_east_points = random.usize_inclusive(10, 19) as i32;
            let south_north_points = random.usize_inclusive(9, 17) as i32;
            let memory_west_east_end = memory_west_east_start + west_east_points - 1;
            let memory_south_north_end = memory_south_north_start + south_north_points - 1;
            let domain_west_east_start = memory_west_east_start + 2;
            let domain_west_east_end = memory_west_east_end - 2;
            let domain_south_north_start = memory_south_north_start + 2;
            let domain_south_north_end = memory_south_north_end - 2;
            let west_east_path = case_index % 4;
            let south_north_path = case_index / 4 % 4;
            let (tile_west_east_start, tile_west_east_end) =
                Self::tile_bounds(domain_west_east_start, domain_west_east_end, west_east_path);
            let (tile_south_north_start, tile_south_north_end) = Self::tile_bounds(
                domain_south_north_start,
                domain_south_north_end,
                south_north_path,
            );
            writer.write_metadata(&[
                seed as i64,
                domain_west_east_start as i64,
                domain_west_east_end as i64,
                domain_south_north_start as i64,
                domain_south_north_end as i64,
                1,
                2,
                memory_west_east_start as i64,
                memory_west_east_end as i64,
                memory_south_north_start as i64,
                memory_south_north_end as i64,
                1,
                1,
                tile_west_east_start as i64,
                tile_west_east_end as i64,
                tile_south_north_start as i64,
                tile_south_north_end as i64,
                1,
                1,
            ])?;

            let value_count = west_east_points as usize * south_north_points as usize;
            let mut perturbation_mass = Vec::with_capacity(value_count);
            let mut base_mass = Vec::with_capacity(value_count);
            for _ in 0..value_count {
                perturbation_mass.push(random.moderate_f32_bits());
                base_mass.push(
                    (90_000.0_f32 + random.i32_inclusive(-8_000, 8_000) as f32 * 0.125).to_bits(),
                );
            }
            if case_index == 0 {
                let active_index = (tile_south_north_start - memory_south_north_start) as usize
                    * west_east_points as usize
                    + (tile_west_east_start - memory_west_east_start) as usize;
                perturbation_mass[active_index] = (-0.0_f32).to_bits();
                if active_index + 1 < value_count {
                    perturbation_mass[active_index + 1] = 1.0e30_f32.to_bits();
                    base_mass[active_index + 1] = (-1.0e30_f32).to_bits();
                }
            }
            if case_index + 1 == CASE_COUNT {
                let active_index = (tile_south_north_start - memory_south_north_start) as usize
                    * west_east_points as usize
                    + (tile_west_east_start - memory_west_east_start) as usize;
                perturbation_mass[active_index] = QUIET_NAN_BITS;
                if active_index + 1 < value_count {
                    perturbation_mass[active_index + 1] = f32::INFINITY.to_bits();
                }
            }
            writer.write_bits(&perturbation_mass)?;
            writer.write_bits(&base_mass)?;
        }

        writer.finish()
    }

    fn tile_bounds(domain_start: i32, domain_end: i32, path: usize) -> (i32, i32) {
        match path {
            0 => (domain_start + 1, domain_end - 1),
            1 => (domain_start, domain_end - 1),
            2 => (domain_start + 1, domain_end),
            3 => (domain_start, domain_end),
            _ => unreachable!("boundary path is reduced modulo four"),
        }
    }
}
