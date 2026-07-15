#!/usr/bin/env python3
"""Validate, run, aggregate, and publish wrf-rs tracking evidence and docs."""

from __future__ import annotations

import argparse
import datetime as dt
import fnmatch
import html
import json
import os
import platform
import re
import shutil
import statistics
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
TRACKING = ROOT / "tracking"
GENERATED = ROOT / "docs" / "generated"
FLOAT = r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[EeDd][-+]?\d+)?"
SAMPLE_LINE = re.compile(
    rf"^(?:(?P<case>[A-Za-z0-9_-]+)_)?sample_\d+_milliseconds_per_call\s+(?P<value>{FLOAT})$"
)
NAMED_LINE = re.compile(rf"^(?P<case>[A-Za-z][A-Za-z0-9_-]*)\s+(?P<value>{FLOAT})$")
NUMBER_LINE = re.compile(rf"^(?P<value>{FLOAT})$")


class TrackingError(RuntimeError):
    """Raised when tracking evidence is incomplete or malformed."""


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as stream:
        value = json.load(stream)
    if not isinstance(value, dict):
        raise TrackingError(f"{path.relative_to(ROOT)} must contain a JSON object")
    return value


def load_benchmarks() -> dict[str, Any]:
    return load_json(TRACKING / "benchmarks.json")


def load_status() -> dict[str, Any]:
    return load_json(TRACKING / "port-status.json")


def benchmark_by_id(identifier: str) -> dict[str, Any]:
    for suite in load_benchmarks()["suites"]:
        if suite["id"] == identifier:
            return suite
    raise TrackingError(f"unknown benchmark suite: {identifier}")


def validate() -> None:
    benchmarks = load_benchmarks()
    status = load_status()
    errors: list[str] = []

    if benchmarks.get("schema_version") != 1:
        errors.append("tracking/benchmarks.json has an unsupported schema_version")
    if not benchmarks.get("ci_runner_class"):
        errors.append("tracking/benchmarks.json must define ci_runner_class")
    if status.get("schema_version") != 1:
        errors.append("tracking/port-status.json has an unsupported schema_version")

    suites = benchmarks.get("suites")
    if not isinstance(suites, list) or not suites:
        errors.append("tracking/benchmarks.json must define a non-empty suites array")
        suites = []

    required = {
        "id",
        "label",
        "package",
        "rust_bench",
        "criterion_group",
        "rust_case",
        "fortran_case",
        "fortran_script",
        "parity_oracle",
        "watch",
    }
    identifiers: set[str] = set()
    for index, suite in enumerate(suites):
        missing = sorted(required - suite.keys())
        if missing:
            errors.append(f"benchmark suite {index} is missing: {', '.join(missing)}")
            continue
        identifier = suite["id"]
        if identifier in identifiers:
            errors.append(f"duplicate benchmark id: {identifier}")
        identifiers.add(identifier)
        for key in ("fortran_script", "parity_oracle"):
            path = ROOT / suite[key]
            if not path.is_file():
                errors.append(f"{identifier}: missing {key} {suite[key]}")
        bench_path = ROOT / "crates" / suite["package"] / "benches" / f"{suite['rust_bench']}.rs"
        if not bench_path.is_file():
            errors.append(f"{identifier}: missing Rust bench {bench_path.relative_to(ROOT)}")
        else:
            bench_source = bench_path.read_text(encoding="utf-8")
            for key in ("criterion_group", "rust_case"):
                if suite[key] not in bench_source:
                    errors.append(
                        f"{identifier}: {key} {suite[key]!r} is absent from "
                        f"{bench_path.relative_to(ROOT)}"
                    )
        if not isinstance(suite["watch"], list) or not suite["watch"]:
            errors.append(f"{identifier}: watch must be a non-empty array")

    catalogued_fortran = {suite["fortran_script"] for suite in suites if "fortran_script" in suite}
    discovered_fortran = {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "scripts").glob("benchmark-*-fortran.sh")
    }
    for path in sorted(discovered_fortran - catalogued_fortran):
        errors.append(f"uncatalogued Fortran benchmark: {path}")
    for path in sorted(catalogued_fortran - discovered_fortran):
        errors.append(f"catalog references a non-benchmark Fortran script: {path}")

    catalogued_rust = {
        f"crates/{suite['package']}/benches/{suite['rust_bench']}.rs"
        for suite in suites
        if "package" in suite and "rust_bench" in suite
    }
    discovered_rust = {
        path.relative_to(ROOT).as_posix()
        for path in (ROOT / "crates").glob("*/benches/*.rs")
    }
    for path in sorted(discovered_rust - catalogued_rust):
        errors.append(f"uncatalogued Rust benchmark: {path}")

    areas = status.get("areas")
    if not isinstance(areas, list) or not areas:
        errors.append("tracking/port-status.json must define a non-empty areas array")
    else:
        area_ids = [area.get("id") for area in areas]
        if len(area_ids) != len(set(area_ids)):
            errors.append("tracking/port-status.json contains duplicate area ids")
        for area in areas:
            for key in ("id", "area", "state", "evidence", "next_gate"):
                if not area.get(key):
                    errors.append(f"port area {area.get('id', '<unknown>')} is missing {key}")

    if errors:
        raise TrackingError("tracking validation failed:\n- " + "\n- ".join(errors))


def state_label(state: str) -> str:
    return state.replace("-", " ").capitalize()


def render_port_status(status: dict[str, Any]) -> str:
    target = status["target"]
    whole = status["whole_model_parity"]
    lines = [
        "<!-- Generated by tools/tracking.py; do not edit. -->",
        "# Port status matrix",
        "",
        f"Target: {target['name']} {target['version']} at `{target['commit']}`.",
        "",
        f"Whole-model parity: **{whole['percent']}% ({state_label(whole['state'])})**. "
        f"{whole['definition']}",
        "",
        "| Area | State | Evidence | Next gate | Tracking |",
        "|---|---|---|---|---|",
    ]
    for area in status["areas"]:
        issue = area.get("tracking_issue")
        tracking = f"[#{issue}](https://github.com/mikeortman/wrf-rs/issues/{issue})" if issue else "—"
        lines.append(
            f"| {area['area']} | {state_label(area['state'])} | {area['evidence']} | "
            f"{area['next_gate']} | {tracking} |"
        )
    lines.extend(
        [
            "",
            "This matrix describes accepted interfaces and evidence, not a percentage extrapolated from file counts.",
            "Mutable work state and sequencing live in GitHub Issues and the WRF Rust Port Project.",
            "",
        ]
    )
    return "\n".join(lines)


def render_benchmark_catalog(benchmarks: dict[str, Any]) -> str:
    lines = [
        "<!-- Generated by tools/tracking.py; do not edit. -->",
        "# Matched benchmark catalog",
        "",
        f"CI runner classification: **{benchmarks['ci_runner_class'].replace('-', ' ')}**.",
        "",
        "| Suite | Rust Criterion case | Optimized Fortran | Parity gate |",
        "|---|---|---|---|",
    ]
    for suite in benchmarks["suites"]:
        rust = f"`{suite['package']}::{suite['rust_bench']}::{suite['rust_case']}`"
        lines.append(
            f"| {suite['label']} | {rust} | `{suite['fortran_script']}` ({suite['fortran_case']}) | "
            f"`{suite['parity_oracle']}` |"
        )
    lines.extend(
        [
            "",
            "The catalog defines commands and change routing. Measurements are produced after successful parity on main.",
            "",
        ]
    )
    return "\n".join(lines)


def generated_files() -> dict[Path, str]:
    return {
        GENERATED / "port-status.md": render_port_status(load_status()),
        GENERATED / "benchmark-catalog.md": render_benchmark_catalog(load_benchmarks()),
    }


def render() -> None:
    validate()
    GENERATED.mkdir(parents=True, exist_ok=True)
    for path, content in generated_files().items():
        path.write_text(content, encoding="utf-8")


def check() -> None:
    validate()
    drifted = []
    for path, expected in generated_files().items():
        if not path.is_file() or path.read_text(encoding="utf-8") != expected:
            drifted.append(path.relative_to(ROOT).as_posix())
    if drifted:
        raise TrackingError(
            "generated tracking files are stale: "
            + ", ".join(drifted)
            + "; run python3 tools/tracking.py render"
        )


def path_matches(path: str, patterns: Iterable[str]) -> bool:
    return any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)


def changed_paths(base: str, head: str) -> list[str]:
    completed = subprocess.run(
        ["git", "diff", "--name-only", base, head],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return [line for line in completed.stdout.splitlines() if line]


def load_benchmarks_at_revision(revision: str) -> dict[str, Any] | None:
    completed = subprocess.run(
        ["git", "show", f"{revision}:tracking/benchmarks.json"],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        return None
    try:
        catalog = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise TrackingError(
            f"tracking/benchmarks.json at {revision} is not valid JSON: {error}"
        ) from error
    if not isinstance(catalog, dict):
        raise TrackingError(
            f"tracking/benchmarks.json at {revision} must contain a JSON object"
        )
    return catalog


def load_toml_at_revision(revision: str, path: str) -> dict[str, Any] | None:
    """Load a TOML object from Git, returning ``None`` when it is unavailable."""
    completed = subprocess.run(
        ["git", "show", f"{revision}:{path}"],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        return None
    try:
        return tomllib.loads(completed.stdout)
    except tomllib.TOMLDecodeError:
        return None


def changed_catalog_suite_ids(
    previous: dict[str, Any] | None,
    current: dict[str, Any],
) -> set[str] | None:
    """Return changed current suite ids, or ``None`` when every suite is safer."""
    if previous is None:
        return None
    previous_top_level = {key: value for key, value in previous.items() if key != "suites"}
    current_top_level = {key: value for key, value in current.items() if key != "suites"}
    if previous_top_level != current_top_level and not is_scoped_routing_watch_migration(
        previous_top_level, current_top_level
    ):
        return None

    previous_suites = previous.get("suites")
    current_suites = current.get("suites")
    if not isinstance(previous_suites, list) or not isinstance(current_suites, list):
        return None
    previous_by_id = {
        suite.get("id"): suite
        for suite in previous_suites
        if isinstance(suite, dict) and isinstance(suite.get("id"), str)
    }
    current_by_id = {
        suite.get("id"): suite
        for suite in current_suites
        if isinstance(suite, dict) and isinstance(suite.get("id"), str)
    }
    if len(previous_by_id) != len(previous_suites) or len(current_by_id) != len(current_suites):
        return None

    changed_current_ids = {
        identifier
        for identifier, suite in current_by_id.items()
        if previous_by_id.get(identifier) != suite
    }
    removed_ids = previous_by_id.keys() - current_by_id.keys()
    order_changed = [suite["id"] for suite in previous_suites] != [
        suite["id"] for suite in current_suites
    ]
    if (removed_ids or order_changed) and not changed_current_ids:
        return None
    return changed_current_ids or None


def is_scoped_routing_watch_migration(
    previous: dict[str, Any], current: dict[str, Any]
) -> bool:
    """Recognize moving catalog and router paths out of unconditional routing."""
    previous_without_watch = {
        key: value for key, value in previous.items() if key != "global_watch"
    }
    current_without_watch = {
        key: value for key, value in current.items() if key != "global_watch"
    }
    if previous_without_watch != current_without_watch:
        return False
    previous_watch = previous.get("global_watch")
    current_watch = current.get("global_watch")
    if not isinstance(previous_watch, list) or not isinstance(current_watch, list):
        return False
    migrated_paths = {"tools/tracking.py", "tracking/benchmarks.json"}
    removed_paths = [path for path in previous_watch if path not in current_watch]
    return (
        [path for path in previous_watch if path not in migrated_paths] == current_watch
        and set(removed_paths) == migrated_paths
        and len(removed_paths) == len(migrated_paths)
    )


def added_suite_package(
    previous: dict[str, Any] | None,
    current: dict[str, Any],
) -> str | None:
    """Return the package when the catalog only adds one new suite and package."""
    if previous is None:
        return None
    previous_suites = previous.get("suites")
    current_suites = current.get("suites")
    if not isinstance(previous_suites, list) or not isinstance(current_suites, list):
        return None
    if len(current_suites) != len(previous_suites) + 1:
        return None
    if not all(isinstance(suite, dict) for suite in previous_suites + current_suites):
        return None
    if any(
        not isinstance(suite.get("id"), str)
        or not isinstance(suite.get("package"), str)
        for suite in previous_suites + current_suites
    ):
        return None

    previous_by_id = {suite.get("id"): suite for suite in previous_suites}
    current_by_id = {suite.get("id"): suite for suite in current_suites}
    if len(previous_by_id) != len(previous_suites) or len(current_by_id) != len(current_suites):
        return None
    if any(current_by_id.get(identifier) != suite for identifier, suite in previous_by_id.items()):
        return None

    added_ids = current_by_id.keys() - previous_by_id.keys()
    if len(added_ids) != 1:
        return None
    added_identifier = next(iter(added_ids))
    if [suite["id"] for suite in current_suites if suite["id"] != added_identifier] != [
        suite["id"] for suite in previous_suites
    ]:
        return None

    package = current_by_id[added_identifier].get("package")
    if not isinstance(package, str) or not package:
        return None
    previous_packages = {suite.get("package") for suite in previous_suites}
    return package if package not in previous_packages else None


def is_additive_workspace_member(
    previous: dict[str, Any] | None,
    current: dict[str, Any] | None,
    package: str,
) -> bool:
    """Return whether the root manifest only adds one package workspace member."""
    if previous is None or current is None:
        return False
    previous_workspace = previous.get("workspace")
    current_workspace = current.get("workspace")
    if not isinstance(previous_workspace, dict) or not isinstance(current_workspace, dict):
        return False
    previous_members = previous_workspace.get("members")
    current_members = current_workspace.get("members")
    if not isinstance(previous_members, list) or not isinstance(current_members, list):
        return False
    if not all(isinstance(member, str) for member in previous_members + current_members):
        return False

    member = f"crates/{package}"
    if member in previous_members or current_members.count(member) != 1:
        return False
    if [current_member for current_member in current_members if current_member != member] != previous_members:
        return False

    previous_without_members = {
        **previous,
        "workspace": {
            key: value for key, value in previous_workspace.items() if key != "members"
        },
    }
    current_without_members = {
        **current,
        "workspace": {
            key: value for key, value in current_workspace.items() if key != "members"
        },
    }
    return previous_without_members == current_without_members


def is_additive_lock_package(
    previous: dict[str, Any] | None,
    current: dict[str, Any] | None,
    package: str,
) -> bool:
    """Return whether the lockfile only adds one stanza for ``package``."""
    if previous is None or current is None:
        return False
    previous_packages = previous.get("package")
    current_packages = current.get("package")
    if not isinstance(previous_packages, list) or not isinstance(current_packages, list):
        return False
    if len(current_packages) != len(previous_packages) + 1:
        return False
    if not all(isinstance(entry, dict) for entry in previous_packages + current_packages):
        return False

    previous_without_packages = {
        key: value for key, value in previous.items() if key != "package"
    }
    current_without_packages = {
        key: value for key, value in current.items() if key != "package"
    }
    if previous_without_packages != current_without_packages:
        return False

    for index, entry in enumerate(current_packages):
        if entry.get("name") != package:
            continue
        if current_packages[:index] + current_packages[index + 1 :] == previous_packages:
            return not any(
                previous_entry.get("name") == package
                for previous_entry in previous_packages
            )
    return False


def scoped_workspace_bootstrap_paths(
    previous_benchmarks: dict[str, Any] | None,
    benchmarks: dict[str, Any],
    previous_workspace_manifest: dict[str, Any] | None,
    workspace_manifest: dict[str, Any] | None,
    previous_lockfile: dict[str, Any] | None,
    lockfile: dict[str, Any] | None,
) -> set[str]:
    """Return global paths proven to contain only one new benchmark package."""
    package = added_suite_package(previous_benchmarks, benchmarks)
    if package is None:
        return set()
    if not is_additive_workspace_member(
        previous_workspace_manifest, workspace_manifest, package
    ):
        return set()
    if not is_additive_lock_package(previous_lockfile, lockfile, package):
        return set()
    return {"Cargo.toml", "Cargo.lock"}


def select_suites(
    paths: list[str],
    run_all: bool = False,
    *,
    benchmarks: dict[str, Any] | None = None,
    previous_benchmarks: dict[str, Any] | None = None,
    previous_workspace_manifest: dict[str, Any] | None = None,
    workspace_manifest: dict[str, Any] | None = None,
    previous_lockfile: dict[str, Any] | None = None,
    lockfile: dict[str, Any] | None = None,
) -> list[dict[str, str]]:
    benchmarks = benchmarks or load_benchmarks()
    suites = benchmarks["suites"]
    scoped_global_paths = set()
    if {"Cargo.toml", "Cargo.lock"}.issubset(paths):
        scoped_global_paths = scoped_workspace_bootstrap_paths(
            previous_benchmarks,
            benchmarks,
            previous_workspace_manifest,
            workspace_manifest,
            previous_lockfile,
            lockfile,
        )
    global_change = any(
        path not in scoped_global_paths
        and path_matches(path, benchmarks["global_watch"])
        for path in paths
    )
    if run_all or global_change:
        selected = suites
    else:
        catalog_changed = "tracking/benchmarks.json" in paths
        routing_code_changed = "tools/tracking.py" in paths
        selected_ids = {
            suite["id"]
            for suite in suites
            if any(path_matches(path, suite["watch"]) for path in paths)
        }
        if catalog_changed:
            catalog_ids = changed_catalog_suite_ids(previous_benchmarks, benchmarks)
            if catalog_ids is None:
                selected = suites
            else:
                selected_ids.update(catalog_ids)
                selected = [suite for suite in suites if suite["id"] in selected_ids]
        elif routing_code_changed:
            selected = suites
        else:
            selected = [suite for suite in suites if suite["id"] in selected_ids]
    return [{"id": suite["id"], "label": suite["label"]} for suite in selected]


def command_output(command: list[str]) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT,
            check=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
    except subprocess.CalledProcessError as error:
        output = error.stdout or ""
        raise TrackingError(
            f"command failed with status {error.returncode}: {' '.join(command)}\n{output}"
        ) from error
    return completed.stdout


def version_line(command: list[str]) -> str:
    try:
        return command_output(command).splitlines()[0]
    except (TrackingError, IndexError):
        return "unavailable"


def parse_fortran_samples(output: str) -> dict[str, list[float]]:
    samples: dict[str, list[float]] = {}
    for raw_line in output.splitlines():
        line = raw_line.strip()
        match = SAMPLE_LINE.match(line)
        if match:
            case = (match.group("case") or "default").rstrip("_")
        else:
            match = NAMED_LINE.match(line)
            if match and match.group("case").lower() not in {
                "checksum",
                "compiler",
                "flags",
                "grid_points_per_call",
                "momentum_updates_per_call",
                "momentum_outputs_per_call",
                "momentum_mass_outputs_per_call",
                "coefficient_outputs_per_call",
                "omega_outputs_per_call",
                "inverse_density_outputs_per_call",
                "pressure_point_outputs_per_call",
                "mass_points_per_call",
            }:
                case = match.group("case")
            else:
                match = NUMBER_LINE.match(line)
                if not match:
                    continue
                case = "default"
        value = float(match.group("value").replace("D", "E").replace("d", "e"))
        samples.setdefault(case, []).append(value)
    return samples


def criterion_estimates(suite: dict[str, Any]) -> list[dict[str, Any]]:
    target_root = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    pattern = f"criterion/{suite['criterion_group']}/{suite['rust_case']}/*/new/estimates.json"
    estimates = []
    for path in sorted(target_root.glob(pattern)):
        try:
            workers = int(path.parents[1].name)
        except ValueError as error:
            raise TrackingError(f"Criterion worker directory is not numeric: {path}") from error
        value = load_json(path)
        median = value["median"]
        interval = median["confidence_interval"]
        sample_path = path.with_name("sample.json")
        sample = load_json(sample_path)
        iterations = sample.get("iters")
        times = sample.get("times")
        if (
            not isinstance(iterations, list)
            or not isinstance(times, list)
            or len(iterations) != len(times)
            or not iterations
            or any(not isinstance(iteration, (int, float)) or iteration <= 0 for iteration in iterations)
            or any(not isinstance(time, (int, float)) or time < 0 for time in times)
        ):
            raise TrackingError(f"malformed Criterion samples: {sample_path}")
        nanoseconds_per_iteration = [
            float(time) / float(iteration)
            for iteration, time in zip(iterations, times, strict=True)
        ]
        estimates.append(
            {
                "workers": workers,
                "median_nanoseconds": median["point_estimate"],
                "lower_nanoseconds": interval["lower_bound"],
                "upper_nanoseconds": interval["upper_bound"],
                "percentiles_nanoseconds": sample_percentiles(
                    nanoseconds_per_iteration
                ),
            }
        )
    if not estimates:
        raise TrackingError(
            f"no Criterion estimates found for {suite['criterion_group']}/{suite['rust_case']}"
        )
    return sorted(estimates, key=lambda estimate: estimate["workers"])


def run_benchmark(identifier: str, output_directory: Path) -> Path:
    suite = benchmark_by_id(identifier)
    output_directory.mkdir(parents=True, exist_ok=True)

    fortran_command = [str(ROOT / suite["fortran_script"])]
    fortran_output = command_output(fortran_command)
    (output_directory / f"{identifier}-fortran.log").write_text(fortran_output, encoding="utf-8")
    parsed = parse_fortran_samples(fortran_output)
    fortran_samples = parsed.get(suite["fortran_case"], [])
    if not fortran_samples:
        available = ", ".join(sorted(parsed)) or "none"
        raise TrackingError(
            f"{identifier}: no Fortran samples for case {suite['fortran_case']}; available: {available}"
        )

    rust_command = [
        "cargo",
        "bench",
        "-p",
        suite["package"],
        "--bench",
        suite["rust_bench"],
        "--",
        suite["rust_case"],
        "--noplot",
        "--sample-size",
        "300",
        "--measurement-time",
        "10",
    ]
    rust_output = command_output(rust_command)
    (output_directory / f"{identifier}-rust.log").write_text(rust_output, encoding="utf-8")
    estimates = criterion_estimates(suite)

    result = {
        "schema_version": 1,
        "suite": {key: suite[key] for key in ("id", "label", "package", "rust_bench", "rust_case", "fortran_script", "fortran_case", "parity_oracle")},
        "source_sha": version_line(["git", "rev-parse", "HEAD"]),
        "collected_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "runner": {
            "classification": os.environ.get(
                "WRF_BENCHMARK_RUNNER_CLASS", "developer-machine"
            ),
            "platform": platform.platform(),
            "machine": platform.machine(),
            "logical_cpus": os.cpu_count(),
            "rust": version_line(["rustc", "--version"]),
            "fortran": version_line(["gfortran", "--version"]),
        },
        "fortran": {
            "samples_milliseconds": fortran_samples,
            "median_milliseconds": statistics.median(fortran_samples),
            "minimum_milliseconds": min(fortran_samples),
            "maximum_milliseconds": max(fortran_samples),
            "percentiles_milliseconds": sample_percentiles(fortran_samples),
        },
        "rust": {"estimates": estimates},
    }
    output_path = output_directory / f"{identifier}.json"
    output_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return output_path


def load_results(directory: Path) -> list[dict[str, Any]]:
    results = []
    for path in directory.rglob("*.json"):
        value = load_json(path)
        if value.get("schema_version") == 1 and "suite" in value and "fortran" in value and "rust" in value:
            results.append(value)
    if not results:
        raise TrackingError(f"no benchmark result JSON found under {directory}")
    return sorted(results, key=lambda result: result["suite"]["label"])


def milliseconds(nanoseconds: float) -> float:
    return nanoseconds / 1_000_000.0


def percentile(samples: list[float], probability: float) -> float:
    """Return a linearly interpolated percentile for a non-empty sample."""

    if not samples:
        raise TrackingError("cannot calculate a percentile from an empty sample")
    if not 0.0 <= probability <= 1.0:
        raise TrackingError(f"percentile probability is outside [0, 1]: {probability}")
    ordered = sorted(float(sample) for sample in samples)
    position = (len(ordered) - 1) * probability
    lower_index = int(position)
    upper_index = min(lower_index + 1, len(ordered) - 1)
    fraction = position - lower_index
    return ordered[lower_index] + (ordered[upper_index] - ordered[lower_index]) * fraction


def sample_percentiles(samples: list[float]) -> dict[str, float]:
    """Return the latency percentiles published by the benchmark dashboard."""

    return {
        "p50": percentile(samples, 0.50),
        "p90": percentile(samples, 0.90),
        "p99": percentile(samples, 0.99),
    }


def percentile_milliseconds(percentiles_nanoseconds: dict[str, float]) -> dict[str, float]:
    return {
        name: milliseconds(value)
        for name, value in percentiles_nanoseconds.items()
    }


def relative_text(rust_ms: float, fortran_ms: float) -> str:
    ratio = rust_ms / fortran_ms
    if 0.98 <= ratio <= 1.02:
        return f"effectively tied ({(ratio - 1.0) * 100:+.1f}%)"
    if ratio < 1.0:
        return f"Rust {1.0 / ratio:.2f}x faster"
    return f"Rust {ratio:.2f}x slower"


def result_rows(
    results: list[dict[str, Any]], run_url: str | None = None
) -> list[dict[str, Any]]:
    rows = []
    for result in results:
        estimates = result["rust"]["estimates"]
        serial = next((estimate for estimate in estimates if estimate["workers"] == 1), None)
        if serial is None:
            raise TrackingError(f"{result['suite']['id']} has no one-worker Rust estimate")
        best = min(estimates, key=lambda estimate: estimate["median_nanoseconds"])
        fortran_ms = result["fortran"]["median_milliseconds"]
        fortran_percentiles = result["fortran"].get("percentiles_milliseconds")
        if fortran_percentiles is None:
            fortran_percentiles = sample_percentiles(
                result["fortran"]["samples_milliseconds"]
            )
        serial_percentiles = percentile_milliseconds(
            serial["percentiles_nanoseconds"]
        )
        best_percentiles = percentile_milliseconds(best["percentiles_nanoseconds"])
        serial_ms = milliseconds(serial["median_nanoseconds"])
        best_ms = milliseconds(best["median_nanoseconds"])
        if fortran_percentiles["p50"] <= 0:
            raise TrackingError(
                f"{result['suite']['id']} has a non-positive Fortran p50"
            )
        best_ratio = best_percentiles["p50"] / fortran_percentiles["p50"]
        rows.append(
            {
                "id": result["suite"]["id"],
                "label": result["suite"]["label"],
                "fortran_milliseconds": fortran_ms,
                "rust_serial_milliseconds": serial_ms,
                "rust_best_milliseconds": best_ms,
                "rust_best_workers": best["workers"],
                "fortran_percentiles_milliseconds": fortran_percentiles,
                "rust_serial_percentiles_milliseconds": serial_percentiles,
                "rust_best_percentiles_milliseconds": best_percentiles,
                "best_ratio": best_ratio,
                "performance_status": "passing" if best_ratio <= 1.02 else "behind",
                "serial_relative": relative_text(serial_ms, fortran_ms),
                "best_relative": relative_text(best_ms, fortran_ms),
                "source_sha": result["source_sha"],
                "collected_at": result["collected_at"],
                "runner": result["runner"],
                "run_url": run_url,
            }
        )
    return rows


def merge_result_rows(
    previous_rows: list[dict[str, Any]], current_rows: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    merged = {row["id"]: row for row in previous_rows}
    merged.update({row["id"]: row for row in current_rows})
    return sorted(merged.values(), key=lambda row: row["label"])


def merge_result_history(
    previous_history: dict[str, list[dict[str, Any]]],
    current_rows: list[dict[str, Any]],
) -> dict[str, list[dict[str, Any]]]:
    """Append new suite observations without duplicating a published run."""

    merged = {
        identifier: list(observations)
        for identifier, observations in previous_history.items()
    }
    for row in current_rows:
        observations = merged.setdefault(row["id"], [])
        observation_key = (row.get("source_sha"), row.get("collected_at"))
        if not any(
            (item.get("source_sha"), item.get("collected_at")) == observation_key
            for item in observations
        ):
            observations.append(row)
    return {identifier: merged[identifier] for identifier in sorted(merged)}


def load_previous_summary(path: Path | None) -> dict[str, Any]:
    if path is None or not path.is_file():
        return {"results": [], "history": {}}
    summary = load_json(path)
    if summary.get("schema_version") not in {2, 3} or not isinstance(
        summary.get("results"), list
    ):
        raise TrackingError(f"unsupported previous performance summary: {path}")
    if summary["schema_version"] == 2:
        history = {row["id"]: [row] for row in summary["results"]}
    else:
        history = summary.get("history")
        if not isinstance(history, dict) or any(
            not isinstance(identifier, str) or not isinstance(observations, list)
            for identifier, observations in history.items()
        ):
            raise TrackingError(f"malformed performance history: {path}")
    return {"results": summary["results"], "history": history}


def render_result_markdown(
    rows: list[dict[str, Any]],
    heading: str,
    current_sha: str,
    run_url: str | None,
    receipt: bool = False,
) -> str:
    lines = [
        "<!-- wrf-rs-performance-receipt -->" if receipt else "<!-- wrf-rs-performance-matrix -->",
        f"## {heading}",
        "",
        f"Latest merge: `{current_sha}`.",
        "",
        "| Suite | Fortran median | Rust, 1 worker | Best Rust | Comparison | Updated |",
        "|---|---:|---:|---:|---|---|",
    ]
    for row in rows:
        short_sha = row["source_sha"][:8]
        source_url = f"https://github.com/mikeortman/wrf-rs/commit/{row['source_sha']}"
        updated = f"[{short_sha}]({source_url})"
        lines.append(
            f"| {row['label']} | {row['fortran_milliseconds']:.6g} ms | "
            f"{row['rust_serial_milliseconds']:.6g} ms | {row['rust_best_milliseconds']:.6g} ms "
            f"({row['rust_best_workers']} workers) | Serial: {row['serial_relative']}; "
            f"best: {row['best_relative']} | {updated} |"
        )
    lines.extend(
        [
            "",
            "These are same-runner Rust/Fortran comparisons. Runner state varies, so small cross-run changes are noise, not a regression verdict.",
        ]
    )
    if run_url:
        lines.append(f"Raw logs, JSON, and workflow summary: {run_url}")
    lines.append("")
    return "\n".join(lines)


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or "section"


def document_title(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return path.stem.replace("-", " ").replace("_", " ").title()


def document_sources() -> list[Path]:
    root_documents = [
        ROOT / "README.md",
        ROOT / "CURRENT_STATE.md",
        ROOT / "TEST_COVERAGE.md",
        ROOT / "PERFORMANCE_PARITY.md",
        ROOT / "UPSTREAM_FINDINGS.md",
        ROOT / "tracking" / "README.md",
    ]
    return [path for path in root_documents if path.is_file()] + sorted(
        (ROOT / "docs").rglob("*.md")
    )


def document_output_path(source: Path) -> Path:
    relative = source.relative_to(ROOT)
    if relative == Path("README.md"):
        return Path("docs/index.html")
    if relative == Path("tracking/README.md"):
        return Path("docs/tracking/index.html")
    if relative.parts[0] == "docs":
        within_docs = Path(*relative.parts[1:])
        if within_docs.name == "README.md":
            return Path("docs") / within_docs.parent / "index.html"
        return Path("docs") / within_docs.with_suffix(".html")
    return Path("docs/project") / f"{slugify(relative.stem)}.html"


def rewrite_document_link(
    target: str,
    source: Path,
    current_output: Path,
    outputs_by_source: dict[Path, Path],
) -> str:
    if target.startswith(("#", "mailto:", "http://", "https://")):
        github_prefix = "https://github.com/mikeortman/wrf-rs/blob/main/"
        if not target.startswith(github_prefix):
            return target
        target = target.removeprefix(github_prefix)
        candidate = ROOT / target.split("#", 1)[0]
    else:
        path_part = target.split("#", 1)[0]
        candidate = (source.parent / path_part).resolve()
    fragment = target.split("#", 1)[1] if "#" in target else ""
    destination = outputs_by_source.get(candidate)
    if destination is None:
        try:
            repository_relative = candidate.relative_to(ROOT)
        except ValueError:
            return target
        if candidate.exists():
            suffix = f"#{fragment}" if fragment else ""
            return (
                "https://github.com/mikeortman/wrf-rs/blob/main/"
                f"{repository_relative.as_posix()}{suffix}"
            )
        return target
    relative_url = os.path.relpath(destination, current_output.parent).replace(os.sep, "/")
    return f"{relative_url}#{fragment}" if fragment else relative_url


def render_inline_markdown(
    text: str,
    source: Path,
    current_output: Path,
    outputs_by_source: dict[Path, Path],
) -> str:
    replacements: list[str] = []

    def placeholder(value: str) -> str:
        replacements.append(value)
        return f"\x00{len(replacements) - 1}\x00"

    text = re.sub(
        r"`([^`]+)`",
        lambda match: placeholder(f"<code>{html.escape(match.group(1))}</code>"),
        text,
    )
    text = re.sub(
        r"<(https?://[^>]+)>",
        lambda match: placeholder(
            f'<a href="{html.escape(match.group(1), quote=True)}">{html.escape(match.group(1))}</a>'
        ),
        text,
    )

    def replace_image(match: re.Match[str]) -> str:
        alt, target = match.group(1), match.group(2)
        rewritten = rewrite_document_link(target, source, current_output, outputs_by_source)
        return placeholder(
            f'<img src="{html.escape(rewritten, quote=True)}" alt="{html.escape(alt, quote=True)}">'
        )

    text = re.sub(r"!\[([^]]*)\]\(([^)]+)\)", replace_image, text)

    def replace_link(match: re.Match[str]) -> str:
        label, target = match.group(1), match.group(2)
        rewritten = rewrite_document_link(target, source, current_output, outputs_by_source)
        return placeholder(
            f'<a href="{html.escape(rewritten, quote=True)}">{html.escape(label)}</a>'
        )

    text = re.sub(r"\[([^]]+)\]\(([^)]+)\)", replace_link, text)
    rendered = html.escape(text)
    rendered = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", rendered)
    rendered = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", rendered)
    for index in reversed(range(len(replacements))):
        replacement = replacements[index]
        rendered = rendered.replace(f"\x00{index}\x00", replacement)
    return rendered


def is_table_separator(line: str) -> bool:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells)


def render_markdown_document(
    source: Path,
    current_output: Path,
    outputs_by_source: dict[Path, Path],
) -> str:
    lines = source.read_text(encoding="utf-8").splitlines()
    rendered: list[str] = []
    paragraph: list[str] = []
    list_tag: str | None = None
    index = 0

    def inline(value: str) -> str:
        return render_inline_markdown(value, source, current_output, outputs_by_source)

    def flush_paragraph() -> None:
        if paragraph:
            rendered.append(f"<p>{inline(' '.join(part.strip() for part in paragraph))}</p>")
            paragraph.clear()

    def close_list() -> None:
        nonlocal list_tag
        if list_tag:
            rendered.append(f"</{list_tag}>")
            list_tag = None

    while index < len(lines):
        line = lines[index]
        stripped = line.strip()
        if stripped.startswith("```"):
            flush_paragraph()
            close_list()
            language = stripped[3:].strip()
            code_lines: list[str] = []
            index += 1
            while index < len(lines) and not lines[index].strip().startswith("```"):
                code_lines.append(lines[index])
                index += 1
            language_class = f' class="language-{html.escape(language, quote=True)}"' if language else ""
            rendered.append(
                f"<pre><code{language_class}>{html.escape(chr(10).join(code_lines))}</code></pre>"
            )
        elif not stripped:
            flush_paragraph()
            close_list()
        elif stripped.startswith("<!--"):
            flush_paragraph()
            close_list()
        elif index + 1 < len(lines) and "|" in line and is_table_separator(lines[index + 1]):
            flush_paragraph()
            close_list()
            headers = [cell.strip() for cell in line.strip().strip("|").split("|")]
            index += 2
            body_rows: list[list[str]] = []
            while index < len(lines) and "|" in lines[index] and lines[index].strip():
                body_rows.append(
                    [cell.strip() for cell in lines[index].strip().strip("|").split("|")]
                )
                index += 1
            index -= 1
            header_html = "".join(f"<th>{inline(cell)}</th>" for cell in headers)
            body_html = "".join(
                "<tr>" + "".join(f"<td>{inline(cell)}</td>" for cell in row) + "</tr>"
                for row in body_rows
            )
            rendered.append(f"<table><thead><tr>{header_html}</tr></thead><tbody>{body_html}</tbody></table>")
        elif match := re.match(r"^(#{1,6})\s+(.+)$", stripped):
            flush_paragraph()
            close_list()
            level = len(match.group(1))
            heading = match.group(2).rstrip("#").strip()
            rendered.append(f'<h{level} id="{slugify(heading)}">{inline(heading)}</h{level}>')
        elif re.fullmatch(r"(?:-{3,}|\*{3,})", stripped):
            flush_paragraph()
            close_list()
            rendered.append("<hr>")
        elif stripped.startswith(">"):
            flush_paragraph()
            close_list()
            quote_lines = []
            while index < len(lines) and lines[index].strip().startswith(">"):
                quote_lines.append(lines[index].strip().lstrip(">").strip())
                index += 1
            index -= 1
            rendered.append(f"<blockquote><p>{inline(' '.join(quote_lines))}</p></blockquote>")
        elif match := re.match(r"^\s*([-+*])\s+(.+)$", line):
            flush_paragraph()
            if list_tag != "ul":
                close_list()
                rendered.append("<ul>")
                list_tag = "ul"
            item_parts = [match.group(2)]
            while (
                index + 1 < len(lines)
                and lines[index + 1].startswith(("  ", "\t"))
                and lines[index + 1].strip()
                and not re.match(r"^\s*[-+*]\s+", lines[index + 1])
            ):
                index += 1
                item_parts.append(lines[index].strip())
            rendered.append(f"<li>{inline(' '.join(item_parts))}</li>")
        elif match := re.match(r"^\s*\d+[.)]\s+(.+)$", line):
            flush_paragraph()
            if list_tag != "ol":
                close_list()
                rendered.append("<ol>")
                list_tag = "ol"
            item_parts = [match.group(1)]
            while (
                index + 1 < len(lines)
                and lines[index + 1].startswith(("  ", "\t"))
                and lines[index + 1].strip()
                and not re.match(r"^\s*\d+[.)]\s+", lines[index + 1])
            ):
                index += 1
                item_parts.append(lines[index].strip())
            rendered.append(f"<li>{inline(' '.join(item_parts))}</li>")
        else:
            close_list()
            paragraph.append(stripped)
        index += 1
    flush_paragraph()
    close_list()
    return "\n".join(rendered)


def root_prefix(current_output: Path) -> str:
    relative = os.path.relpath(Path("."), current_output.parent).replace(os.sep, "/")
    return "" if relative == "." else f"{relative}/"


def page_template(
    title: str,
    current_output: Path,
    current_section: str,
    body: str,
) -> str:
    prefix = root_prefix(current_output)
    benchmark_current = ' aria-current="page"' if current_section == "benchmarks" else ""
    docs_current = ' aria-current="page"' if current_section == "docs" else ""
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="description" content="wrf-rs numerical parity, performance history, and technical documentation.">
<meta property="og:type" content="website"><meta property="og:site_name" content="wrf-rs"><meta property="og:title" content="{html.escape(title, quote=True)} · wrf-rs">
<meta property="og:description" content="Numerical parity, performance history, and technical documentation for the Rust WRF port.">
<meta name="theme-color" content="#071018"><title>{html.escape(title)} · wrf-rs</title>
<link rel="stylesheet" href="{prefix}assets/site.css"></head><body>
<header class="site-header"><nav class="nav-shell" aria-label="Primary navigation">
<a class="brand" href="{prefix}index.html"><span class="brand-mark">WR</span><span>wrf-rs <small>Weather model port</small></span></a>
<button class="nav-toggle" type="button" data-nav-toggle aria-expanded="false" aria-label="Toggle navigation">Menu</button>
<div class="nav-links" data-nav-links><a href="{prefix}index.html"{benchmark_current}>Benchmarks</a><a href="{prefix}docs/index.html"{docs_current}>Documentation</a><a href="https://github.com/mikeortman/wrf-rs">GitHub</a></div>
</nav></header>{body}
<footer class="site-footer"><div class="nav-shell">Numerical claims are backed by matched Rust/Fortran evidence. Cross-run absolute timing is informational.</div></footer>
<script src="{prefix}assets/site.js"></script></body></html>"""


def format_duration(milliseconds_value: float | None) -> str:
    if milliseconds_value is None:
        return "—"
    if milliseconds_value >= 1.0:
        return f"{milliseconds_value:.3f} ms"
    if milliseconds_value >= 0.001:
        return f"{milliseconds_value * 1_000:.2f} µs"
    return f"{milliseconds_value * 1_000_000:.1f} ns"


def row_percentile(row: dict[str, Any], field: str, name: str) -> float | None:
    percentiles = row.get(field)
    if isinstance(percentiles, dict) and isinstance(percentiles.get(name), (int, float)):
        return float(percentiles[name])
    if name == "p50":
        fallback = {
            "fortran_percentiles_milliseconds": "fortran_milliseconds",
            "rust_serial_percentiles_milliseconds": "rust_serial_milliseconds",
            "rust_best_percentiles_milliseconds": "rust_best_milliseconds",
        }.get(field)
        value = row.get(fallback) if fallback else None
        return float(value) if isinstance(value, (int, float)) else None
    return None


def row_ratio(row: dict[str, Any], percentile_name: str = "p50") -> float | None:
    rust_value = row_percentile(row, "rust_best_percentiles_milliseconds", percentile_name)
    fortran_value = row_percentile(row, "fortran_percentiles_milliseconds", percentile_name)
    if rust_value is None or fortran_value in {None, 0.0}:
        return None
    return rust_value / fortran_value


def row_status(row: dict[str, Any]) -> str:
    ratio = row_ratio(row)
    return "passing" if ratio is not None and ratio <= 1.02 else "behind"


def percentile_cells(row: dict[str, Any], field: str) -> str:
    return '<div class="percentiles">' + "".join(
        f"<span>{name}<strong>{format_duration(row_percentile(row, field, name))}</strong></span>"
        for name in ("p50", "p90", "p99")
    ) + "</div>"


def render_dashboard(rows: list[dict[str, Any]], current_sha: str, run_url: str | None) -> str:
    passing = sum(row_status(row) == "passing" for row in rows)
    behind = len(rows) - passing
    collected = max((row.get("collected_at", "") for row in rows), default="")[:10] or "Awaiting data"
    runner = next((row.get("runner", {}).get("classification") for row in rows if row.get("runner")), "Awaiting data")
    table_rows = []
    for row in rows:
        status = row_status(row)
        ratio = row_ratio(row)
        ratio_text = f"{ratio:.2f}×" if ratio is not None else "—"
        relative = row.get("best_relative", "Awaiting percentile data")
        table_rows.append(f"""<tr data-benchmark-row data-label="{html.escape(row['label'].lower(), quote=True)}" data-status="{status}">
<td><a class="suite-link" href="benchmarks/{html.escape(row['id'], quote=True)}.html">{html.escape(row['label'])}</a><span class="suite-sub">{html.escape(relative)}</span></td>
<td><span class="status status-{status}">{'At parity' if status == 'passing' else 'Behind'}</span></td>
<td>{percentile_cells(row, 'fortran_percentiles_milliseconds')}</td>
<td>{percentile_cells(row, 'rust_best_percentiles_milliseconds')}</td>
<td><span class="ratio {'positive' if status == 'passing' else 'negative'}">{ratio_text}</span><span class="suite-sub">{row.get('rust_best_workers', '—')} workers</span></td>
</tr>""")
    evidence = f'<a href="{html.escape(run_url, quote=True)}">latest workflow evidence</a>' if run_url else "published evidence"
    body = f"""<main class="page-shell"><section class="hero"><div><p class="eyebrow">Validated against WRF v4.7.1</p>
<h1>Performance you can <span>interrogate.</span></h1><p class="lede">Matched Rust and optimized Fortran measurements, latency distributions, and durable history—published beside the documentation that explains every numerical contract.</p></div>
<div class="hero-meta"><p><strong>Source</strong> <a href="https://github.com/mikeortman/wrf-rs/commit/{html.escape(current_sha, quote=True)}" class="mono">{html.escape(current_sha[:8])}</a></p><p><strong>Runner</strong> {html.escape(str(runner))}</p><p><strong>Evidence</strong> {evidence}</p></div></section>
<section class="metrics" aria-label="Performance summary"><div class="metric"><span class="metric-label">At or ahead of parity</span><span class="metric-value positive">{passing}</span><span class="metric-detail">Best Rust p50 ≤ 1.02× Fortran</span></div>
<div class="metric"><span class="metric-label">Behind baseline</span><span class="metric-value {'negative' if behind else ''}">{behind}</span><span class="metric-detail">Optimization candidates</span></div>
<div class="metric"><span class="metric-label">Tracked suites</span><span class="metric-value">{len(rows)}</span><span class="metric-detail">Matched and parity-gated</span></div>
<div class="metric"><span class="metric-label">Latest measurement</span><span class="metric-value">{html.escape(collected)}</span><span class="metric-detail">History persists across deploys</span></div></section>
<section><div class="section-heading"><div><p class="eyebrow">Current matrix</p><h2>Rust versus optimized Fortran</h2></div><p>Green means the best Rust p50 is within 2% of—or faster than—the same-runner Fortran p50. Open a suite for its full p50/p90/p99 history.</p></div>
<div class="toolbar"><input class="control search" type="search" placeholder="Search benchmark suites" aria-label="Search benchmark suites" data-benchmark-search><select class="control" aria-label="Filter by performance status" data-status-filter><option value="all">All performance</option><option value="passing">At parity</option><option value="behind">Behind</option></select></div>
<div class="benchmark-table-wrap"><table class="benchmark-table"><thead><tr><th>Suite</th><th>Status</th><th>Fortran latency</th><th>Best Rust latency</th><th>p50 ratio</th></tr></thead><tbody>{''.join(table_rows)}</tbody></table><div class="empty-state" data-empty-state>No suites match this filter.</div></div></section></main>"""
    return page_template("Performance", Path("index.html"), "benchmarks", body)


def history_points(observations: list[dict[str, Any]]) -> list[dict[str, Any]]:
    points = []
    for observation in observations:
        point: dict[str, Any] = {"date": observation.get("collected_at", "")[:10] or "unknown"}
        for name in ("p50", "p90", "p99"):
            point[name] = row_ratio(observation, name)
        points.append(point)
    return points


def render_suite_page(row: dict[str, Any], observations: list[dict[str, Any]]) -> str:
    status = row_status(row)
    ratio = row_ratio(row)
    ratio_text = f"{ratio:.2f}× Fortran" if ratio is not None else "Awaiting data"
    points = html.escape(json.dumps(history_points(observations)), quote=True)
    history_rows = []
    for observation in reversed(observations):
        observation_ratio = row_ratio(observation)
        sha = observation.get("source_sha", "")
        history_rows.append(f"""<tr><td>{html.escape(observation.get('collected_at', '')[:10])}</td><td>{format_duration(row_percentile(observation, 'fortran_percentiles_milliseconds', 'p50'))}</td><td>{format_duration(row_percentile(observation, 'rust_best_percentiles_milliseconds', 'p50'))}</td><td>{f'{observation_ratio:.2f}×' if observation_ratio is not None else '—'}</td><td><a class="mono" href="https://github.com/mikeortman/wrf-rs/commit/{html.escape(sha, quote=True)}">{html.escape(sha[:8])}</a></td></tr>""")
    body = f"""<main class="page-shell"><p class="eyebrow"><a href="../index.html">Performance</a> / suite detail</p><section class="suite-hero"><div><h1>{html.escape(row['label'])}</h1><p class="lede">Distribution-aware comparison using matched optimized builds on the same runner.</p></div><div class="suite-status-card"><span class="status status-{status}">{'At parity' if status == 'passing' else 'Behind'}</span><strong class="{'positive' if status == 'passing' else 'negative'}">{ratio_text}</strong><span class="suite-sub">Best Rust p50 · {row.get('rust_best_workers', '—')} workers</span></div></section>
<section class="metrics"><div class="metric"><span class="metric-label">Fortran p50</span><span class="metric-value">{format_duration(row_percentile(row, 'fortran_percentiles_milliseconds', 'p50'))}</span><span class="metric-detail">Optimized serial baseline</span></div><div class="metric"><span class="metric-label">Rust p50</span><span class="metric-value">{format_duration(row_percentile(row, 'rust_best_percentiles_milliseconds', 'p50'))}</span><span class="metric-detail">Best worker count</span></div><div class="metric"><span class="metric-label">Rust p90</span><span class="metric-value">{format_duration(row_percentile(row, 'rust_best_percentiles_milliseconds', 'p90'))}</span><span class="metric-detail">Tail latency</span></div><div class="metric"><span class="metric-label">Rust p99</span><span class="metric-value">{format_duration(row_percentile(row, 'rust_best_percentiles_milliseconds', 'p99'))}</span><span class="metric-detail">Long-tail latency</span></div></section>
<section class="chart-card"><div class="chart-head"><div><p class="eyebrow">Historical ratio</p><h2>Rust latency relative to Fortran</h2></div><div class="chart-legend"><span class="legend-key" style="--legend-color:#56d6e7">p50</span><span class="legend-key" style="--legend-color:#f8c66a">p90</span><span class="legend-key" style="--legend-color:#ff7185">p99</span><span class="legend-key" style="--legend-color:#54dc93">parity</span></div></div><canvas class="performance-chart" data-performance-chart data-history="{points}" role="img" aria-label="Historical Rust to Fortran latency ratio"></canvas></section>
<section><div class="section-heading"><div><p class="eyebrow">Run ledger</p><h2>Published observations</h2></div><p>Each point links to the exact source revision. Absolute cross-run timing can move with runner state; the same-run ratio is the primary signal.</p></div><div class="benchmark-table-wrap"><table class="history-table"><thead><tr><th>Date</th><th>Fortran p50</th><th>Rust p50</th><th>Ratio</th><th>Commit</th></tr></thead><tbody>{''.join(history_rows)}</tbody></table></div></section></main>"""
    return page_template(row["label"], Path("benchmarks") / f"{row['id']}.html", "benchmarks", body)


def document_group(source: Path) -> str:
    relative = source.relative_to(ROOT)
    if relative == Path("README.md"):
        return "Overview"
    if relative.parts[0] == "docs" and len(relative.parts) > 1:
        return relative.parts[1].replace("-", " ").title()
    if relative.parts[0] == "tracking":
        return "Tracking"
    return "Project records"


def render_docs_navigation(
    current_output: Path,
    sources: list[Path],
    outputs_by_source: dict[Path, Path],
) -> str:
    group_order = {"Overview": 0, "Wiki": 1, "Architecture": 2, "Physics": 3, "Io": 4, "Performance": 5, "Generated": 6, "Tracking": 7, "Project records": 8}
    grouped: dict[str, list[Path]] = {}
    for source in sources:
        grouped.setdefault(document_group(source), []).append(source)
    sections = []
    for group, group_sources in sorted(grouped.items(), key=lambda item: group_order.get(item[0], 99)):
        links = []
        for source in sorted(group_sources, key=lambda item: document_title(item).lower()):
            destination = outputs_by_source[source]
            target = os.path.relpath(destination, current_output.parent).replace(os.sep, "/")
            current = ' aria-current="page"' if destination == current_output else ""
            links.append(f'<a href="{target}" data-doc-link{current}>{html.escape(document_title(source))}</a>')
        sections.append(f'<section class="docs-nav-group"><h2>{html.escape(group)}</h2>{"".join(links)}</section>')
    return "".join(sections)


def render_document_page(
    source: Path,
    current_output: Path,
    sources: list[Path],
    outputs_by_source: dict[Path, Path],
) -> str:
    title = document_title(source)
    navigation = render_docs_navigation(current_output, sources, outputs_by_source)
    article = render_markdown_document(source, current_output, outputs_by_source)
    source_url = f"https://github.com/mikeortman/wrf-rs/blob/main/{source.relative_to(ROOT).as_posix()}"
    body = f"""<main class="page-shell docs-layout"><aside class="docs-sidebar" aria-label="Documentation navigation"><input class="control docs-search" type="search" placeholder="Filter documentation" aria-label="Filter documentation" data-docs-search>{navigation}</aside><article class="doc-article"><div class="doc-breadcrumbs">Documentation / {html.escape(document_group(source))}</div>{article}<footer class="doc-footer">Canonical source: <a href="{html.escape(source_url, quote=True)}">{html.escape(source.relative_to(ROOT).as_posix())}</a></footer></article></main>"""
    return page_template(title, current_output, "docs", body)


def build_pages_site(
    output_directory: Path,
    summary: dict[str, Any],
    current_sha: str,
    run_url: str | None,
) -> None:
    output_directory.mkdir(parents=True, exist_ok=True)
    assets_output = output_directory / "assets"
    assets_output.mkdir(exist_ok=True)
    for asset in (ROOT / "tools" / "pages_assets").iterdir():
        if asset.is_file():
            shutil.copyfile(asset, assets_output / asset.name)

    rows = summary.get("results", [])
    history = summary.get("history", {})
    (output_directory / "index.html").write_text(
        render_dashboard(rows, current_sha, run_url), encoding="utf-8"
    )
    benchmark_output = output_directory / "benchmarks"
    benchmark_output.mkdir(exist_ok=True)
    for row in rows:
        observations = history.get(row["id"], [row])
        (benchmark_output / f"{row['id']}.html").write_text(
            render_suite_page(row, observations), encoding="utf-8"
        )

    sources = document_sources()
    outputs_by_source = {source.resolve(): document_output_path(source) for source in sources}
    for source in sources:
        relative_output = outputs_by_source[source.resolve()]
        destination = output_directory / relative_output
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            render_document_page(source, relative_output, sources, outputs_by_source),
            encoding="utf-8",
        )


def aggregate(
    results_directory: Path,
    output_directory: Path,
    run_url: str | None,
    previous_summary: Path | None,
) -> None:
    results = load_results(results_directory)
    output_directory.mkdir(parents=True, exist_ok=True)
    current_sha = results[0]["source_sha"]
    current_rows = result_rows(results, run_url)
    previous = load_previous_summary(previous_summary)
    rows = merge_result_rows(previous["results"], current_rows)
    history = merge_result_history(previous["history"], current_rows)
    receipt = render_result_markdown(
        current_rows, "Post-merge performance receipt", current_sha, run_url, receipt=True
    )
    markdown = render_result_markdown(
        rows, "Latest matched performance matrix", current_sha, run_url
    )
    summary = {
        "schema_version": 3,
        "source_sha": current_sha,
        "collected_at": results[0]["collected_at"],
        "results": rows,
        "history": history,
    }
    (output_directory / "benchmark-receipt.md").write_text(receipt, encoding="utf-8")
    (output_directory / "benchmark-summary.md").write_text(markdown, encoding="utf-8")
    (output_directory / "benchmark-summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )
    build_pages_site(output_directory, summary, current_sha, run_url)


def build_site_from_summary(
    output_directory: Path,
    previous_summary: Path | None,
    current_sha: str,
    run_url: str | None,
) -> None:
    previous = load_previous_summary(previous_summary)
    summary = {
        "schema_version": 3,
        "source_sha": current_sha,
        "collected_at": max(
            (row.get("collected_at", "") for row in previous["results"]),
            default="",
        ),
        "results": previous["results"],
        "history": previous["history"],
    }
    output_directory.mkdir(parents=True, exist_ok=True)
    (output_directory / "benchmark-summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )
    build_pages_site(output_directory, summary, current_sha, run_url)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("render", help="regenerate Markdown projections")
    subparsers.add_parser("check", help="validate catalogs and generated files")

    select = subparsers.add_parser("select", help="select benchmarks affected by a diff")
    select.add_argument("--base")
    select.add_argument("--head", default="HEAD")
    select.add_argument("--path", action="append", default=[])
    select.add_argument("--all", action="store_true", dest="run_all")
    select.add_argument("--github-output", type=Path)

    run = subparsers.add_parser("run-benchmark", help="run one matched benchmark suite")
    run.add_argument("--id", required=True, dest="identifier")
    run.add_argument("--output-directory", type=Path, required=True)

    aggregate_parser = subparsers.add_parser("aggregate", help="aggregate normalized results")
    aggregate_parser.add_argument("--results-directory", type=Path, required=True)
    aggregate_parser.add_argument("--output-directory", type=Path, required=True)
    aggregate_parser.add_argument("--run-url")
    aggregate_parser.add_argument("--previous-summary", type=Path)
    site_parser = subparsers.add_parser(
        "build-site", help="build Pages from the latest cumulative benchmark summary"
    )
    site_parser.add_argument("--output-directory", type=Path, required=True)
    site_parser.add_argument("--previous-summary", type=Path)
    site_parser.add_argument("--source-sha", required=True)
    site_parser.add_argument("--run-url")
    return parser


def main() -> int:
    arguments = build_parser().parse_args()
    try:
        if arguments.command == "render":
            render()
        elif arguments.command == "check":
            check()
        elif arguments.command == "select":
            paths = arguments.path
            benchmarks = None
            previous_benchmarks = None
            previous_workspace_manifest = None
            workspace_manifest = None
            previous_lockfile = None
            lockfile = None
            if arguments.base:
                paths.extend(changed_paths(arguments.base, arguments.head))
                if "tracking/benchmarks.json" in paths:
                    previous_benchmarks = load_benchmarks_at_revision(arguments.base)
                    benchmarks = load_benchmarks_at_revision(arguments.head)
                if {"Cargo.toml", "Cargo.lock"}.issubset(paths):
                    previous_workspace_manifest = load_toml_at_revision(
                        arguments.base, "Cargo.toml"
                    )
                    workspace_manifest = load_toml_at_revision(
                        arguments.head, "Cargo.toml"
                    )
                    previous_lockfile = load_toml_at_revision(
                        arguments.base, "Cargo.lock"
                    )
                    lockfile = load_toml_at_revision(arguments.head, "Cargo.lock")
            selected = select_suites(
                sorted(set(paths)),
                arguments.run_all,
                benchmarks=benchmarks,
                previous_benchmarks=previous_benchmarks,
                previous_workspace_manifest=previous_workspace_manifest,
                workspace_manifest=workspace_manifest,
                previous_lockfile=previous_lockfile,
                lockfile=lockfile,
            )
            matrix = {"include": selected}
            encoded = json.dumps(matrix, separators=(",", ":"))
            if arguments.github_output:
                with arguments.github_output.open("a", encoding="utf-8") as stream:
                    stream.write(f"matrix={encoded}\ncount={len(selected)}\n")
            else:
                print(encoded)
        elif arguments.command == "run-benchmark":
            print(run_benchmark(arguments.identifier, arguments.output_directory))
        elif arguments.command == "aggregate":
            aggregate(
                arguments.results_directory,
                arguments.output_directory,
                arguments.run_url,
                arguments.previous_summary,
            )
        elif arguments.command == "build-site":
            build_site_from_summary(
                arguments.output_directory,
                arguments.previous_summary,
                arguments.source_sha,
                arguments.run_url,
            )
    except (TrackingError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
