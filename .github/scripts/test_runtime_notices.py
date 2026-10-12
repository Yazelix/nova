import hashlib
import io
import json
import os
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import runtime_notices


class RuntimeNoticesTests(unittest.TestCase):
    def archive(self, root, name, contents):
        path = root / ("a" * 32 + "-" + name + ".tar.gz")
        with tarfile.open(path, "w:gz") as archive:
            for member, content in contents.items():
                data = content.encode()
                info = tarfile.TarInfo(member)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
        return path

    def inputs(self, root):
        standard = self.archive(root, "rust-std-1.2.3-x86_64-unknown-linux-gnu", {
            "rust-std-1.2.3-x86_64-unknown-linux-gnu/" + name: name + " original\n"
            for name in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE"]})
        compiler = self.archive(root, "rustc-1.2.3-x86_64-unknown-linux-gnu", {
            "rustc-1.2.3-x86_64-unknown-linux-gnu/" + name: name + " original\n"
            for name in ["COPYRIGHT", "LICENSE-MIT", "LICENSE-APACHE",
                         "rustc/share/doc/rust/COPYRIGHT-library.html",
                         "rustc/share/doc/rust/licenses/LLVM-exception.txt"]})
        go = self.archive(root, "go1.2.3.src", {
            "go/LICENSE": "Go permission\n", "go/PATENTS": "Go patent grant\n",
            "go/src/vendor/example.org/pkg/LICENSE": "dependency attribution\n",
            "go/src/runtime/runtime.go": "not a notice\n"})
        graph = {"root.drv": {"env": {}, "inputs": {
            "drvs": {key: {} for key in ["std.drv", "compiler.drv", "go.drv"]}}}}
        for key, name, source in [("std", "rust-std-stable-2020-01-01", standard),
                                  ("compiler", "rustc-stable-2020-01-01", compiler),
                                  ("go", "go-1.2.3", go),
                                  ("unrelated", "go-9.9.9", root / "absent")]:
            graph[key + ".drv"] = {"env": {"name": name, "src": str(source)},
                                    "inputs": {"drvs": {}}}
        return graph, standard, compiler, go

    def test_pinned_notices_reproducibility_and_overwrite_refusal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            graph, standard, compiler, go = self.inputs(root)
            output, again = root / "notices.tar.gz", root / "again.tar.gz"
            with patch.object(runtime_notices, "STORE", root):
                for destination, mask in [(output, 0o022), (again, 0o077)]:
                    previous = os.umask(mask)
                    try:
                        runtime_notices.write(graph, "root.drv", destination)
                    finally:
                        os.umask(previous)
                self.assertEqual(output.read_bytes(), again.read_bytes())
                with self.assertRaises(FileExistsError):
                    runtime_notices.write(graph, "root.drv", output)
            with tarfile.open(output) as archive:
                manifest = json.load(archive.extractfile("SOURCE-MANIFEST.json"))
                self.assertEqual(manifest["redistribution_review"], "required")
                self.assertEqual({p["derivation"] for p in manifest["providers"]},
                                 {"std.drv", "go.drv"})
                self.assertEqual(len(manifest["notice_archives"]), 3)
                for record in manifest["notice_archives"]:
                    source = root / record["archive"]
                    self.assertEqual(record["sha256"], hashlib.sha256(source.read_bytes()).hexdigest())
                prefix = "sources/language-runtime-notices/" + go.name
                self.assertEqual(archive.extractfile(prefix + "/go/LICENSE").read(), b"Go permission\n")
                self.assertEqual(archive.extractfile(prefix + "/go/src/vendor/example.org/pkg/LICENSE").read(),
                                 b"dependency attribution\n")
                self.assertNotIn(prefix + "/go/src/runtime/runtime.go", archive.getnames())
                self.assertTrue(any(name.endswith("/COPYRIGHT-library.html") for name in archive.getnames()))
                self.assertTrue(any(name.endswith("/licenses/LLVM-exception.txt") for name in archive.getnames()))

    def test_missing_inputs_or_unsafe_notices_prevent_publication(self):
        for fault in ["compiler", "missing", "version", "notice", "escape"]:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                graph, standard, compiler, go = self.inputs(root)
                if fault == "compiler":
                    del graph["root.drv"]["inputs"]["drvs"]["compiler.drv"]
                elif fault == "missing":
                    standard.unlink()
                elif fault == "version":
                    graph["go.drv"]["env"]["name"] = "go-9.9.9"
                else:
                    go.unlink()
                    contents = {"go/LICENSE": "permission\n"}
                    if fault == "escape":
                        contents |= {"go/PATENTS": "patent grant\n", "../escape/LICENSE": "escape\n"}
                    self.archive(root, "go1.2.3.src", contents)
                output = root / "notices.tar.gz"
                with patch.object(runtime_notices, "STORE", root), self.assertRaises(ValueError):
                    runtime_notices.write(graph, "root.drv", output)
                self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
