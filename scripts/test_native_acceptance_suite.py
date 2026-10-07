import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import native_acceptance_suite as suite
from native_acceptance_suite import parse_capture, write_report


class AcceptanceSuiteTests(unittest.TestCase):
    def test_parse_capture_fields(self):
        line = (
            "SMOKE_CAPTURE policy=bounded-focus frames=3600 p99_ms=18.123 max_ms=21.456 "
            "over_33_3_ms=0 interval_drops=0 sim_p99_ms=2.500 sim_samples=3600 "
            "max_sim_cpu_ms=4.000"
        )
        self.assertEqual(parse_capture(line)["p99_ms"], "18.123")
        self.assertEqual(parse_capture(line)["sim_samples"], "3600")
        feel = "SMOKE_FEEL policy=bounded-focus camera_n=240 camera_p95_ms=18.2 paint_n=30 paint_p95_ms=19.0 ignite_n=30 ignite_p95_ms=20.0 detonate_n=30 detonate_p95_ms=21.0"
        self.assertEqual(suite.parse_feel(feel)["detonate_p95_ms"], "21.0")

    def test_validator_accepts_actual_low_frame_count_for_complete_timed_capture(self):
        text = "\n".join([
            "SMOKE_CAPTURE_STARTED policy=traditional fixture=quiet-world seconds=60 render=1920x1080 load_gate=external",
            "SMOKE_FEEL policy=traditional focus=linked camera_n=0 camera_p50_ms=0 camera_p95_ms=0 paint_n=0 paint_p50_ms=0 paint_p95_ms=0 ignite_n=0 ignite_p50_ms=0 ignite_p95_ms=0 detonate_n=0 detonate_p50_ms=0 detonate_p95_ms=0",
            "SMOKE_CAPTURE policy=traditional frames=1190 p50_ms=50 p95_ms=50.2 p99_ms=66.646 max_ms=67.612 over_33_3_ms=1190 interval_drops=0 sim_p99_ms=48.878 sim_samples=1191 max_sim_cpu_ms=68.663",
            "SMOKE_RESULT ok",
        ])
        metrics, feel, reasons = suite.validate_capture(text, "quiet-world", "traditional", 60)
        self.assertEqual(metrics["frames"], "1190")
        self.assertEqual(feel["policy"], "traditional")
        self.assertEqual(reasons, [])

    def test_runner_accepts_complete_functional_sample_and_writes_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fake_app = root / "fake-app"
            fake_app.write_text(
                "#!/bin/sh\n"
                "echo 'SMOKE_CAPTURE_STARTED policy=bounded-fifo fixture=quiet-world seconds=1 render=1920x1080 load_gate=external'\n"
                "echo 'SMOKE_FEEL policy=bounded-fifo camera_n=0 camera_p50_ms=0 camera_p95_ms=0 paint_n=0 paint_p50_ms=0 paint_p95_ms=0 ignite_n=0 ignite_p50_ms=0 ignite_p95_ms=0 detonate_n=0 detonate_p50_ms=0 detonate_p95_ms=0'\n"
                "echo 'SMOKE_CAPTURE policy=bounded-fifo frames=42 p99_ms=18.123 max_ms=19.000 over_33_3_ms=0 interval_drops=0 sim_p99_ms=2.500 sim_samples=43 max_sim_cpu_ms=3.000'\n"
                "echo 'SMOKE_RESULT ok'\n",
                encoding="utf-8",
            )
            fake_app.chmod(0o755)
            report = root / "report.md"
            argv = [
                "native_acceptance_suite.py", "--binary", str(fake_app), "--output", str(report),
                "--fixtures", "quiet-world", "--policies", "bounded-fifo", "--repetitions", "1",
                "--seconds", "1", "--no-caffeinate",
            ]
            with patch("sys.argv", argv), patch.object(suite, "load_one", return_value=1.0):
                self.assertEqual(suite.main(), 0)
            text = report.read_text(encoding="utf-8")
            self.assertIn("1/72 accepted captures", text)
            self.assertIn("quiet-world | bounded-fifo | 1 | 1.00/1.00", text)
            self.assertIn("## Interaction feedback samples", text)
            self.assertIn("quiet-world | bounded-fifo | 1 | 0 / 0", text)

    def test_resume_reclassifies_retained_complete_capture_without_recapture(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.lock").write_bytes((suite.ROOT / "Cargo.lock").read_bytes())
            log_path = root / "benchmarks/tmp/native-acceptance/run.log"
            log_path.parent.mkdir(parents=True)
            log_path.write_text("\n".join([
                "SMOKE_CAPTURE_STARTED policy=traditional fixture=quiet-world seconds=60 render=1920x1080 load_gate=external",
                "SMOKE_FEEL policy=traditional focus=linked camera_n=0 camera_p50_ms=0 camera_p95_ms=0 paint_n=0 paint_p50_ms=0 paint_p95_ms=0 ignite_n=0 ignite_p50_ms=0 ignite_p95_ms=0 detonate_n=0 detonate_p50_ms=0 detonate_p95_ms=0",
                "SMOKE_CAPTURE policy=traditional frames=1190 p50_ms=50 p95_ms=50.2 p99_ms=66.646 max_ms=67.612 over_33_3_ms=1190 interval_drops=0 sim_p99_ms=48.878 sim_samples=1191 max_sim_cpu_ms=68.663",
                "SMOKE_RESULT ok",
            ]), encoding="utf-8")
            attempt = dict(
                fixture="quiet-world", policy="traditional", rep=1, attempt=1,
                load_before="2.40", load_after="2.66", status="rejected",
                reason="capture duration, fixture, policy, or sample completeness mismatch",
                log=str(log_path.relative_to(root)),
            )
            report = root / "report.md"
            with patch.object(suite, "ROOT", root):
                write_report(report, "d05cb91", [attempt], "2026-10-06T23:54:00-04:00")
                revision, started, attempts, notes = suite.load_report_attempts(report)
            self.assertEqual(revision, "d05cb91")
            self.assertEqual(attempts[0]["status"], "accepted")
            self.assertEqual(attempts[0]["metrics"]["frames"], "1190")
            self.assertIn("no recapture was made", notes[0])

    def test_resume_marks_occluded_pre_capture_failure_retryable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.lock").write_bytes((suite.ROOT / "Cargo.lock").read_bytes())
            log_path = root / "benchmarks/tmp/native-acceptance/startup.log"
            log_path.parent.mkdir(parents=True)
            log_path.write_text(
                "smoke timed out (timeout=0 occluded=6458 outdated=0)\n"
                "smoke presents=0 pan=false zoom=false readback=false",
                encoding="utf-8",
            )
            attempt = dict(
                fixture="explosive-lattice", policy="bounded-focus", rep=1, attempt=1,
                load_before="1.68", load_after="1.41", status="rejected",
                reason="app failed (exit 1 or no SMOKE_RESULT ok); missing or ambiguous capture telemetry",
                log=str(log_path.relative_to(root)),
            )
            report = root / "report.md"
            with patch.object(suite, "ROOT", root):
                write_report(report, "d05cb91", [attempt], "2026-10-07T00:57:00-04:00")
                _, _, attempts, notes = suite.load_report_attempts(report)
            self.assertFalse(suite.is_occluded_startup_failure(
                log_path.read_text(encoding="utf-8") + " SMOKE_CAPTURE_STARTED policy=bounded-focus"
            ))
            self.assertEqual(attempts[0]["status"], "rejected")
            self.assertEqual(attempts[0]["reason"], suite.STARTUP_FAILURE_REASON)
            self.assertIn("no timing data was collected", notes[0])

    def test_report_keeps_rejected_attempt_and_marks_incomplete(self):
        attempts = [
            {
                "fixture": "quiet-world",
                "policy": "bounded-focus",
                "rep": 1,
                "attempt": 1,
                "load_before": "2.00",
                "load_after": "3.10",
                "status": "rejected",
                "reason": "one-minute load gate failed",
                "log": "benchmarks/tmp/run.log",
            }
        ]
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "results.md"
            write_report(output, "abc123", attempts, "2026-10-06T00:00:00-04:00")
            report = output.read_text()
        self.assertIn("Status: incomplete", report)
        self.assertIn("| quiet-world | bounded-focus | 1 | 1 | 2.00/3.10 |", report)
        self.assertIn("one-minute load gate failed", report)
        self.assertIn("| quiet-world | bounded-focus | 0/3 |", report)


if __name__ == "__main__":
    unittest.main()
