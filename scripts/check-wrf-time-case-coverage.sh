#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
golden_output="$repository_root/upstream/WRF/external/esmf_time_f90/Test1.out.correct"
rust_source="$repository_root/crates/wrf-time/src"
active_cases="${TMPDIR:-/tmp}/wrf-time-active-cases.$$"
missing_cases="${TMPDIR:-/tmp}/wrf-time-missing-cases.$$"

cleanup() {
    rm -f "$active_cases" "$missing_cases"
}
trap cleanup EXIT HUP INT TERM

if [ ! -f "$golden_output" ]; then
    echo "WRF source is missing; run scripts/fetch-wrf.sh first" >&2
    exit 1
fi

rg '^PASS:' "$golden_output" | sed -E 's/^PASS:[[:space:]]+//' > "$active_cases"
printf '%s\n' \
    SimpleClockAdvance \
    StdYearClockAdvance \
    LeapYearClockAdvance \
    LeapYearFractionClockAdvance >> "$active_cases"
sort -u "$active_cases" -o "$active_cases"

: > "$missing_cases"
while IFS= read -r case_name; do
    if ! rg -q --fixed-strings "\"$case_name\"" "$rust_source"; then
        printf '%s\n' "$case_name" >> "$missing_cases"
    fi
done < "$active_cases"

active_count=$(wc -l < "$active_cases" | tr -d ' ')
missing_count=$(wc -l < "$missing_cases" | tr -d ' ')
covered_count=$((active_count - missing_count))

if [ "$missing_count" -ne 0 ]; then
    echo "WRF time case coverage: $covered_count/$active_count"
    echo "Missing active cases:"
    cat "$missing_cases"
    exit 1
fi

echo "PASS WRF time case coverage: $covered_count/$active_count"
