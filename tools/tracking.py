#!/usr/bin/env python3
"""Validate, select, run, and aggregate wrf-rs tracking evidence."""

from __future__ import annotations

import argparse
import datetime as dt
import fnmatch
import html
import json
import os
import platform
import re
import statistics
import subprocess
import sys
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


def select_suites(paths: list[str], run_all: bool = False) -> list[dict[str, str]]:
    benchmarks = load_benchmarks()
    suites = benchmarks["suites"]
    global_change = any(path_matches(path, benchmarks["global_watch"]) for path in paths)
    selected = suites if run_all or global_change else [
        suite for suite in suites if any(path_matches(path, suite["watch"]) for path in paths)
    ]
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
        estimates.append(
            {
                "workers": workers,
                "median_nanoseconds": median["point_estimate"],
                "lower_nanoseconds": interval["lower_bound"],
                "upper_nanoseconds": interval["upper_bound"],
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
        serial_ms = milliseconds(serial["median_nanoseconds"])
        best_ms = milliseconds(best["median_nanoseconds"])
        rows.append(
            {
                "id": result["suite"]["id"],
                "label": result["suite"]["label"],
                "fortran_milliseconds": fortran_ms,
                "rust_serial_milliseconds": serial_ms,
                "rust_best_milliseconds": best_ms,
                "rust_best_workers": best["workers"],
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


def load_previous_rows(path: Path | None) -> list[dict[str, Any]]:
    if path is None or not path.is_file():
        return []
    summary = load_json(path)
    if summary.get("schema_version") != 2 or not isinstance(summary.get("results"), list):
        raise TrackingError(f"unsupported previous performance summary: {path}")
    return summary["results"]


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
            "These are same-runner Rust/Fortran comparisons. Hosted runners vary, so small cross-run changes are noise, not a regression verdict.",
        ]
    )
    if run_url:
        lines.append(f"Raw logs, JSON, and workflow summary: {run_url}")
    lines.append("")
    return "\n".join(lines)


def render_result_html(
    rows_to_render: list[dict[str, Any]], current_sha: str, run_url: str | None
) -> str:
    rows = []
    for row in rows_to_render:
        source_url = f"https://github.com/mikeortman/wrf-rs/commit/{row['source_sha']}"
        rows.append(
            "<tr>"
            f"<td>{html.escape(row['label'])}</td>"
            f"<td>{row['fortran_milliseconds']:.6g} ms</td>"
            f"<td>{row['rust_serial_milliseconds']:.6g} ms</td>"
            f"<td>{row['rust_best_milliseconds']:.6g} ms ({row['rust_best_workers']} workers)</td>"
            f"<td>{html.escape(row['best_relative'])}</td>"
            f'<td><a href="{html.escape(source_url)}">{html.escape(row["source_sha"][:8])}</a></td>'
            "</tr>"
        )
    source = html.escape(current_sha)
    run_link = f'<p><a href="{html.escape(run_url)}">Workflow evidence</a></p>' if run_url else ""
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>wrf-rs performance</title><style>
body{{font:16px/1.5 system-ui,sans-serif;max-width:1200px;margin:2rem auto;padding:0 1rem;color:#17202a}}
table{{border-collapse:collapse;width:100%}}th,td{{border:1px solid #ccd1d1;padding:.55rem;text-align:left}}
th{{background:#edf2f7}}code{{background:#edf2f7;padding:.15rem .3rem}}.note{{max-width:80ch;color:#52606d}}
</style></head><body><h1>wrf-rs post-merge performance</h1><p>Source <code>{source}</code></p>
<p class="note">Matched Rust and optimized Fortran run on the same GitHub-hosted runner. Relative results are useful; small cross-run absolute changes are not.</p>
<table><thead><tr><th>Suite</th><th>Fortran median</th><th>Rust, 1 worker</th><th>Best Rust</th><th>Best comparison</th><th>Updated</th></tr></thead>
<tbody>{''.join(rows)}</tbody></table>{run_link}</body></html>"""


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
    rows = merge_result_rows(load_previous_rows(previous_summary), current_rows)
    receipt = render_result_markdown(
        current_rows, "Post-merge performance receipt", current_sha, run_url, receipt=True
    )
    markdown = render_result_markdown(
        rows, "Latest matched performance matrix", current_sha, run_url
    )
    summary = {
        "schema_version": 2,
        "source_sha": current_sha,
        "collected_at": results[0]["collected_at"],
        "results": rows,
    }
    (output_directory / "benchmark-receipt.md").write_text(receipt, encoding="utf-8")
    (output_directory / "benchmark-summary.md").write_text(markdown, encoding="utf-8")
    (output_directory / "benchmark-summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )
    (output_directory / "index.html").write_text(
        render_result_html(rows, current_sha, run_url), encoding="utf-8"
    )


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
            if arguments.base:
                paths.extend(changed_paths(arguments.base, arguments.head))
            selected = select_suites(sorted(set(paths)), arguments.run_all)
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
    except (TrackingError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
