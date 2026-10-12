import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import verify_darwin_smoke as proof


class DarwinSmokeContract(unittest.TestCase):
    def test_only_latest_exact_revision_native_work_is_proof(self):
        sha = "a" * 40
        base = "repos/Yazelix/nova/actions"
        workflow = {"id": 42, "path": ".github/workflows/darwin_package_smoke.yml"}
        run = {
            "id": 10, "workflow_id": 42, "path": workflow["path"],
            "repository": {"id": 746026727, "full_name": "Yazelix/nova"},
            "head_repository": {"id": 746026727, "full_name": "Yazelix/nova"},
            "head_sha": sha, "event": "workflow_dispatch", "run_attempt": 2,
            "updated_at": "2026-10-12T12:00:00Z",
            "status": "completed", "conclusion": "success",
        }
        job = {
            "id": 100, "run_id": 10, "run_attempt": 2, "head_sha": sha,
            "name": "macos-package-smoke", "labels": ["macos-15"],
            "status": "completed", "conclusion": "success",
            "steps": [{"name": name, "status": "completed", "conclusion": "success"}
                      for name in ["Build Yazi PTY checks",
                                   "Build remaining aarch64-darwin packages and checks"]],
        }
        runs_url = f"{base}/workflows/42/runs?head_sha={sha}&per_page=100"
        jobs_url = f"{base}/runs/10/attempts/2/jobs?per_page=100"
        fixture = {
            f"{base}/workflows/darwin_package_smoke.yml": [workflow],
            runs_url: [{"total_count": 1, "workflow_runs": [run]}],
            jobs_url: [{"total_count": 1, "jobs": [job]}],
            f"{base}/runs/10": [copy.deepcopy(run)],
        }

        result = proof.verify(sha, fixture.__getitem__)
        self.assertIn("https://github.com/Yazelix/nova/actions/runs/10", result)
        self.assertIn(sha, result)
        self.assertIn("attempt 2", result)
        self.assertIn("Build Yazi PTY checks", result)

        # A native scheduled run is valid; its idle/no-op form is not.
        scheduled = copy.deepcopy(fixture)
        scheduled[runs_url][0]["workflow_runs"][0]["event"] = "schedule"
        scheduled[f"{base}/runs/10"][0]["event"] = "schedule"
        proof.verify(sha, scheduled.__getitem__)

        for record, key, value in [
            ("workflow", "path", ".github/workflows/version_gate.yml"),
            ("run", "workflow_id", 99),
            ("run", "path", ".github/workflows/version_gate.yml"),
            ("run", "repository", {"id": 1, "full_name": "other/nova"}),
            ("run", "head_repository", {"id": 1, "full_name": "other/nova"}),
            ("run", "head_sha", "b" * 40),
            ("run", "event", "pull_request"),
            ("run", "status", "in_progress"),
            ("run", "conclusion", "failure"),
            ("run", "conclusion", "cancelled"),
            ("run", "conclusion", "timed_out"),
            ("run", "conclusion", "skipped"),
            ("job", "name", "some-other-job"),
            ("job", "run_id", 99),
            ("job", "run_attempt", 1),
            ("job", "head_sha", "b" * 40),
            ("job", "labels", ["ubuntu-latest"]),
            ("job", "conclusion", "failure"),
            ("job", "steps", []),
            ("step", "conclusion", "failure"),
            ("step", "conclusion", "skipped"),
            ("step", "status", "in_progress"),
            ("refresh", "id", 99),
            ("refresh", "run_attempt", 3),
            ("refresh", "status", "in_progress"),
        ]:
            with self.subTest(record=record, key=key, value=value):
                data = copy.deepcopy(fixture)
                records = {
                    "workflow": data[f"{base}/workflows/darwin_package_smoke.yml"][0],
                    "run": data[runs_url][0]["workflow_runs"][0],
                    "job": data[jobs_url][0]["jobs"][0],
                    "step": data[jobs_url][0]["jobs"][0]["steps"][0],
                    "refresh": data[f"{base}/runs/10"][0],
                }
                records[record][key] = value
                with self.assertRaises((ValueError, KeyError, TypeError)):
                    proof.verify(sha, data.__getitem__)

        for conclusion in ["skipped", "failure"]:
            with self.subTest(scheduled=conclusion):
                data = copy.deepcopy(scheduled)
                for step in data[jobs_url][0]["jobs"][0]["steps"]:
                    step["conclusion"] = conclusion
                with self.assertRaises(ValueError):
                    proof.verify(sha, data.__getitem__)

        for count, runs in [(0, []), (2, [run]), (1000, [run]), (2, [run, run])]:
            with self.subTest(count=count, runs=len(runs)):
                data = copy.deepcopy(fixture)
                data[runs_url] = [{"total_count": count, "workflow_runs": runs}]
                with self.assertRaises(ValueError):
                    proof.verify(sha, data.__getitem__)

        # Include all pages and statuses: a later failure or rerun supersedes green.
        older = dict(run, id=9, updated_at="2026-10-12T11:00:00Z", conclusion="failure")
        data = copy.deepcopy(fixture)
        data[runs_url] = [{"total_count": 2, "workflow_runs": [older]},
                          {"total_count": 2, "workflow_runs": [run]}]
        proof.verify(sha, data.__getitem__)
        for latest in [dict(run, id=11, conclusion="failure",
                            updated_at="2026-10-12T13:00:00Z"),
                       dict(older, run_attempt=3, status="in_progress",
                            updated_at="2026-10-12T13:00:00Z"),
                       dict(older, run_attempt=3, updated_at=run["updated_at"])]:
            with self.subTest(latest=latest):
                data[runs_url] = [{"total_count": 2, "workflow_runs": [run]},
                                  {"total_count": 2, "workflow_runs": [latest]}]
                with self.assertRaises(ValueError):
                    proof.verify(sha, data.__getitem__)

        for jobs in [[], [job, dict(job, id=101)]]:
            data = copy.deepcopy(fixture)
            data[jobs_url] = [{"total_count": len(jobs), "jobs": jobs}]
            with self.assertRaises(ValueError):
                proof.verify(sha, data.__getitem__)
        data = copy.deepcopy(fixture)
        data[jobs_url][0]["jobs"][0]["steps"].append(copy.deepcopy(job["steps"][0]))
        with self.assertRaises(ValueError):
            proof.verify(sha, data.__getitem__)

        for invalid_sha in ["main", "a" * 39, "a" * 40 + "&status=success"]:
            with self.assertRaises(ValueError):
                proof.verify(invalid_sha, fixture.__getitem__)

        def unavailable(endpoint):
            raise OSError("GitHub API unavailable")

        with self.assertRaises(OSError):
            proof.verify(sha, unavailable)

    def test_api_paginates_with_a_deadline_and_propagates_errors(self):
        pages = [{"total_count": 2, "workflow_runs": [1]},
                 {"total_count": 2, "workflow_runs": [2]}]
        with patch.object(proof.subprocess, "run") as run:
            run.return_value.stdout = json.dumps(pages)
            self.assertEqual(proof.github_api("repos/Yazelix/nova/actions/runs"), pages)
            args, kwargs = run.call_args
            self.assertIn("--paginate", args[0])
            self.assertIn("--slurp", args[0])
            self.assertIn("github.com", args[0])
            self.assertTrue(kwargs["check"])
            self.assertEqual(kwargs["timeout"], 60)
            for error in [subprocess.CalledProcessError(1, "gh"),
                          subprocess.TimeoutExpired("gh", 60)]:
                run.side_effect = error
                with self.assertRaises(subprocess.SubprocessError):
                    proof.github_api("repos/Yazelix/nova/actions/runs")

        # Exercise the real CLI so captured child stderr cannot hide the cause.
        with tempfile.TemporaryDirectory() as directory:
            gh = Path(directory) / "gh"
            gh.write_text("#!/bin/sh\n"
                          "printf '%s\\n' 'gh: Resource not accessible by integration (HTTP 403)' >&2\n"
                          "exit 1\n")
            gh.chmod(0o755)
            result = subprocess.run(
                [sys.executable, "-B", proof.__file__, "a" * 40],
                env={"PATH": directory}, capture_output=True, text=True, timeout=10,
            )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")
        self.assertIn("Darwin smoke proof rejected", result.stderr)
        self.assertIn("Resource not accessible by integration (HTTP 403)", result.stderr)


if __name__ == "__main__":
    unittest.main()
