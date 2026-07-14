use std::path::Path;

use crate::corpus_writer::CorpusWriter;
use crate::deterministic_random::DeterministicRandom;
use crate::generator_error::GeneratorResult;

const SHEET_CASE_COUNT: usize = 24;
const SLAB_CASE_COUNT: usize = 16;
const QUIET_NAN_BITS: u32 = 0x7FC1_2345;
const POSITIVE_INFINITY_BITS: u32 = f32::INFINITY.to_bits();

pub(crate) struct PositiveDefiniteCorpus;

impl PositiveDefiniteCorpus {
    pub(crate) fn write(output_directory: &Path) -> GeneratorResult<()> {
        Self::write_sheet(output_directory)?;
        Self::write_slab(output_directory)
    }

    fn write_sheet(output_directory: &Path) -> GeneratorResult<()> {
        let mut writer =
            CorpusWriter::create(&output_directory.join("positive_definite_sheet.in"))?;
        writer.write_metadata(&[SHEET_CASE_COUNT as i64])?;

        for case_index in 0..SHEET_CASE_COUNT {
            let seed = 1_591_201_u64 + case_index as u64 * 104_729;
            let mut random = DeterministicRandom::new(seed);
            let west_east_points = random.usize_inclusive(2, 31);
            let south_north_points = random.usize_inclusive(1, 7);
            let mut totals = Vec::with_capacity(south_north_points);
            let mut values = Vec::with_capacity(west_east_points * south_north_points);

            for line_index in 0..south_north_points {
                totals.push(Self::sheet_total_bits(line_index, &mut random));
                for point_index in 0..west_east_points {
                    values.push(Self::sheet_value_bits(
                        case_index,
                        line_index,
                        point_index,
                        &mut random,
                    ));
                }
            }

            writer.write_metadata(&[
                seed as i64,
                west_east_points as i64,
                south_north_points as i64,
            ])?;
            writer.write_bits(&totals)?;
            writer.write_bits(&values)?;
        }

        writer.finish()
    }

    fn sheet_total_bits(line_index: usize, random: &mut DeterministicRandom) -> u32 {
        match line_index % 5 {
            2 => (random.i32_inclusive(-4_000, -1) as f32 * 0.25).to_bits(),
            4 => 1.0e20_f32.to_bits(),
            _ => (random.i32_inclusive(1, 8_000) as f32 * 0.25).to_bits(),
        }
    }

    fn sheet_value_bits(
        case_index: usize,
        line_index: usize,
        point_index: usize,
        random: &mut DeterministicRandom,
    ) -> u32 {
        if case_index + 1 == SHEET_CASE_COUNT && line_index == 0 {
            return match point_index {
                0 => QUIET_NAN_BITS,
                1 => POSITIVE_INFINITY_BITS,
                _ => (random.i32_inclusive(0, 16_000) as f32 * 0.062_5).to_bits(),
            };
        }

        match line_index % 5 {
            0 => match point_index {
                0 => (-0.0_f32).to_bits(),
                1 if case_index == 0 => 1,
                2 if case_index == 0 => 1.0e-30_f32.to_bits(),
                3 if case_index == 0 => 1.0e30_f32.to_bits(),
                _ => (random.i32_inclusive(0, 16_000) as f32 * 0.062_5).to_bits(),
            },
            1 | 2 => {
                if point_index == 0 {
                    (random.i32_inclusive(-8_000, -1) as f32 * 0.125).to_bits()
                } else {
                    (random.i32_inclusive(0, 8_000) as f32 * 0.125).to_bits()
                }
            }
            3 => (-4.0_f32).to_bits(),
            4 => {
                if point_index == 0 {
                    (-1.0e30_f32).to_bits()
                } else {
                    1.0e30_f32.to_bits()
                }
            }
            _ => unreachable!("line mode is reduced modulo five"),
        }
    }

    fn write_slab(output_directory: &Path) -> GeneratorResult<()> {
        let mut writer = CorpusWriter::create(&output_directory.join("positive_definite_slab.in"))?;
        writer.write_metadata(&[SLAB_CASE_COUNT as i64])?;

        for case_index in 0..SLAB_CASE_COUNT {
            let seed = 2_771_003_u64 + case_index as u64 * 130_363;
            let mut random = DeterministicRandom::new(seed);
            let memory_west_east_start = -3 + case_index as i32 % 5;
            let memory_south_north_start = -4 + case_index as i32 % 6;
            let memory_bottom_top_start = -1 + case_index as i32 % 3;
            let west_east_points = random.usize_inclusive(8, 15) as i32;
            let south_north_points = random.usize_inclusive(6, 12) as i32;
            let bottom_top_points = random.usize_inclusive(5, 8) as i32;
            let memory_west_east_end = memory_west_east_start + west_east_points - 1;
            let memory_south_north_end = memory_south_north_start + south_north_points - 1;
            let memory_bottom_top_end = memory_bottom_top_start + bottom_top_points - 1;
            let domain_west_east_start = memory_west_east_start + 1;
            let domain_west_east_end = memory_west_east_end - 1;
            let domain_south_north_start = memory_south_north_start + 1;
            let domain_south_north_end = memory_south_north_end - 1;
            let domain_bottom_top_start = memory_bottom_top_start;
            let domain_bottom_top_end = memory_bottom_top_end;
            let tile_west_east_start = domain_west_east_start;
            let tile_west_east_end = domain_west_east_end;
            let tile_south_north_start = domain_south_north_start + case_index as i32 % 2;
            let tile_south_north_end = domain_south_north_end - (case_index as i32 / 2) % 2;
            let tile_bottom_top_start = memory_bottom_top_start + 1;
            let tile_bottom_top_end = memory_bottom_top_end - case_index as i32 % 2;
            let metadata = [
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
            ];
            writer.write_metadata(&metadata)?;

            let mut values = Vec::with_capacity(
                west_east_points as usize
                    * south_north_points as usize
                    * bottom_top_points as usize,
            );
            let exceptional_line_index = ((tile_south_north_start - memory_south_north_start)
                * bottom_top_points
                + tile_bottom_top_start
                - memory_bottom_top_start) as usize;
            for south_north_index in memory_south_north_start..=memory_south_north_end {
                for bottom_top_index in memory_bottom_top_start..=memory_bottom_top_end {
                    let line_index = ((south_north_index - memory_south_north_start)
                        * bottom_top_points
                        + bottom_top_index
                        - memory_bottom_top_start) as usize;
                    for west_east_index in memory_west_east_start..=memory_west_east_end {
                        values.push(Self::slab_value_bits(
                            case_index,
                            line_index,
                            exceptional_line_index,
                            west_east_index,
                            domain_west_east_start,
                            &mut random,
                        ));
                    }
                }
            }
            if case_index == 0 {
                let active_value_index = exceptional_line_index * west_east_points as usize
                    + (domain_west_east_start - memory_west_east_start) as usize;
                values[active_value_index] = (-0.0_f32).to_bits();
                values[active_value_index + 1] = 1.0e30_f32.to_bits();
            }
            writer.write_bits(&values)?;
        }

        writer.finish()
    }

    fn slab_value_bits(
        case_index: usize,
        line_index: usize,
        exceptional_line_index: usize,
        west_east_index: i32,
        active_west_east_start: i32,
        random: &mut DeterministicRandom,
    ) -> u32 {
        let point_index = west_east_index - active_west_east_start;
        if case_index + 1 == SLAB_CASE_COUNT && line_index == exceptional_line_index {
            return match point_index {
                0 => QUIET_NAN_BITS,
                1 => POSITIVE_INFINITY_BITS,
                _ => (random.i32_inclusive(1, 8_000) as f32 * 0.125).to_bits(),
            };
        }

        match line_index % 4 {
            0 => (random.i32_inclusive(0, 8_000) as f32 * 0.125).to_bits(),
            1 => {
                if point_index == 0 {
                    (-750.0_f32).to_bits()
                } else {
                    (random.i32_inclusive(0, 8_000) as f32 * 0.125).to_bits()
                }
            }
            2 => (-16.0_f32).to_bits(),
            3 => {
                if point_index == 0 {
                    (-1.0e30_f32).to_bits()
                } else {
                    1.0e30_f32.to_bits()
                }
            }
            _ => unreachable!("line mode is reduced modulo four"),
        }
    }
}
