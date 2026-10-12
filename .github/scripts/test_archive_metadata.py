import hashlib
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "packaging"))
import archive_metadata
import grammar_sources


class ArchiveMetadataTests(unittest.TestCase):
    def payload(self, root):
        (root / "bin").mkdir()
        (root / "bin/yzx").write_bytes(b"native launcher")
        (root / "bin/yzx").chmod(0o755)
        (root / "share/yazelix").mkdir(parents=True)
        (root / "share/yazelix/runtime_identity.json").write_text(
            json.dumps({"version": "1.3.0", "channel": "edge", "rio_revision": "none"}))
        (root / "share/notice").write_text("original notice\n")
        (root / "share/notice").chmod(0o444)
        (root / "notice-link").symlink_to("share/notice")
        return {"schema_version": 1, "revision": "a" * 40,
                "target": "x86_64-unknown-linux-gnu", "variant": "no-rio",
                "entrypoints": ["bin/yzx"], "compatibility": {"libc": "glibc"}}

    @unittest.skipIf(os.geteuid() == 0, "root can read permission-denied directories")
    def test_unreadable_trees_fail_before_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "payload"
            root.mkdir()
            identity = self.payload(root)
            hidden = root / "hidden"
            hidden.mkdir()
            (hidden / "LICENSE").write_text("required notice\n")
            output = Path(temporary) / "sources.tar.gz"
            hidden.chmod(0)
            try:
                actions = {"metadata": lambda: archive_metadata.write(root, identity),
                           "sources": lambda: grammar_sources.bundle({}, {root.name: root}, output)}
                for name, action in actions.items():
                    with self.subTest(tool=name), self.assertRaises(PermissionError):
                        action()
                self.assertFalse((root / archive_metadata.PACKAGE).exists())
                self.assertFalse((root / archive_metadata.INDEX).exists())
                self.assertFalse(output.exists())
            finally:
                hidden.chmod(0o755)

    def test_identity_integrity_and_installed_tree_mutations(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            identity = self.payload(root)
            archive_metadata.write(root, identity)
            metadata = archive_metadata.verify(root)
            self.assertEqual(metadata["revision"], "a" * 40)
            self.assertEqual((metadata["version"], metadata["channel"]), ("1.3.0", "edge"))
            saved = json.loads((root / metadata["file_index"]["path"]).read_text())
            self.assertEqual(saved["files"]["share/notice"], {
                "type": "file", "mode": 0o444,
                "sha256": hashlib.sha256(b"original notice\n").hexdigest()})
            self.assertEqual(saved["files"]["notice-link"], {
                "type": "symlink", "target": "share/notice"})
            with self.assertRaises(FileExistsError):
                archive_metadata.write(root, identity)
            for mutation in ["content", "mode", "extra", "link", "index"]:
                with self.subTest(mutation=mutation):
                    file = root / "share/notice"
                    if mutation == "content":
                        file.chmod(0o644)
                        file.write_text("changed notice\n")
                    elif mutation == "mode":
                        file.chmod(0o644)
                    elif mutation == "extra":
                        (root / "extra").write_text("unexpected payload")
                    elif mutation == "link":
                        (root / "notice-link").unlink()
                        (root / "notice-link").symlink_to("bin/yzx")
                    else:
                        (root / metadata["file_index"]["path"]).write_text("{}")
                    with self.assertRaises(ValueError):
                        archive_metadata.verify(root)
                    if mutation == "extra":
                        (root / "extra").unlink()
                    elif mutation == "link":
                        (root / "notice-link").unlink()
                        (root / "notice-link").symlink_to("share/notice")
                    file.chmod(0o644)
                    file.write_text("original notice\n")
                    file.chmod(0o444)
                    (root / metadata["file_index"]["path"]).write_text(
                        archive_metadata.encode(saved).decode())
            archive_metadata.verify(root)
            for field in ["schema_version", "mode"]:
                with self.subTest(field=field):
                    malformed = json.loads(archive_metadata.encode(saved))
                    if field == "schema_version":
                        malformed[field] = True
                    else:
                        malformed["files"]["share/notice"][field] = float(0o444)
                    (root / metadata["file_index"]["path"]).write_bytes(archive_metadata.encode(malformed))
                    metadata["file_index"]["sha256"] = archive_metadata.digest(
                        root / metadata["file_index"]["path"])
                    (root / "share/yazelix/package.json").write_bytes(archive_metadata.encode(metadata))
                    with self.assertRaises(ValueError):
                        archive_metadata.verify(root)

    def test_invalid_payload_and_unpinned_development_identity(self):
        for fault in ["escape", "absolute", "dangling", "fifo", "setuid", "revision", "entrypoint", "schema"]:
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                identity = self.payload(root)
                if fault in ["escape", "absolute", "dangling"]:
                    target = {"escape": "../outside", "absolute": "/etc/passwd",
                              "dangling": "absent"}[fault]
                    (root / "bad-link").symlink_to(target)
                elif fault == "fifo":
                    os.mkfifo(root / "fifo")
                elif fault == "setuid":
                    (root / "bin/yzx").chmod(0o4755)
                elif fault == "revision":
                    identity["revision"] = "edge"
                elif fault == "schema":
                    identity["schema_version"] = True
                else:
                    identity["entrypoints"] = ["../yzx"]
                with self.assertRaises(ValueError):
                    archive_metadata.write(root, identity)
                self.assertFalse((root / "share/yazelix/package.json").exists())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            identity = self.payload(root) | {"revision": None}
            with self.assertRaisesRegex(ValueError, "immutable"):
                archive_metadata.write(root, identity, release=True)
            self.assertFalse((root / "share/yazelix/package.json").exists())
            archive_metadata.write(root, identity)
            self.assertIsNone(archive_metadata.verify(root)["revision"])
            with self.assertRaisesRegex(ValueError, "immutable"):
                archive_metadata.verify(root, release=True)


if __name__ == "__main__":
    unittest.main()
