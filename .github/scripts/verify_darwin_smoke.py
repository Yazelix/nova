"""NOVA-DARWIN-SMOKE-REUSE-001: consume native proof, never rebuild it."""

import datetime
import json
import re
import subprocess
import sys


BASE = "repos/Yazelix/nova/actions"
WORKFLOW = ".github/workflows/darwin_package_smoke.yml"
NATIVE_STEPS = (
    "Build Yazi PTY checks",
    "Build remaining aarch64-darwin packages and checks",
)


def github_api(endpoint):
    result = subprocess.run(
        ["gh", "api", "--hostname", "github.com", "--paginate", "--slurp", endpoint],
        check=True, stdout=subprocess.PIPE, text=True, timeout=60,
    )
    return json.loads(result.stdout)


def listed(endpoint, key, api):
    pages = api(endpoint + "&per_page=100" if "?" in endpoint else endpoint + "?per_page=100")
    items = [item for page in pages for item in page[key]]
    # GitHub's filtered run searches expose at most 1,000 results.
    if (not pages or len(items) >= 1000
            or any(page["total_count"] != len(items) for page in pages)
            or len({item["id"] for item in items}) != len(items)):
        raise ValueError("Incomplete or ambiguous GitHub API pagination")
    return items


def require_success(record):
    if record["status"] != "completed" or record["conclusion"] != "success":
        raise ValueError(f"Native proof did not succeed: {record.get('name', 'workflow run')}")


def check_run(run, workflow_id, sha):
    if (run["workflow_id"] != workflow_id or run["path"] != WORKFLOW
            or run["head_sha"] != sha or run["event"] not in {"workflow_dispatch", "schedule"}
            or type(run["id"]) is not int or run["id"] <= 0
            or type(run["run_attempt"]) is not int or run["run_attempt"] <= 0
            or any(run[key]["id"] != 746026727 or run[key]["full_name"] != "Yazelix/nova"
                   for key in ["repository", "head_repository"])):
        raise ValueError("Untrusted workflow, repository, revision or run attempt")
    updated = datetime.datetime.fromisoformat(run["updated_at"])
    if updated.utcoffset() is None:
        raise ValueError("Run update time has no timezone")
    return updated, run["id"]


def verify(sha, api=github_api):
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("Expected a full candidate commit SHA")
    workflow, = api(f"{BASE}/workflows/darwin_package_smoke.yml")
    if workflow["path"] != WORKFLOW or type(workflow["id"]) is not int:
        raise ValueError("Unexpected Darwin smoke workflow identity")
    runs = listed(f"{BASE}/workflows/{workflow['id']}/runs?head_sha={sha}", "workflow_runs", api)
    if not runs:
        raise ValueError("No exact-revision Darwin smoke run")
    # Include failed/incomplete runs; a rerun can update an older run ID.
    ordered = sorted((check_run(item, workflow["id"], sha), item) for item in runs)
    if len(ordered) > 1 and ordered[-1][0][0] == ordered[-2][0][0]:
        raise ValueError("Latest run update order is ambiguous; dispatch a fresh smoke run")
    run = ordered[-1][1]
    require_success(run)
    jobs = listed(f"{BASE}/runs/{run['id']}/attempts/{run['run_attempt']}/jobs", "jobs", api)
    if len(jobs) != 1:
        raise ValueError("Expected one native Darwin smoke job")
    job, = jobs
    if (job["name"] != "macos-package-smoke" or "macos-15" not in job["labels"]
            or job["run_id"] != run["id"] or job["run_attempt"] != run["run_attempt"]
            or job["head_sha"] != sha):
        raise ValueError("Unexpected native job or run attempt")
    require_success(job)
    for name in NATIVE_STEPS:
        steps = [step for step in job["steps"] if step["name"] == name]
        if len(steps) != 1:
            raise ValueError(f"Missing or ambiguous native step: {name}")
        require_success(steps[0])

    refreshed, = api(f"{BASE}/runs/{run['id']}")
    check_run(refreshed, workflow["id"], sha)
    require_success(refreshed)
    if (refreshed["id"] != run["id"] or refreshed["run_attempt"] != run["run_attempt"]
            or refreshed["updated_at"] != run["updated_at"]):
        raise ValueError("Native run changed during verification; retry after it settles")
    return (f"Accepted Darwin smoke: https://github.com/Yazelix/nova/actions/runs/{run['id']}\n"
            f"Candidate: {sha}; attempt {run['run_attempt']}\n"
            f"Native steps: {'; '.join(NATIVE_STEPS)}")


if __name__ == "__main__":
    try:
        if len(sys.argv) != 2:
            raise ValueError("Usage: verify_darwin_smoke.py FULL_CANDIDATE_SHA")
        print(verify(sys.argv[1]))
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"Darwin smoke proof rejected: {error}\n"
              "Complete Darwin Package Smoke for this exact SHA before dispatching Version Gate.\n"
              "Dispatch: gh workflow run darwin_package_smoke.yml --repo Yazelix/nova --ref CHANNEL\n"
              "Use the channel pointing at the candidate and verify the dispatched head SHA.", file=sys.stderr)
        sys.exit(1)
