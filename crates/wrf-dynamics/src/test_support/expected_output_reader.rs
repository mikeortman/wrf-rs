use std::str::Lines;

pub(crate) struct ExpectedOutputReader<'a> {
    lines: Lines<'a>,
}

impl<'a> ExpectedOutputReader<'a> {
    pub(crate) fn new(expected_output: &'a str) -> Self {
        Self {
            lines: expected_output.lines(),
        }
    }

    pub(crate) fn assert_next(
        &mut self,
        seed: u64,
        field_name: &str,
        value_index: usize,
        actual_value: f32,
    ) {
        let line = self.lines.next().unwrap_or_else(|| {
            panic!(
                "Fortran output ended before seed {seed}, field {field_name}, index {value_index}"
            )
        });
        let mut columns = line.split_whitespace();
        let expected_seed = Self::parse_u64(columns.next(), "seed", line);
        let expected_field_name = columns
            .next()
            .unwrap_or_else(|| panic!("missing field name in expected output line: {line}"));
        let expected_index = Self::parse_usize(columns.next(), "index", line);
        let expected_value = columns
            .next()
            .unwrap_or_else(|| panic!("missing value in expected output line: {line}"));
        assert_eq!(
            columns.next(),
            None,
            "extra columns in expected output: {line}"
        );
        assert_eq!(expected_seed, seed, "unexpected case seed at {line}");
        assert_eq!(
            expected_field_name, field_name,
            "unexpected field for seed {seed}, index {value_index}"
        );
        assert_eq!(
            expected_index, value_index,
            "unexpected index for seed {seed}, field {field_name}"
        );

        if expected_value == "NAN" {
            assert!(
                actual_value.is_nan(),
                "seed {seed}, field {field_name}, index {value_index}: expected NaN, got {actual_value:?} ({:08X})",
                actual_value.to_bits()
            );
            return;
        }

        let expected_bits = u32::from_str_radix(expected_value, 16).unwrap_or_else(|error| {
            panic!("invalid value bits in expected output line {line}: {error}")
        });
        assert_eq!(
            actual_value.to_bits(),
            expected_bits,
            "seed {seed}, field {field_name}, index {value_index}: expected {expected_bits:08X}, got {:08X}",
            actual_value.to_bits()
        );
    }

    pub(crate) fn finish(mut self) {
        assert_eq!(
            self.lines.next(),
            None,
            "Fortran expected output contains unconsumed values"
        );
    }

    fn parse_u64(value: Option<&str>, column_name: &str, line: &str) -> u64 {
        value
            .unwrap_or_else(|| panic!("missing {column_name} in expected output line: {line}"))
            .parse()
            .unwrap_or_else(|error| {
                panic!("invalid {column_name} in expected output line {line}: {error}")
            })
    }

    fn parse_usize(value: Option<&str>, column_name: &str, line: &str) -> usize {
        value
            .unwrap_or_else(|| panic!("missing {column_name} in expected output line: {line}"))
            .parse()
            .unwrap_or_else(|error| {
                panic!("invalid {column_name} in expected output line {line}: {error}")
            })
    }
}
