import hashlib
import json
import os
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import grammar_sources


class GrammarSourcesTests(unittest.TestCase):
    def test_exact_sources_notices_and_reproducible_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            contents = {"LICENSES/MIT.txt": "permission notice\n",
                        "src/parser.c": "/* original attribution */\n",
                        "README.md": "License: MIT\n",
                        "package.json": '{"license":"ISC"}\n'}
            for name, content in contents.items():
                file = source / name
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text(content)
            (source / "notice-link").symlink_to("LICENSES/MIT.txt")
            reentered = os.path.relpath("/", source) + (source / "LICENSES/MIT.txt").as_posix()
            (source / "reentered-notice").symlink_to(reentered)
            revision = "a" * 40
            config = {"grammar": [{"name": name, "source": {
                "git": "https://example.org/grammars", "rev": revision,
                "subpath": name}} for name in ["one", "two"]]}
            graph = {"root.drv": {"env": {"name": "archive"}, "inputs": {
                "drvs": {name + ".drv": {} for name in ["one", "two"]}}}}
            for name in ["one", "two", "unrelated"]:
                graph[name + ".drv"] = {"env": {
                    "name": f"helix-tree-sitter-{name}-{revision}", "src": str(source)},
                    "inputs": {"drvs": {}}, "outputs": {"out": {"path": name}}}
            arguments = (graph, "root.drv", config, "b" * 40)
            with patch.object(grammar_sources, "STORE", root):
                manifest, sources = grammar_sources.inventory(*arguments)
                legacy = {"/nix/store/" + k: {"env": v["env"], "inputDrvs": {
                    "/nix/store/" + child: ["out"] for child in v["inputs"]["drvs"]}}
                    for k, v in graph.items()}
                old_manifest, old_sources = grammar_sources.inventory(
                    legacy, "/nix/store/root.drv", config, "b" * 40)
                self.assertEqual((old_manifest, old_sources), (manifest, sources))
                structured = graph | {"two.drv": {
                    "env": {}, "name": graph["two.drv"]["env"]["name"],
                    "structuredAttrs": {"src": str(source)}, "inputs": {"drvs": {}}}}
                self.assertEqual(grammar_sources.inventory(
                    structured, "root.drv", config, "b" * 40), (manifest, sources))
            self.assertEqual([g["name"] for g in manifest["grammars"]], ["one", "two"])
            self.assertEqual(len(sources), 1)
            first, second = root / "first.tar.gz", root / "second.tar.gz"
            grammar_sources.bundle(manifest, sources, first)
            grammar_sources.bundle(manifest, sources, second)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with tarfile.open(first) as archive:
                saved = json.load(archive.extractfile("SOURCE-MANIFEST.json"))
                self.assertEqual(saved["revision"], "b" * 40)
                for name, content in contents.items():
                    member = "sources/source/" + name
                    self.assertEqual(archive.extractfile(member).read().decode(), content)
                self.assertTrue(archive.getmember("sources/source/notice-link").issym())
                self.assertEqual(archive.getmember("sources/source/reentered-notice").linkname,
                                 "LICENSES/MIT.txt")
                self.assertEqual(saved["sources"][0]["symlinks"]["reentered-notice"], {
                    "target": reentered, "archived": True, "archive_target": "LICENSES/MIT.txt"})
                self.assertEqual(saved["sources"][0]["files"]["LICENSES/MIT.txt"],
                                 hashlib.sha256(contents["LICENSES/MIT.txt"].encode()).hexdigest())
                self.assertEqual(archive.getmember("sources/source/src/parser.c").mtime, 1)
                archive.extractall(root / "unpacked", filter="data")
            self.assertEqual((root / "unpacked/sources/source/reentered-notice").read_text(),
                             contents["LICENSES/MIT.txt"])
            with self.assertRaises(FileExistsError):
                grammar_sources.bundle(manifest, sources, first)
            stale = {"grammar": [{"name": "one", "source": {"rev": "c" * 40}}]}
            with self.assertRaisesRegex(ValueError, "source pin"):
                grammar_sources.inventory(graph, "root.drv", stale, "b" * 40)
            with self.assertRaisesRegex(ValueError, "grammar source"):
                grammar_sources.inventory(*arguments)
            (source / "escape").symlink_to(root)
            (source / "dangling").symlink_to("absent")
            (root / "private").write_text("outside selected source")
            safe_archive = root / "safe.tar.gz"
            grammar_sources.bundle(manifest, sources, safe_archive)
            with tarfile.open(safe_archive) as archive:
                saved = json.load(archive.extractfile("SOURCE-MANIFEST.json"))
                self.assertEqual(saved["sources"][0]["symlinks"]["escape"],
                                 {"target": str(root), "archived": False})
                self.assertFalse(saved["sources"][0]["symlinks"]["dangling"]["archived"])
                self.assertNotIn("sources/source/escape", archive.getnames())
                self.assertNotIn("sources/source/private", archive.getnames())
            other = root / "other-source"
            other.mkdir()
            (other / "LICENSE").write_text("dependency permission\n")
            (source / "dependency-notice").symlink_to(other / "LICENSE")
            combined = root / "combined.tar.gz"
            grammar_sources.bundle(manifest, sources | {other.name: other}, combined)
            with tarfile.open(combined) as archive:
                link = archive.getmember("sources/source/dependency-notice")
                self.assertEqual(link.linkname, "../other-source/LICENSE")
                self.assertEqual(archive.extractfile("sources/other-source/LICENSE").read(),
                                 b"dependency permission\n")
                saved = json.load(archive.extractfile("SOURCE-MANIFEST.json"))
                record = next(s for s in saved["sources"] if s["path"] == "sources/source")
                self.assertEqual(record["symlinks"]["dependency-notice"], {
                    "target": str(other / "LICENSE"), "archived": True,
                    "archive_target": "../other-source/LICENSE"})

    def test_legacy_nix_graph_and_missing_inputs_fail_closed(self):
        legacy = {"/nix/store/root.drv": {"env": {"name": "archive"},
                   "inputDrvs": {"/nix/store/missing.drv": ["out"]}, "outputs": {}}}
        with self.assertRaisesRegex(ValueError, "missing derivation"):
            grammar_sources.inventory(legacy, "/nix/store/root.drv", {}, "a" * 40)
        with self.assertRaisesRegex(ValueError, "revision"):
            grammar_sources.inventory(legacy, "/nix/store/root.drv", {}, "edge")


if __name__ == "__main__":
    unittest.main()
