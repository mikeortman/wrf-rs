"""Unit tests for queryable tracking and benchmark normalization."""

from __future__ import annotations

import copy
import importlib.util
import json
import os
import re
import tempfile
import unittest
from pathlib import Path
from unittest import mock


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
    @staticmethod
    def catalogs_with_changed_suite(
        identifier: str,
    ) -> tuple[dict[str, object], dict[str, object]]:
        current = tracking.load_benchmarks()
        previous = copy.deepcopy(current)
        suite = next(suite for suite in previous["suites"] if suite["id"] == identifier)
        suite["label"] = f"{suite['label']} before change"
        return previous, current

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

    def test_kessler_driver_change_selects_only_coupled_trajectory(self) -> None:
        selected = tracking.select_suites(
            ["crates/wrf-physics/src/microphysics/driver/microphysics_driver.rs"]
        )
        self.assertEqual(
            [suite["id"] for suite in selected],
            ["kessler-precipitation-trajectory"],
        )

    def test_catalog_only_change_selects_the_changed_current_suite(self) -> None:
        previous, current = self.catalogs_with_changed_suite("held-suarez")
        selected = tracking.select_suites(
            ["tracking/benchmarks.json"],
            benchmarks=current,
            previous_benchmarks=previous,
        )
        self.assertEqual([suite["id"] for suite in selected], ["held-suarez"])

    def test_catalog_and_driver_change_select_only_the_changed_current_suite(self) -> None:
        current = tracking.load_benchmarks()
        previous = copy.deepcopy(current)
        previous["suites"] = [
            suite
            for suite in previous["suites"]
            if suite["id"] != "kessler-precipitation-trajectory"
        ]
        previous["global_watch"] = [
            *current["global_watch"],
            "tools/tracking.py",
            "tracking/benchmarks.json",
        ]
        selected = tracking.select_suites(
            [
                "tracking/benchmarks.json",
                "crates/wrf-physics/src/microphysics/driver/microphysics_driver.rs",
            ],
            benchmarks=current,
            previous_benchmarks=previous,
        )
        self.assertEqual(
            [suite["id"] for suite in selected],
            ["kessler-precipitation-trajectory"],
        )

    def test_routing_catalog_and_driver_change_select_only_changed_suite(self) -> None:
        current = tracking.load_benchmarks()
        previous = copy.deepcopy(current)
        previous["suites"] = [
            suite
            for suite in previous["suites"]
            if suite["id"] != "kessler-precipitation-trajectory"
        ]

        selected = tracking.select_suites(
            [
                "tools/tracking.py",
                "tracking/benchmarks.json",
                "crates/wrf-physics/src/microphysics/driver/microphysics_driver.rs",
            ],
            benchmarks=current,
            previous_benchmarks=previous,
        )

        self.assertEqual(
            [suite["id"] for suite in selected],
            ["kessler-precipitation-trajectory"],
        )

    def test_routing_code_change_without_catalog_change_selects_every_suite(self) -> None:
        selected = tracking.select_suites(["tools/tracking.py"])
        self.assertEqual(len(selected), len(tracking.load_benchmarks()["suites"]))

    def test_top_level_catalog_change_selects_every_current_suite(self) -> None:
        current = tracking.load_benchmarks()
        previous = copy.deepcopy(current)
        previous["ci_runner_class"] = "previous-runner"

        selected = tracking.select_suites(
            ["tracking/benchmarks.json"],
            benchmarks=current,
            previous_benchmarks=previous,
        )

        self.assertEqual(len(selected), len(current["suites"]))

    def test_catalog_without_revision_context_safely_selects_every_suite(self) -> None:
        selected = tracking.select_suites(["tracking/benchmarks.json"])
        self.assertEqual(len(selected), len(tracking.load_benchmarks()["suites"]))

    def test_run_all_still_selects_every_suite(self) -> None:
        selected = tracking.select_suites(["docs/wiki/Home.md"], run_all=True)
        self.assertEqual(len(selected), len(tracking.load_benchmarks()["suites"]))


class RelativeTextTests(unittest.TestCase):
    def test_close_results_are_tied(self) -> None:
        self.assertIn("effectively tied", tracking.relative_text(1.01, 1.0))

    def test_reports_material_direction(self) -> None:
        self.assertEqual(tracking.relative_text(0.5, 1.0), "Rust 2.00x faster")
        self.assertEqual(tracking.relative_text(2.0, 1.0), "Rust 2.00x slower")


class PercentileTests(unittest.TestCase):
    def test_interpolates_p50_p90_and_p99(self) -> None:
        self.assertEqual(
            tracking.sample_percentiles([1.0, 2.0, 3.0, 4.0]),
            {"p50": 2.5, "p90": 3.7, "p99": 3.9699999999999998},
        )

    def test_rejects_an_empty_sample(self) -> None:
        with self.assertRaisesRegex(tracking.TrackingError, "empty sample"):
            tracking.percentile([], 0.5)

    def test_reads_criterion_latency_samples_per_iteration(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            target = Path(temporary_directory)
            result_directory = target / "criterion/group/case/4/new"
            result_directory.mkdir(parents=True)
            (result_directory / "estimates.json").write_text(
                json.dumps(
                    {
                        "median": {
                            "point_estimate": 2_500_000.0,
                            "confidence_interval": {
                                "lower_bound": 2_400_000.0,
                                "upper_bound": 2_600_000.0,
                            },
                        }
                    }
                ),
                encoding="utf-8",
            )
            (result_directory / "sample.json").write_text(
                json.dumps(
                    {
                        "iters": [1.0, 2.0, 4.0, 8.0],
                        "times": [1_000_000.0, 4_000_000.0, 12_000_000.0, 32_000_000.0],
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": str(target)}):
                estimate = tracking.criterion_estimates(
                    {"criterion_group": "group", "rust_case": "case"}
                )[0]
        self.assertEqual(estimate["workers"], 4)
        self.assertEqual(estimate["percentiles_nanoseconds"]["p50"], 2_500_000.0)
        self.assertEqual(estimate["percentiles_nanoseconds"]["p90"], 3_700_000.0)


class ResultRowTests(unittest.TestCase):
    def test_exposes_distribution_and_green_performance_status(self) -> None:
        result = {
            "suite": {"id": "suite", "label": "Suite"},
            "source_sha": "12345678",
            "collected_at": "2026-07-14T12:00:00+00:00",
            "runner": {"classification": "test"},
            "fortran": {
                "median_milliseconds": 2.0,
                "samples_milliseconds": [1.9, 2.0, 2.2],
                "percentiles_milliseconds": {"p50": 2.0, "p90": 2.16, "p99": 2.196},
            },
            "rust": {
                "estimates": [
                    {
                        "workers": 1,
                        "median_nanoseconds": 1_500_000.0,
                        "percentiles_nanoseconds": {
                            "p50": 1_500_000.0,
                            "p90": 1_700_000.0,
                            "p99": 1_900_000.0,
                        },
                    }
                ]
            },
        }
        row = tracking.result_rows([result])[0]
        self.assertEqual(row["performance_status"], "passing")
        self.assertEqual(row["rust_best_percentiles_milliseconds"]["p99"], 1.9)
        self.assertEqual(row["best_ratio"], 0.75)


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

    def test_history_appends_new_runs_without_duplicates(self) -> None:
        old = {"a": [{"id": "a", "source_sha": "old", "collected_at": "first"}]}
        current = {"id": "a", "source_sha": "new", "collected_at": "second"}
        merged = tracking.merge_result_history(old, [current, current])
        self.assertEqual([row["source_sha"] for row in merged["a"]], ["old", "new"])

    def test_schema_two_summary_becomes_one_point_history(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "summary.json"
            path.write_text(
                json.dumps(
                    {
                        "schema_version": 2,
                        "results": [{"id": "suite", "source_sha": "old"}],
                    }
                ),
                encoding="utf-8",
            )
            loaded = tracking.load_previous_summary(path)
        self.assertEqual(loaded["history"]["suite"][0]["source_sha"], "old")


class PagesTests(unittest.TestCase):
    @staticmethod
    def sample_row() -> dict[str, object]:
        return {
            "id": "sample-suite",
            "label": "Sample suite",
            "fortran_milliseconds": 2.0,
            "rust_serial_milliseconds": 2.5,
            "rust_best_milliseconds": 1.5,
            "rust_best_workers": 4,
            "fortran_percentiles_milliseconds": {"p50": 2.0, "p90": 2.2, "p99": 2.4},
            "rust_serial_percentiles_milliseconds": {"p50": 2.5, "p90": 2.7, "p99": 3.0},
            "rust_best_percentiles_milliseconds": {"p50": 1.5, "p90": 1.7, "p99": 1.9},
            "best_relative": "Rust 1.33x faster",
            "source_sha": "1234567890abcdef",
            "collected_at": "2026-07-14T12:00:00+00:00",
            "runner": {"classification": "test-runner"},
        }

    def test_builds_dashboard_suite_history_and_documentation(self) -> None:
        row = self.sample_row()
        summary = {
            "schema_version": 3,
            "results": [row],
            "history": {"sample-suite": [row]},
        }
        with tempfile.TemporaryDirectory() as temporary_directory:
            output = Path(temporary_directory)
            tracking.build_pages_site(output, summary, row["source_sha"], None)
            dashboard = (output / "index.html").read_text(encoding="utf-8")
            suite = (output / "benchmarks/sample-suite.html").read_text(encoding="utf-8")
            docs = (output / "docs/index.html").read_text(encoding="utf-8")
            self.assertTrue((output / "assets/site.css").is_file())
            missing_targets = []
            for page in output.rglob("*.html"):
                page_html = page.read_text(encoding="utf-8")
                for target in re.findall(r'(?:href|src)="([^"#?]+)', page_html):
                    if "://" not in target and not target.startswith(("mailto:", "/")):
                        if not (page.parent / target).resolve().exists():
                            missing_targets.append((page, target))
        self.assertIn("At parity", dashboard)
        self.assertIn("p50", dashboard)
        self.assertIn("data-performance-chart", suite)
        self.assertIn("wrf-rs", docs)
        self.assertIn("wiki/System-Overview.html", docs)
        self.assertNotIn("\x00", docs)
        self.assertEqual(missing_targets, [])


if __name__ == "__main__":
    unittest.main()
