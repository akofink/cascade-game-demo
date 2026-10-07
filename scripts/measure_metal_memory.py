#!/usr/bin/env python3
"""Measure Metal System Trace's per-process currentAllocatedSize high-water."""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=pathlib.Path, help="use an existing release binary")
    parser.add_argument("--fixture", default="mixed-overload", choices=(
        "quiet-world", "explosive-lattice", "sand-release", "reservoir-breach",
        "burning-forest", "dirty-world-sweep", "tiny-capacity", "mixed-overload",
    ))
    parser.add_argument("--policy", default="bounded-focus", choices=(
        "bounded-focus", "bounded-fifo", "traditional",
    ))
    parser.add_argument("--seconds", type=int, default=8, help="short functional capture duration, 1..60")
    args = parser.parse_args()
    if not 1 <= args.seconds <= 60:
        parser.error("seconds must be in 1..=60")
    if subprocess.run(["xcrun", "xctrace", "list", "templates"], capture_output=True).returncode:
        parser.error("xcrun/xctrace is unavailable")

    if args.binary:
        binary = args.binary.resolve()
    else:
        subprocess.run(["cargo", "build", "--release", "-p", "cascade-app"], cwd=ROOT, check=True)
        binary = ROOT / "target/release/cascade-app"
    if not binary.is_file():
        parser.error(f"app binary not found: {binary}")

    output_dir = ROOT / "benchmarks/tmp/metal-memory"
    output_dir.mkdir(parents=True, exist_ok=True)
    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    trace = output_dir / f"metal-memory-{stamp}.trace"
    table_xml = output_dir / f"metal-memory-{stamp}.xml"
    command = [
        "xcrun", "xctrace", "record", "--template", "Metal System Trace",
        "--output", str(trace), "--launch", "--", str(binary),
        "--capture-policy", args.policy, "--fixture", args.fixture,
        "--capture-seconds", str(args.seconds), "--world-size", "4096",
    ]
    print("Recording only GPU allocation telemetry; do not use this run for performance results.", flush=True)
    print("Command:", " ".join(command), flush=True)
    recorded = subprocess.run(command, cwd=ROOT)
    if not trace.exists():
        return recorded.returncode or 1

    export = subprocess.run([
        "xcrun", "xctrace", "export", "--input", str(trace),
        "--xpath", '/trace-toc/run[@number="1"]/data/table[@schema="metal-current-allocated-size"]',
        "--output", str(table_xml),
    ])
    if export.returncode:
        return export.returncode
    xml = ET.parse(table_xml).getroot()
    sizes = [int(node.text) for node in xml.iter("size-in-bytes") if node.text]
    if not sizes:
        print("Metal System Trace returned no currentAllocatedSize samples", file=sys.stderr)
        return 1
    peak = max(sizes)
    print(f"Metal currentAllocatedSize high-water: {peak:,} B ({peak / 1024 / 1024:.3f} MiB)")
    print(f"Allocation-size events: {len(sizes)}")
    print(f"Trace: {trace.relative_to(ROOT)}")
    print(f"Export: {table_xml.relative_to(ROOT)}")
    return recorded.returncode


if __name__ == "__main__":
    raise SystemExit(main())
