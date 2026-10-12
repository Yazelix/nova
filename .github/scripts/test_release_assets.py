import hashlib
import json
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import grammar_sources
import release_assets

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "packaging"))
import archive_metadata


class ReleaseAssetsTests(unittest.TestCase):
    def candidate(self, directory):
        root = directory / "installed"
        (root / "bin").mkdir(parents=True)
        (root / "bin/yzx").write_text("candidate program\n")
        (root / "bin/yzx").chmod(0o755)
        (root / "share/yazelix").mkdir(parents=True)
        (root / "share/yazelix/runtime_identity.json").write_text(
            json.dumps({"version": "1.3.0", "channel": "edge"}))
        archive_metadata.write(root, {"schema_version": 1, "revision": "a" * 40,
            "target": "x86_64-unknown-linux-gnu", "variant": "no-rio",
            "entrypoints": ["bin/yzx"], "compatibility": {"libc": "glibc", "minimum": "2.39"}})
        identity = archive_metadata.verify(root, True)
        archive = directory / "runtime.tar.gz"
        with tarfile.open(archive, "w:gz") as stream:
            stream.add(root, arcname=".")
        source = directory / "source"
        source.mkdir()
        (source / "LICENSE").write_text("original notice\n")
        sources, notices = directory / "sources.tar.gz", directory / "notices.tar.gz"
        grammar_sources.bundle({"schema_version": 1, "package": identity,
            "archive_derivation": "archive.drv", "redistribution_review": "required",
            "components": [{"name": "program"}], "build_graph": {"rust.drv": {}}},
            {source.name: source}, sources)
        grammar_sources.bundle({"schema_version": 1, "archive_derivation": "archive.drv",
            "redistribution_review": "required", "providers": [{"derivation": "rust.drv"}]},
            {source.name: source}, notices)
        return root, archive, sources, notices

    def test_preparation_binds_exact_archives_and_reassembled_sources(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            inputs = self.candidate(directory)
            with patch.object(release_assets, "PART_BYTES", 128):
                release_assets.prepare(*inputs, "nova-v1.3.0", directory / "assets")
            assets = directory / "assets"
            manifest = json.loads((assets / "nova-release.json").read_text())
            self.assertEqual(manifest["revision"], "a" * 40)
            self.assertEqual(manifest["redistribution_review"], "required")
            self.assertNotIn("publication", manifest)
            runtime = manifest["targets"]["x86_64-unknown-linux-gnu"]["archive"]
            self.assertEqual((assets / runtime["name"]).read_bytes(), inputs[1].read_bytes())
            self.assertEqual(runtime["url"], "https://github.com/Yazelix/nova/releases/download/nova-v1.3.0/" + runtime["name"])
            combined = b"".join((assets / part["name"]).read_bytes() for part in manifest["sources"]["parts"])
            self.assertEqual(combined, inputs[2].read_bytes())
            self.assertEqual(hashlib.sha256(combined).hexdigest(), manifest["sources"]["sha256"])
            sums = (assets / "SHA256SUMS").read_text().splitlines()
            self.assertEqual({line.split("  ")[1] for line in sums},
                             {p.name for p in assets.iterdir()} - {"SHA256SUMS"})
            for line in sums:
                digest, name = line.split("  ")
                self.assertEqual(archive_metadata.digest(assets / name), digest)
            with self.assertRaises(FileExistsError):
                release_assets.prepare(*inputs, "nova-v1.3.0", assets)

    def test_wrong_tag_payload_or_source_association_cannot_leave_assets(self):
        for fault in ["tag", "archive-bytes", "archive-link", "archive-mode", "source-identity",
                      "source-schema", "notice-derivation", "notice-provider", "notice-review"]:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                root, archive, sources, notices = self.candidate(directory)
                tag = "nova-v1.3.0"
                if fault == "tag":
                    tag = "nova-v1.3.1"
                elif fault.startswith("archive"):
                    if fault == "archive-bytes":
                        (root / "bin/yzx").write_text("unverified replacement\n")
                    elif fault == "archive-link":
                        (root / "bin/escape").symlink_to("/etc/passwd")
                    else:
                        (root / "bin/yzx").chmod(0o644)
                    with tarfile.open(archive, "w:gz") as stream:
                        stream.add(root, arcname=".")
                    if fault == "archive-bytes":
                        (root / "bin/yzx").write_text("candidate program\n")
                    elif fault == "archive-link":
                        (root / "bin/escape").unlink()
                    else:
                        (root / "bin/yzx").chmod(0o755)
                else:
                    target = sources if fault.startswith("source") else notices
                    metadata = grammar_sources.verify_bundle(target)
                    target.unlink()
                    if fault == "source-identity": metadata["package"]["revision"] = "b" * 40
                    elif fault == "source-schema": metadata["schema_version"] = True
                    elif fault == "notice-derivation": metadata["archive_derivation"] = "wrong.drv"
                    elif fault == "notice-provider": metadata["providers"] = [{"derivation": "foreign.drv"}]
                    else: metadata["redistribution_review"] = "cleared"
                    grammar_sources.bundle(metadata, {"source": directory / "source"}, target)
                output = directory / "assets"
                with self.assertRaises((ValueError, KeyError)):
                    release_assets.prepare(root, archive, sources, notices, tag, output)
                self.assertFalse(output.exists())

    def test_concurrent_destination_file_is_never_replaced(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            inputs = self.candidate(directory)
            output = directory / "assets"
            original = Path.mkdir
            target = output / "yazelix-nova-1.3.0-x86_64-unknown-linux-gnu-no-rio.tar.gz"

            def concurrent_file(path, *args, **kwargs):
                result = original(path, *args, **kwargs)
                if path == output:
                    target.write_text("concurrent work\n")
                return result

            with patch.object(Path, "mkdir", concurrent_file):
                with self.assertRaises(FileExistsError):
                    release_assets.prepare(*inputs, "nova-v1.3.0", output)
            self.assertEqual(target.read_text(), "concurrent work\n")


if __name__ == "__main__":
    unittest.main()
