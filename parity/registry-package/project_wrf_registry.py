#!/usr/bin/env python3
"""Project generated WRF Registry artifacts into stable package semantics."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


PACKAGE_CONSTANT = re.compile(r"^\s*INTEGER, PARAMETER :: (\w+) = (-?\d+)\s*$")
PACKAGE_CONDITION = re.compile(
    r"^\s*IF \(model_config_rec%(\w+)(?:\(idomain\))?==(-?\d+)\)THEN\s*$"
)
PACKAGE_MEMBER = re.compile(
    r"^\s*IF \( (\w+)_index_table\( PARAM_(\w+) , idomain \) \.lt\. 1 \) THEN\s*$"
)
PARAMETER_INDEX = re.compile(r"^\s*INTEGER, PARAMETER :: PARAM_(\w+) = (\d+)\s*$")
PARAMETER_COUNT = re.compile(r"^\s*INTEGER, PARAMETER :: PARAM_NUM_(\w+) = (\d+)\s*$")
FIRST_PACKED = re.compile(r"^\s*INTEGER\s*, PARAMETER :: PARAM_FIRST_SCALAR = (\d+)\s*$")


@dataclass(frozen=True)
class GeneratedPackage:
    name: str
    configuration_name: str
    choice: int
    groups: tuple[tuple[str, tuple[str, ...]], ...]


def fail(message: str) -> None:
    raise SystemExit(f"WRF Registry projection failed: {message}")


def parse_module(path: Path) -> tuple[list[tuple[str, int]], dict[str, int], dict[str, int], int]:
    package_constants: list[tuple[str, int]] = []
    parameter_indices: dict[str, int] = {}
    parameter_counts: dict[str, int] = {}
    first_packed: int | None = None
    is_package_section = False

    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip() == "! package constants":
            is_package_section = True
            continue
        if line.strip() == "! 4D array constants":
            is_package_section = False
            continue

        if is_package_section and (match := PACKAGE_CONSTANT.match(line)):
            package_constants.append((match.group(1), int(match.group(2))))
        elif match := PARAMETER_COUNT.match(line):
            parameter_counts[match.group(1)] = int(match.group(2))
        elif match := FIRST_PACKED.match(line):
            first_packed = int(match.group(1))
        elif match := PARAMETER_INDEX.match(line):
            parameter_indices[match.group(1)] = int(match.group(2))

    if not package_constants:
        fail(f"no package constants found in {path}")
    if not parameter_indices or not parameter_counts or first_packed is None:
        fail(f"incomplete scalar constants in {path}")
    return package_constants, parameter_indices, parameter_counts, first_packed


def parse_package_blocks(path: Path) -> list[tuple[str, int, tuple[tuple[str, tuple[str, ...]], ...]]]:
    blocks: list[tuple[str, int, tuple[tuple[str, tuple[str, ...]], ...]]] = []
    configuration_name: str | None = None
    choice: int | None = None
    groups: list[tuple[str, list[str]]] = []

    def finish_block() -> None:
        nonlocal configuration_name, choice, groups
        if configuration_name is None or choice is None:
            return
        frozen_groups = tuple((name, tuple(members)) for name, members in groups)
        blocks.append((configuration_name, choice, frozen_groups))
        configuration_name = None
        choice = None
        groups = []

    for line in path.read_text(encoding="utf-8").splitlines():
        if match := PACKAGE_CONDITION.match(line):
            finish_block()
            configuration_name = match.group(1)
            choice = int(match.group(2))
            continue
        if configuration_name is None:
            continue
        if match := PACKAGE_MEMBER.match(line):
            scalar_array_name = match.group(1)
            member_name = match.group(2)
            if not groups or groups[-1][0] != scalar_array_name:
                groups.append((scalar_array_name, []))
            groups[-1][1].append(member_name)

    finish_block()
    if not blocks:
        fail(f"no package conditions found in {path}")
    return blocks


def combine_packages(
    constants: list[tuple[str, int]],
    blocks: list[tuple[str, int, tuple[tuple[str, tuple[str, ...]], ...]]],
) -> list[GeneratedPackage]:
    if len(constants) != len(blocks):
        fail(f"package count differs between generated artifacts: {len(constants)} != {len(blocks)}")

    packages: list[GeneratedPackage] = []
    for (name, constant_choice), (configuration_name, block_choice, groups) in zip(
        constants, blocks, strict=True
    ):
        if constant_choice != block_choice:
            fail(f"choice differs for package {name}: {constant_choice} != {block_choice}")
        packages.append(GeneratedPackage(name, configuration_name, block_choice, groups))
    return packages


def format_groups(groups: tuple[tuple[str, tuple[str, ...]], ...]) -> str:
    if not groups:
        return "-"
    return ";".join(f"{name}:{','.join(members)}" for name, members in groups)


def main() -> None:
    if len(sys.argv) != 3:
        fail("usage: project_wrf_registry.py MODULE_STATE_DESCRIPTION SCALAR_INDICES")

    module_path = Path(sys.argv[1])
    scalar_indices_path = Path(sys.argv[2])
    constants, parameter_indices, parameter_counts, first_packed = parse_module(module_path)
    blocks = parse_package_blocks(scalar_indices_path)
    packages = combine_packages(constants, blocks)

    for package in packages:
        print(
            f"PACKAGE|name={package.name}|configuration={package.configuration_name}"
            f"|choice={package.choice}|groups={format_groups(package.groups)}"
        )

    if set(parameter_counts) != {"moist"}:
        fail(f"expected only the moist scalar array, got {sorted(parameter_counts)}")
    print(
        f"ARRAY|name=moist|definition_member_count={parameter_counts['moist']}"
        f"|reserved_parameter=0|first_packed={first_packed}"
    )
    for member_name, parameter_index in parameter_indices.items():
        print(f"MEMBER|array=moist|name={member_name}|parameter={parameter_index}")


if __name__ == "__main__":
    main()
