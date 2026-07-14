"""Unit tests for queryable tracking and benchmark normalization."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "tracking.py"
SPEC = importlib.util.spec_from_file_location("wrf_tracking", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
tracking = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(tracking)


class FortranSampleTests(unittest.TestCase):
    def test_parses_supported_output_families(self) -> None:
        output = """
compiler GNU Fortran
sample_1_milliseconds_per_call 1.250000
sample_2_milliseconds_per_call 1.500000
sheet_sample_1_milliseconds_per_call 2.500000
nonhydrostatic 3.750000
  4.125000
checksum 9.999999E+12
"""
        self.assertEqual(
            tracking.parse_fortran_samples(output),
            {
                "default": [1.25, 1.5, 4.125],
                "sheet": [2.5],
                "nonhydrostatic": [3.75],
            },
        )

    def test_ignores_metadata_values(self) -> None:
        output = "grid_points_per_call 655360\nchecksum 1.0E+09\n"
        self.assertEqual(tracking.parse_fortran_samples(output), {})


class SelectionTests(unittest.TestCase):
    def test_selects_a_scientific_family(self) -> None:
        selected = tracking.select_suites(
            ["crates/wrf-dynamics/src/held_suarez/cpu.rs"]
        )
        self.assertEqual([suite["id"] for suite in selected], ["held-suarez"])

    def test_shared_compute_change_selects_every_suite(self) -> None:
        selected = tracking.select_suites(["crates/wrf-compute/src/cpu.rs"])
        self.assertEqual(len(selected), len(tracking.load_benchmarks()["suites"]))

    def test_docs_only_change_selects_nothing(self) -> None:
        self.assertEqual(tracking.select_suites(["docs/wiki/Home.md"]), [])


class RelativeTextTests(unittest.TestCase):
    def test_close_results_are_tied(self) -> None:
        self.assertIn("effectively tied", tracking.relative_text(1.01, 1.0))

    def test_reports_material_direction(self) -> None:
        self.assertEqual(tracking.relative_text(0.5, 1.0), "Rust 2.00x faster")
        self.assertEqual(tracking.relative_text(2.0, 1.0), "Rust 2.00x slower")


class CumulativeMatrixTests(unittest.TestCase):
    def test_current_results_replace_only_their_suite(self) -> None:
        previous = [
            {"id": "a", "label": "A", "source_sha": "old-a"},
            {"id": "b", "label": "B", "source_sha": "old-b"},
        ]
        current = [{"id": "b", "label": "B", "source_sha": "new-b"}]
        self.assertEqual(
            tracking.merge_result_rows(previous, current),
            [
                {"id": "a", "label": "A", "source_sha": "old-a"},
                {"id": "b", "label": "B", "source_sha": "new-b"},
            ],
        )


if __name__ == "__main__":
    unittest.main()
