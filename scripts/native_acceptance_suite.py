#!/usr/bin/env python3
"""Run the complete quiet-gated native acceptance matrix and publish its summary."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import os
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURES = (
    "quiet-world",
    "explosive-lattice",
    "sand-release",
    "reservoir-breach",
    "burning-forest",
    "dirty-world-sweep",
    "tiny-capacity",
    "mixed-overload",
)
POLICIES = ("bounded-focus", "bounded-fifo", "traditional")
LOAD_LIMIT = 3.0


def load_one() -> float:
    return os.getloadavg()[0]


def parse_record(text: str, prefix: str) -> dict[str, str] | None:
    line = next((line for line in text.splitlines() if line.startswith(prefix)), None)
    if line is None:
        return None
    return dict(re.findall(r"([a-zA-Z0-9_]+)=([^ ]+)", line))


def parse_capture(text: str) -> dict[str, str] | None:
    return parse_record(text, "SMOKE_CAPTURE ")


def parse_feel(text: str) -> dict[str, str] | None:
    return parse_record(text, "SMOKE_FEEL ")


def fmt_load(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.2f}"


def write_report(path: pathlib.Path, revision: str, attempts: list[dict], started: str) -> None:
    accepted = [item for item in attempts if item["status"] == "accepted"]
    rejected = [item for item in attempts if item["status"] == "rejected"]
    complete = all(
        sum(item["status"] == "accepted" for item in attempts
            if item["fixture"] == fixture and item["policy"] == policy) >= 3
        for fixture in FIXTURES for policy in POLICIES
    )
    lines = [
        "# Native all-fixture acceptance suite v1",
        "",
        f"Status: {'complete' if complete else 'incomplete'}; {len(accepted)}/72 accepted captures; {len(rejected)} rejected attempts.",
        "",
        f"Source revision: `{revision}`  ",
        f"Suite started: {started}  ",
        f"Reference host: `{os.uname().nodename}`, MacBook Air (Mac14,2), Apple M2/8 CPU cores/16 GB RAM/integrated GPU, macOS {platform.mac_ver()[0]}; Rust `{subprocess.check_output(['rustc', '--version'], text=True).strip()}`.",
        f"Capture: release app, 4096² world, 1920×1080, 60 Hz target, profile `m2-16gb-v3`; Cargo.lock SHA-256 `{hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest()}`.",
        "Power mode, thermal state, background activity, and physical display refresh are not controlled.",
        "",
        "Each fixture/policy cell requires three accepted 60-second captures. A run is accepted only when one-minute load is below 3 both immediately before and after the process, the app exits successfully with `SMOKE_RESULT ok`, reports exactly 60 seconds at 1920×1080, has no interval drops, and has all requested telemetry. A rejected attempt is retained below and never contributes to accepted percentiles.",
        "",
        "## Accepted captures",
        "",
        "| Fixture | Policy | Rep | Load before/after | Frames | Frame p99 ms | Max ms | >33.3 ms | Slice p99 ms | Slice max ms |",
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for item in accepted:
        m = item["metrics"]
        lines.append(
            f"| {item['fixture']} | {item['policy']} | {item['rep']} | "
            f"{item['load_before']}/{item['load_after']} | {m['frames']} | {m['p99_ms']} | "
            f"{m['max_ms']} | {m['over_33_3_ms']} | {m['sim_p99_ms']} | {m['max_sim_cpu_ms']} |"
        )
    lines += ["", "## Section 15 checklist rows", "", "| Fixture | Policy | Accepted runs | Frame p99 ≤20 ms | Slice p99 ≤4 ms | >33.3 ms count / max disclosed |", "|---|---|---:|---|---|---|"]
    for fixture in FIXTURES:
        for policy in POLICIES:
            rows = [item for item in accepted if item["fixture"] == fixture and item["policy"] == policy]
            target_runs = len(rows) == 3
            frame = all(float(item["metrics"]["p99_ms"]) <= 20 for item in rows) if target_runs else False
            slice_target = all(float(item["metrics"]["sim_p99_ms"]) <= 4 for item in rows) if target_runs else False
            disclosure = bool(rows) and all(
                "over_33_3_ms" in item["metrics"] and "max_ms" in item["metrics"] for item in rows
            )
            lines.append(
                f"| {fixture} | {policy} | {len(rows)}/3 | "
                f"{'met' if frame else 'unmet' if target_runs else 'incomplete'} | "
                f"{'met' if slice_target else 'unmet' if target_runs else 'incomplete'} | "
                f"{'reported' if disclosure else 'incomplete'} |"
            )
    lines += ["", "## Interaction feedback samples", "", "These CPU-side event/action-to-visible-upload p95 values use the app's documented method. Counts below 30 are descriptive only; quiet-world intentionally has no scripted interaction samples.", "", "| Fixture | Policy | Rep | Camera n / p95 ms | Paint n / p95 ms | Ignite n / p95 ms | Detonate n / p95 ms |", "|---|---|---:|---:|---:|---:|---:|"]
    for item in accepted:
        feel = item.get("feel", {})
        lines.append(
            f"| {item['fixture']} | {item['policy']} | {item['rep']} | "
            f"{feel.get('camera_n', 'n/a')} / {feel.get('camera_p95_ms', 'n/a')} | "
            f"{feel.get('paint_n', 'n/a')} / {feel.get('paint_p95_ms', 'n/a')} | "
            f"{feel.get('ignite_n', 'n/a')} / {feel.get('ignite_p95_ms', 'n/a')} | "
            f"{feel.get('detonate_n', 'n/a')} / {feel.get('detonate_p95_ms', 'n/a')} |"
        )
    lines += ["", "## Rejected attempts", ""]
    if rejected:
        lines += ["| Fixture | Policy | Rep | Attempt | Load before/after | Reason | Log |", "|---|---|---:|---:|---:|---|---|"]
        for item in rejected:
            lines.append(
                f"| {item['fixture']} | {item['policy']} | {item['rep']} | {item['attempt']} | "
                f"{item['load_before']}/{item['load_after']} | {item['reason']} | `{item['log']}` |"
            )
    else:
        lines.append("None.")
    lines += [
        "",
        "## Interpretation and limits",
        "",
        "Frame intervals are native windowed event-loop intervals; they include presentation pacing and are not GPU duration. Slice CPU values are monotonic wall-time samples around the app's simulation dispatch, not thread CPU time. Fixture captures do not add external disturbances or player actions, except mixed-overload, which uses the app's scripted sustained-overload action stream to exercise focus latency. The load gate does not exclude all OS/driver interference.",
        "",
        "Raw per-attempt logs are retained locally under `benchmarks/tmp/` and are not committed.",
        "",
    ]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=pathlib.Path, help="use an existing release binary instead of building")
    parser.add_argument("--output", type=pathlib.Path, default=ROOT / "benchmarks/results/native-acceptance-v1.md")
    parser.add_argument("--fixtures", nargs="+", choices=FIXTURES, default=FIXTURES, help=argparse.SUPPRESS)
    parser.add_argument("--policies", nargs="+", choices=POLICIES, default=POLICIES, help=argparse.SUPPRESS)
    parser.add_argument("--repetitions", type=int, default=3, help=argparse.SUPPRESS)
    parser.add_argument("--seconds", type=int, default=60, help=argparse.SUPPRESS)
    parser.add_argument("--no-caffeinate", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 60 or args.repetitions < 1:
        parser.error("seconds must be 1..=60 and repetitions must be positive")

    if args.binary:
        binary = args.binary.resolve()
    else:
        subprocess.run(["cargo", "build", "--release", "-p", "cascade-app"], cwd=ROOT, check=True)
        binary = ROOT / "target/release/cascade-app"
    if not binary.is_file():
        parser.error(f"app binary not found: {binary}")
    if not args.no_caffeinate and shutil.which("caffeinate") is None:
        parser.error("caffeinate is required on the reference macOS host; use --no-caffeinate only for functional tests")

    log_dir = ROOT / "benchmarks/tmp/native-acceptance"
    log_dir.mkdir(parents=True, exist_ok=True)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    started = dt.datetime.now().astimezone().isoformat(timespec="seconds")
    attempts: list[dict] = []
    output = args.output if args.output.is_absolute() else ROOT / args.output
    write_report(output, revision, attempts, started)
    fixture_policy_rep = [
        (fixture, policy, rep)
        for rep in range(1, args.repetitions + 1)
        for fixture in args.fixtures
        for policy in (args.policies if rep % 2 else tuple(reversed(args.policies)))
        for _ in (0,)
    ]
    try:
        for fixture, policy, rep in fixture_policy_rep:
            while sum(item["status"] == "accepted" for item in attempts
                      if item["fixture"] == fixture and item["policy"] == policy) < rep:
                attempt_number = sum(item["fixture"] == fixture and item["policy"] == policy
                                     and item["rep"] == rep for item in attempts) + 1
                before = load_one()
                base = dict(fixture=fixture, policy=policy, rep=rep, attempt=attempt_number,
                            load_before=fmt_load(before), load_after="n/a", log="-")
                if before >= LOAD_LIMIT:
                    base.update(status="rejected", reason="pre-run one-minute load not below 3")
                    attempts.append(base)
                    print(f"REJECT {fixture}/{policy} rep={rep} load={before:.2f} before launch", flush=True)
                    write_report(output, revision, attempts, started)
                    time.sleep(5)
                    continue
                log_name = f"{fixture}-{policy}-rep{rep}-attempt{attempt_number}.log"
                log_path = log_dir / log_name
                base["log"] = str(log_path.relative_to(ROOT))
                command = [str(binary), "--capture-policy", policy, "--fixture", fixture,
                           "--capture-seconds", str(args.seconds), "--world-size", "4096"]
                if not args.no_caffeinate and shutil.which("caffeinate"):
                    command = ["caffeinate", "-dimsu", *command]
                print(f"RUN {fixture}/{policy} rep={rep} attempt={attempt_number} load={before:.2f}", flush=True)
                with log_path.open("w", encoding="utf-8") as log:
                    result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
                after = load_one()
                base["load_after"] = fmt_load(after)
                text = log_path.read_text(encoding="utf-8", errors="replace")
                metrics = parse_capture(text)
                feel = parse_feel(text)
                starts = [line for line in text.splitlines() if line.startswith("SMOKE_CAPTURE_STARTED ")]
                feel_lines = [line for line in text.splitlines() if line.startswith("SMOKE_FEEL ")]
                reasons = []
                if before >= LOAD_LIMIT or after >= LOAD_LIMIT:
                    reasons.append("one-minute load gate failed")
                if result.returncode != 0 or "SMOKE_RESULT ok" not in text:
                    reasons.append(f"app failed (exit {result.returncode} or no SMOKE_RESULT ok)")
                if metrics is None or len(starts) != 1 or feel is None or len(feel_lines) != 1:
                    reasons.append("missing or ambiguous capture telemetry")
                elif (
                    metrics.get("frames") != str(args.seconds * 60)
                    or metrics.get("interval_drops") != "0"
                    or int(metrics.get("sim_samples", "0")) < int(metrics.get("frames", "0"))
                    or int(metrics.get("sim_samples", "0")) > int(metrics.get("frames", "0")) + 1
                    or metrics.get("policy") != policy
                    or feel.get("policy") != policy
                    or not any(
                        f"fixture={fixture} " in line
                        and f"seconds={args.seconds} " in line
                        and "render=1920x1080 " in line
                        for line in starts
                    )
                ):
                    reasons.append("capture duration, fixture, policy, or sample completeness mismatch")
                if reasons:
                    base.update(status="rejected", reason="; ".join(reasons))
                    attempts.append(base)
                    print(f"REJECT {fixture}/{policy} rep={rep} load={before:.2f}/{after:.2f}: {base['reason']}", flush=True)
                    if any(reason != "one-minute load gate failed" for reason in reasons):
                        print("Stopping after a non-load capture failure; inspect the retained log and fix the cause before retrying.", file=sys.stderr)
                        return 1
                else:
                    base.update(status="accepted", reason="", metrics=metrics, feel=feel)
                    attempts.append(base)
                    print(f"ACCEPT {fixture}/{policy} rep={rep} load={before:.2f}/{after:.2f} p99={metrics['p99_ms']} ms", flush=True)
                write_report(output, revision, attempts, started)
    except KeyboardInterrupt:
        print("Interrupted; partial results were written.", file=sys.stderr)
        return 130
    finally:
        write_report(output, revision, attempts, started)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
