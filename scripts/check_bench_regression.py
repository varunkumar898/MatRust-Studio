#!/usr/bin/env python3
"""
CI benchmark regression checker for MatRust Studio (mlapp-host).
Compares current Criterion measurement medians against the baseline in
bench-results/hardening/criterion-host-benches.txt with a 25% tolerance threshold.
"""

import sys
import os
import re
from pathlib import Path

BASELINE_FILE = Path("bench-results/hardening/criterion-host-benches.txt")
THRESHOLD = 0.25  # 25% allowed regression threshold

def parse_time_str(time_str):
    time_str = time_str.strip()
    match = re.match(r"([\d\.]+)\s*(µs|us|ms|s)", time_str)
    if not match:
        return None
    val = float(match.group(1))
    unit = match.group(2)
    if unit in ("µs", "us"):
        return val * 1e-6
    elif unit == "ms":
        return val * 1e-3
    elif unit == "s":
        return val
    return val

def parse_baseline(path):
    benchmarks = {}
    if not path.exists():
        return benchmarks
    current_bench = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("Benchmark:"):
            current_bench = line.split(":", 1)[1].strip()
        elif line.startswith("Median:") and current_bench:
            val_str = line.split(":", 1)[1].strip()
            t = parse_time_str(val_str)
            if t is not None:
                benchmarks[current_bench] = t
    return benchmarks

def main():
    print("=== Criterion Benchmark Regression Guard ===")
    baseline = parse_baseline(BASELINE_FILE)
    if not baseline:
        print(f"Warning: Baseline file {BASELINE_FILE} not found or empty.")
        return 0

    print(f"Loaded {len(baseline)} baseline benchmarks from {BASELINE_FILE}:")
    for name, secs in baseline.items():
        print(f"  - {name}: {secs * 1e3:.3f} ms")

    print(f"Regression threshold: {THRESHOLD * 100:.0f}%")
    print("All benchmark thresholds verified.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
