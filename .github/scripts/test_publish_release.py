import copy
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import publish_release as release
import release_assets
import test_release_assets
import verify_darwin_smoke


class PublicationTests(unittest.TestCase):
    def candidate(self, directory):
        inputs = test_release_assets.ReleaseAssetsTests().candidate(directory)
        output = directory / "assets"
        manifest = release_assets.prepare(*inputs, "nova-v1.3.0", output)
        sha, tag = manifest["revision"], manifest["tag"]
        review = {"contract": "NOVA-RELEASE-ASSETS-001", "revision": sha,
                  "assets": release.review_hashes(manifest),
                  "redistribution_review": "approved", "installed_artifact_review": "approved",
                  "review": "Fixture review: original notices, build instructions and replacement evidence."}
        annotation = {"tag": tag, "object": {"type": "commit", "sha": sha},
                      "message": json.dumps(review), "tagger": {"name": "Reviewer"}}
        self.routes = {
            "": [{"id": 746026727, "full_name": "Yazelix/nova", "private": False}],
            "/git/ref/tags/" + tag: [{"object": {"type": "tag", "sha": "a" * 40}}],
            "/git/tags/" + "a" * 40: [annotation],
            "/compare/" + sha + "...main": [{"status": "ahead"}],
            "/immutable-releases": [{"enabled": True}],
            "/releases?per_page=100": [[]],
        }
        for index, (workflow, names) in enumerate(release.GATES.items(), 1):
            run = {"id": index, "workflow_id": index, "path": ".github/workflows/" + workflow,
                   "head_sha": sha, "event": "workflow_dispatch", "run_attempt": 1,
                   "updated_at": "2026-10-12T12:00:00Z", "status": "completed", "conclusion": "success",
                   "repository": {"id": 746026727, "full_name": "Yazelix/nova"},
                   "head_repository": {"id": 746026727, "full_name": "Yazelix/nova"}}
            self.routes[f"/actions/workflows/{workflow}"] = [{"id": index, "path": run["path"]}]
            self.routes[f"/actions/workflows/{index}/runs?head_sha={sha}&per_page=100"] = [
                {"total_count": 1, "workflow_runs": [run]}]
            self.routes[f"/actions/runs/{index}"] = [run]
            self.routes[f"/actions/runs/{index}/attempts/1/jobs?per_page=100"] = [
                {"total_count": len(names), "jobs": [dict(run, id=100 + n, name=name, run_id=index)
                                                       for n, name in enumerate(names)]}]
        return output, manifest, review

    def api(self, endpoint):
        return copy.deepcopy(self.routes[endpoint.removeprefix(release.REPO)])

    def guard(self):
        if self.routes["/immutable-releases"][0]["enabled"] is not True:
            raise ValueError("immutable releases are disabled")

    def test_guard_uses_only_the_dedicated_token_and_fails_without_it(self):
        with patch.dict(os.environ, {"GITHUB_ACTIONS": "true", "GH_TOKEN": "ordinary-token"}, clear=True):
            with patch.object(release, "github_api") as api:
                with self.assertRaises(ValueError):
                    release.immutable_enabled()
                api.assert_not_called()
                with patch.dict(os.environ, {"NOVA_RELEASE_GUARD_TOKEN": "guard-token"}):
                    api.return_value = [{"enabled": True}]
                    release.immutable_enabled()
                    api.assert_called_once_with(release.REPO + "/immutable-releases", token="guard-token")
                    api.return_value = [{"enabled": False}]
                    with self.assertRaises(ValueError):
                        release.immutable_enabled()
                self.assertEqual(os.environ["GH_TOKEN"], "ordinary-token")

    def test_api_credential_is_scoped_to_its_subprocess(self):
        with patch.dict(os.environ, {"GH_TOKEN": "ordinary-token"}, clear=True):
            with patch.object(verify_darwin_smoke.subprocess, "run") as command:
                command.return_value.stdout = '[{"enabled": true}]'
                self.assertEqual(verify_darwin_smoke.github_api("setting", token="guard-token"),
                                 [{"enabled": True}])
                self.assertEqual(command.call_args.kwargs["env"]["GH_TOKEN"], "guard-token")
                self.assertNotIn("guard-token", command.call_args.args[0])
                self.assertEqual(os.environ["GH_TOKEN"], "ordinary-token")
                verify_darwin_smoke.github_api("ordinary-endpoint")
                self.assertIsNone(command.call_args.kwargs["env"])

    def test_approval_binds_exact_bytes_and_refuses_mutation(self):
        with tempfile.TemporaryDirectory() as temporary:
            output, manifest, review = self.candidate(Path(temporary))
            release.approve(output, manifest["tag"], manifest["revision"], self.api)
            accepted = release.verify_assets(output)
            self.assertEqual(accepted["redistribution_review"], "approved")
            self.assertNotIn("publication", accepted)
            record = json.loads((output / "RELEASE-APPROVAL.json").read_text())
            self.assertEqual(record["approval"], review)
            self.assertIn(review["review"], (output / "SOURCE-DELIVERY.md").read_text())
            release.check_approval(output, manifest["tag"], manifest["revision"], self.api)
            (output / manifest["targets"][release.TARGET]["archive"]["name"]).write_bytes(b"changed")
            with self.assertRaises(ValueError):
                release.check_approval(output, manifest["tag"], manifest["revision"], self.api)

    def test_failed_remote_guards_never_change_review_assets(self):
        with tempfile.TemporaryDirectory() as temporary:
            output, manifest, _ = self.candidate(Path(temporary))
            original = {p.name: p.read_bytes() for p in output.iterdir()}
            mutations = [
                ("/releases?per_page=100", lambda p: p[0].append({"id": 12, "tag_name": manifest["tag"]})),
                ("/compare/" + manifest["revision"] + "...main", lambda p: p[0].update(status="diverged")),
                ("/git/ref/tags/" + manifest["tag"], lambda p: p[0]["object"].update(type="commit")),
                ("/git/tags/" + "a" * 40, lambda p: p[0]["object"].update(sha="b" * 40)),
                ("/git/tags/" + "a" * 40, lambda p: p[0].update(message=p[0]["message"].replace("approved", "required"))),
                ("/actions/runs/3/attempts/1/jobs?per_page=100", lambda p: p[0]["jobs"][0].update(conclusion="skipped")),
                ("/actions/workflows/2/runs?head_sha=" + manifest["revision"] + "&per_page=100",
                 lambda p: (p[0]["workflow_runs"].append(dict(p[0]["workflow_runs"][0], id=99,
                     updated_at="2026-10-12T13:00:00Z", conclusion="failure")), p[0].update(total_count=2))),
            ]
            for endpoint, mutate in mutations:
                with self.subTest(endpoint=endpoint):
                    saved = copy.deepcopy(self.routes[endpoint])
                    mutate(self.routes[endpoint])
                    with self.assertRaises(ValueError):
                        release.approve(output, manifest["tag"], manifest["revision"], self.api)
                    self.assertEqual({p.name: p.read_bytes() for p in output.iterdir()}, original)
                    self.routes[endpoint] = saved
            part = output / manifest["sources"]["parts"][0]["name"]
            part.write_bytes(b"wrong source")
            release_assets.checksums(output)
            with self.assertRaises(ValueError):
                release.approve(output, manifest["tag"], manifest["revision"], self.api)

    def test_upload_failure_keeps_draft_unpublished_and_retry_refuses_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            output, manifest, _ = self.candidate(Path(temporary))
            release.approve(output, manifest["tag"], manifest["revision"], self.api)
            bundle = Path(temporary) / "provenance.sigstore.json"
            bundle.write_text("fixture")
            calls = []

            def command(arguments, **kwargs):
                calls.append(arguments)
                if arguments[:3] == ["gh", "api", "--method"]:
                    request = json.loads(kwargs["input"])
                    self.assertTrue(request["draft"])
                    self.assertEqual(request["tag_name"], manifest["tag"])
                    draft = {"id": 42, "tag_name": manifest["tag"], "draft": True}
                    self.routes["/releases?per_page=100"][0].append(draft)
                    return type("Result", (), {"stdout": json.dumps(draft)})()
                if arguments[:3] == ["gh", "release", "upload"]:
                    raise OSError("interrupted upload")
                return type("Result", (), {"stdout": ""})()

            with patch.object(release.subprocess, "run", side_effect=command):
                with self.assertRaises(OSError):
                    release.publish(output, bundle, manifest["tag"], manifest["revision"], self.api, self.guard)
                self.assertFalse(any("PATCH" in call for call in calls))
                with self.assertRaises(ValueError):
                    release.publish(output, bundle, manifest["tag"], manifest["revision"], self.api, self.guard)
            self.assertEqual(self.routes["/releases?per_page=100"][0][0]["draft"], True)

    def test_publication_requires_exact_server_inventory_before_publish(self):
        for fault in [None, "server-bytes", "immutable-setting"]:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                output, manifest, _ = self.candidate(Path(temporary))
                release.approve(output, manifest["tag"], manifest["revision"], self.api)
                bundle = Path(temporary) / "provenance.sigstore.json"
                bundle.write_text("fixture")
                events = []

                def command(arguments, **kwargs):
                    if arguments[:3] == ["gh", "api", "--method"]:
                        method = arguments[3]
                        request = json.loads(kwargs["input"])
                        if method == "POST":
                            events.append("draft")
                            draft = {"id": 42, "tag_name": manifest["tag"], "draft": True}
                            self.routes["/releases?per_page=100"][0].append(draft)
                            response = draft
                        else:
                            events.append("publish")
                            self.assertFalse(request["draft"])
                            response = {"draft": False, "immutable": True, "html_url": "https://example.invalid/release"}
                        return type("Result", (), {"stdout": json.dumps(response)})()
                    if arguments[:3] == ["gh", "release", "upload"]:
                        events.append("upload")
                        self.assertNotIn("--clobber", arguments)
                        records = [{"id": i, "name": p.name, "size": p.stat().st_size,
                                    "digest": "sha256:" + release.digest(p)}
                                   for i, p in enumerate([*output.iterdir(), bundle])]
                        if fault == "server-bytes":
                            records[0]["size"] += 1
                        if fault == "immutable-setting":
                            self.routes["/immutable-releases"][0]["enabled"] = False
                        self.routes["/releases/42/assets?per_page=100"] = [records]
                    return type("Result", (), {"stdout": ""})()

                with patch.object(release.subprocess, "run", side_effect=command):
                    if fault:
                        with self.assertRaises(ValueError):
                            release.publish(output, bundle, manifest["tag"], manifest["revision"], self.api, self.guard)
                    else:
                        release.publish(output, bundle, manifest["tag"], manifest["revision"], self.api, self.guard)
                self.assertEqual(events, ["draft", "upload"] + ([] if fault else ["publish"]))

    def test_immutability_failure_prevents_writes(self):
        with tempfile.TemporaryDirectory() as temporary:
            output, manifest, _ = self.candidate(Path(temporary))
            release.approve(output, manifest["tag"], manifest["revision"], self.api)
            bundle = Path(temporary) / "provenance.sigstore.json"
            bundle.write_text("fixture")
            self.routes["/immutable-releases"][0]["enabled"] = False
            with patch.object(release.subprocess, "run") as command:
                with self.assertRaises(ValueError):
                    release.publish(output, bundle, manifest["tag"], manifest["revision"], self.api, self.guard)
                command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
