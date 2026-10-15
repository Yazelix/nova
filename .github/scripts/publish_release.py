"""NOVA-RELEASE-ASSETS-001: require reviewed bytes before immutable publication."""

import argparse
import datetime
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

from release_assets import checksums, digest, encode
from verify_darwin_smoke import github_api, listed, require_success

REPO = "repos/Yazelix/nova"
TARGET = "x86_64-unknown-linux-gnu"
WORKFLOW = "Yazelix/nova/.github/workflows/portable_release.yml"
TAG = r"nova-v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?"
GATES = {"ci.yml": ["linux"],
         "publish_nix_cache.yml": ["publish_x86_64_linux", "publish_aarch64_darwin"],
         "version_gate.yml": ["release-gate"]}


def review_hashes(manifest):
    return {"runtime": manifest["targets"][TARGET]["archive"]["sha256"],
            "sources": manifest["sources"]["sha256"],
            "language_runtime_notices": manifest["language_runtime_notices"]["sha256"]}


def verify_assets(directory):
    paths = list(directory.iterdir())
    if any(not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", p.name)
           or p.is_symlink() or not p.is_file() or p.stat().st_size >= 2 * 1024 ** 3 for p in paths):
        raise ValueError("unsafe release asset")
    expected = "".join(f"{digest(p)}  {p.name}\n" for p in sorted(paths) if p.name != "SHA256SUMS")
    if (directory / "SHA256SUMS").read_text() != expected:
        raise ValueError("release checksum inventory differs")
    manifest = json.loads((directory / "nova-release.json").read_text())
    if (type(manifest["schema_version"]) is not int or manifest["schema_version"] != 1
            or set(manifest["targets"]) != {TARGET} or manifest["targets"][TARGET]["variant"] != "no-rio"
            or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", manifest["version"])
            or manifest["tag"] != "nova-v" + manifest["version"]
            or not re.fullmatch(r"[0-9a-f]{40}", manifest["revision"])):
        raise ValueError("unsupported release identity")
    records = [manifest["targets"][TARGET]["archive"], manifest["language_runtime_notices"],
               *manifest["sources"]["parts"]]
    names = [record["name"] for record in records]
    files = {"nova-release.json", "SHA256SUMS", "SOURCE-DELIVERY.md", *names}
    if manifest["redistribution_review"] == "approved":
        files.add("RELEASE-APPROVAL.json")
    elif manifest["redistribution_review"] != "required":
        raise ValueError("unknown review state")
    if len(names) != len(set(names)) or {p.name for p in paths} != files:
        raise ValueError("unexpected or missing release asset")
    for record in records:
        path = directory / record["name"]
        if (record["size"] != path.stat().st_size or record["sha256"] != digest(path)
                or record["url"] != f"https://github.com/Yazelix/nova/releases/download/{manifest['tag']}/{path.name}"):
            raise ValueError("manifest and asset bytes differ")
    whole, size = hashlib.sha256(), 0
    for part in manifest["sources"]["parts"]:
        with (directory / part["name"]).open("rb") as stream:
            while block := stream.read(1024 * 1024):
                whole.update(block)
                size += len(block)
    if size != manifest["sources"]["size"] or whole.hexdigest() != manifest["sources"]["sha256"]:
        raise ValueError("source parts do not reconstruct the reviewed source archive")
    return manifest


def portable(tag, api):
    """Plain version tags mark Stable releases; only an approval annotation requests publication."""
    if not re.fullmatch(TAG, tag):
        raise ValueError("expected a Nova version tag")
    reference, = api(f"{REPO}/git/ref/tags/{tag}")
    if reference["object"]["type"] != "tag":
        return False
    record, = api(f"{REPO}/git/tags/{reference['object']['sha']}")
    try:
        message = json.loads(record["message"])
    except json.JSONDecodeError:
        return False
    return isinstance(message, dict) and message.get("contract") == "NOVA-RELEASE-ASSETS-001"


def annotation(tag, sha, api):
    if not re.fullmatch(TAG, tag):
        raise ValueError("expected a Nova version tag")
    reference, = api(f"{REPO}/git/ref/tags/{tag}")
    if reference["object"]["type"] != "tag":
        raise ValueError("publication requires an annotated tag")
    object_sha = reference["object"]["sha"]
    record, = api(f"{REPO}/git/tags/{object_sha}")
    approval = json.loads(record["message"])
    if (record["tag"] != tag or record["object"]["type"] != "commit" or record["object"]["sha"] != sha
            or approval["contract"] != "NOVA-RELEASE-ASSETS-001" or approval["revision"] != sha
            or approval["redistribution_review"] != "approved"
            or approval["installed_artifact_review"] != "approved"
            or not isinstance(approval["review"], str) or not approval["review"].strip()
            or set(approval["assets"]) != {"runtime", "sources", "language_runtime_notices"}
            or any(not re.fullmatch(r"[0-9a-f]{64}", h) for h in approval["assets"].values())):
        raise ValueError("annotation does not approve this exact revision and its review")
    return {"tag_object": object_sha, "tagger": record["tagger"], "approval": approval}


def gates(tag, sha, api, draft_id=None):
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("expected a full candidate SHA")
    repository, = api(REPO)
    comparison, = api(f"{REPO}/compare/{sha}...main")
    if (repository["id"] != 746026727 or repository["full_name"] != "Yazelix/nova"
            or repository["private"] or comparison["status"] not in {"ahead", "identical"}):
        raise ValueError("canonical accepted revision is required")
    for page in api(f"{REPO}/releases?per_page=100"):
        for item in page:
            if item["tag_name"] == tag and not (item["id"] == draft_id and item["draft"] is True):
                raise ValueError("release already exists; never overwrite or resume it automatically")
    for workflow, required in GATES.items():
        owner, = api(f"{REPO}/actions/workflows/{workflow}")
        path = ".github/workflows/" + workflow
        if owner["path"] != path:
            raise ValueError("unexpected gate workflow")
        runs = listed(f"{REPO}/actions/workflows/{owner['id']}/runs?head_sha={sha}", "workflow_runs", api)
        for run in runs:
            if (run["workflow_id"] != owner["id"] or run["path"] != path or run["head_sha"] != sha
                    or run["event"] not in {"push", "workflow_dispatch"}
                    or any(run[k]["id"] != 746026727 or run[k]["full_name"] != "Yazelix/nova"
                           for k in ["repository", "head_repository"])):
                raise ValueError("untrusted gate run")
        ordered = sorted(runs, key=lambda r: (datetime.datetime.fromisoformat(r["updated_at"]), r["id"]))
        if not ordered or (len(ordered) > 1 and ordered[-1]["updated_at"] == ordered[-2]["updated_at"]):
            raise ValueError("missing or ambiguous latest gate run")
        run = ordered[-1]
        require_success(run)
        jobs = listed(f"{REPO}/actions/runs/{run['id']}/attempts/{run['run_attempt']}/jobs", "jobs", api)
        for name in required:
            matching = [job for job in jobs if job["name"] == name]
            if len(matching) != 1 or any(matching[0][k] != v for k, v in {
                    "head_sha": sha, "run_id": run["id"], "run_attempt": run["run_attempt"]}.items()):
                raise ValueError("missing or mismatched required gate job")
            require_success(matching[0])
        refreshed, = api(f"{REPO}/actions/runs/{run['id']}")
        if any(refreshed[k] != run[k] for k in ["id", "run_attempt", "updated_at", "status", "conclusion",
                                                "workflow_id", "path", "head_sha"]):
            raise ValueError("gate run changed during verification")


def approve(directory, tag, sha, api=github_api):
    manifest = verify_assets(directory)
    record = annotation(tag, sha, api)
    gates(tag, sha, api)
    if (manifest["tag"] != tag or manifest["revision"] != sha
            or manifest["redistribution_review"] != "required"
            or review_hashes(manifest) != record["approval"]["assets"]):
        raise ValueError("reviewed hashes differ from prepared candidate")
    with (directory / "RELEASE-APPROVAL.json").open("xb") as stream:
        stream.write(encode(record))
    manifest["redistribution_review"] = "approved"
    (directory / "nova-release.json").write_bytes(encode(manifest))
    delivery = directory / "SOURCE-DELIVERY.md"
    delivery.write_text(delivery.read_text().replace("Publication and redistribution review remain pending.",
        "The annotated tag records the maintainer's approval in RELEASE-APPROVAL.json.")
        .replace("Corresponding-source/build instructions, prominent native-library notices and applicable "
                 "replacement/relink obligations require review before publication.",
                 "The maintainer's source/build, notice and replacement/relink review is recorded in RELEASE-APPROVAL.json.")
        + "\n## Maintainer source and notice review\n\n" + record["approval"]["review"] + "\n")
    checksums(directory)


def check_approval(directory, tag, sha, api=github_api, draft_id=None):
    manifest = verify_assets(directory)
    record = annotation(tag, sha, api)
    gates(tag, sha, api, draft_id)
    if (manifest["tag"] != tag or manifest["revision"] != sha or manifest["redistribution_review"] != "approved"
            or review_hashes(manifest) != record["approval"]["assets"]
            or json.loads((directory / "RELEASE-APPROVAL.json").read_text()) != record):
        raise ValueError("release approval changed or does not match")
    return manifest


def provenance(directory, bundle, sha, ref):
    for path in sorted(directory.iterdir()):
        subprocess.run(["gh", "attestation", "verify", str(path), "--bundle", str(bundle),
                        "--repo", "Yazelix/nova", "--signer-workflow", WORKFLOW,
                        "--source-digest", sha, "--signer-digest", sha, "--source-ref", ref,
                        "--deny-self-hosted-runners"], check=True, timeout=120)


def immutable_enabled():
    token = os.environ.get("NOVA_RELEASE_GUARD_TOKEN")
    if os.environ.get("GITHUB_ACTIONS") == "true" and not token:
        raise ValueError("missing dedicated read-only App token for immutable-release verification")
    setting, = github_api(f"{REPO}/immutable-releases", token=token)
    if setting["enabled"] is not True:
        raise ValueError("immutable releases must be enabled before publication")


def publish(directory, bundle, tag, sha, api=github_api, guard=immutable_enabled):
    guard()
    check_approval(directory, tag, sha, api)
    provenance(directory, bundle, sha, "refs/tags/" + tag)
    body = (directory / "SOURCE-DELIVERY.md").read_text()
    result = subprocess.run(["gh", "api", "--method", "POST", f"{REPO}/releases", "--input", "-"],
                            input=json.dumps({"tag_name": tag, "target_commitish": sha, "name": tag,
                                              "body": body, "draft": True, "make_latest": "false"}),
                            text=True, stdout=subprocess.PIPE, check=True, timeout=60)
    draft = json.loads(result.stdout)
    if draft["tag_name"] != tag or draft["draft"] is not True:
        raise ValueError("unexpected draft release")
    subprocess.run(["gh", "release", "upload", tag, *map(str, sorted(directory.iterdir())), str(bundle),
                    "--repo", "Yazelix/nova"], check=True, timeout=1800)
    check_approval(directory, tag, sha, api, draft["id"])
    uploaded = [asset for page in api(f"{REPO}/releases/{draft['id']}/assets?per_page=100") for asset in page]
    expected = {p.name: (p.stat().st_size, "sha256:" + digest(p)) for p in [*directory.iterdir(), bundle]}
    if (len(uploaded) != len(expected)
            or {a["name"]: (a["size"], a["digest"]) for a in uploaded} != expected):
        raise ValueError("uploaded assets differ; draft remains unpublished")
    guard()
    result = subprocess.run(["gh", "api", "--method", "PATCH", f"{REPO}/releases/{draft['id']}", "--input", "-"],
                            input=json.dumps({"draft": False, "make_latest": "false"}), text=True,
                            stdout=subprocess.PIPE, check=True, timeout=60)
    published = json.loads(result.stdout)
    print(published["html_url"], flush=True)
    if published["draft"] is not False or published["immutable"] is not True:
        raise ValueError("published release did not report immutable enforcement")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["classify", "preflight", "approve", "check", "publish", "verify", "provenance"])
    parser.add_argument("directory", type=Path)
    parser.add_argument("tag")
    parser.add_argument("sha")
    parser.add_argument("--bundle", type=Path)
    parser.add_argument("--ref")
    args = parser.parse_args()
    if args.operation == "classify":
        print("portable=" + str(portable(args.tag, github_api)).lower())
    elif args.operation == "preflight":
        annotation(args.tag, args.sha, github_api)
        gates(args.tag, args.sha, github_api)
    elif args.operation == "approve":
        approve(args.directory, args.tag, args.sha)
    elif args.operation == "check":
        check_approval(args.directory, args.tag, args.sha)
    elif args.operation == "verify":
        manifest = verify_assets(args.directory)
        if (manifest["tag"], manifest["revision"]) != (args.tag, args.sha):
            raise ValueError("unexpected downloaded candidate")
    elif args.operation == "provenance":
        provenance(args.directory, args.bundle, args.sha, args.ref)
    else:
        publish(args.directory, args.bundle, args.tag, args.sha)
