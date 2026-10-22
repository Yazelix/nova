import hashlib
import io
import json
import os
import pty
import re
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

INSTALLER = Path(__file__).resolve().parents[2] / "packaging/install.sh"
RELEASES = "https://github.com/Yazelix/nova/releases"
NAME = "yazelix-nova-{}-x86_64-unknown-linux-gnu-no-rio.tar.gz"
CURL = """#!/bin/sh
out='' url=''
while [ $# -gt 0 ]; do
    case $1 in --output) out=$2; shift 2 ;; -*) shift ;; *) url=$1; shift ;; esac
done
printf '%s\\n' "$url" >> "$FIXTURE/requests"
case $url in
    {releases}/latest/download/*) file=$FIXTURE/$(cat "$FIXTURE/latest")/${{url##*/}} ;;
    {releases}/download/nova-v*) version=${{url#{releases}/download/nova-v}}; file=$FIXTURE/${{version%%/*}}/${{url##*/}} ;;
    *) exit 6 ;;
esac
[ -f "$file" ] || exit 22
cat "$file" > "$out"
""".format(releases=RELEASES)


class InstallerTests(unittest.TestCase):
    """NOVA-INSTALL-001 against local fake releases: no network, no Nix."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name)
        self.fixture, self.tools, self.home = self.base / "releases", self.base / "tools", self.base / "home dir"
        for path in [self.fixture, self.tools, self.home]:
            path.mkdir()
        (self.tools / "curl").write_text(CURL)
        (self.tools / "curl").chmod(0o755)
        self.root, self.bin = self.home / ".local/share/yazelix-nova", self.home / ".local/bin"

    @staticmethod
    def archive(version, program=None, packaged=None):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:gz") as stream:
            for name, mode, data in [
                    ("bin/yzx", 0o555, program or f"#!/bin/sh\necho 'Yazelix Nova ({version})'\n"),
                    ("share/yazelix/package.json", 0o444,
                     json.dumps({"target": "x86_64-unknown-linux-gnu", "version": packaged or version}, indent=2) + "\n")]:
                info = tarfile.TarInfo("./" + name)
                info.mode, info.size = mode, len(data.encode())
                stream.addfile(info, io.BytesIO(data.encode()))
            for name in ["./", "./bin/", "./share/", "./share/yazelix/"]:
                info = tarfile.TarInfo(name)
                info.type, info.mode = tarfile.DIRTYPE, 0o555
                stream.addfile(info)
        return buffer.getvalue()

    def release(self, version, program=None, packaged=None, minimum="2.39", archive=None, size=None):
        directory = self.fixture / version
        directory.mkdir()
        archive = self.archive(version, program, packaged) if archive is None else archive
        files = {NAME.format(version): archive,
                 NAME.format(version).replace(".tar.gz", "-language-runtime-notices.tar.gz"): b"notices",
                 "nova-release.json": json.dumps({"targets": {"x86_64-unknown-linux-gnu": {
                     "archive": {"name": NAME.format(version), "size": len(archive) if size is None else size},
                     "compatibility": {"libc": {"family": "glibc", "minimum": minimum}}}}}, indent=2).encode()}
        for name, data in files.items():
            (directory / name).write_bytes(data)
        (directory / "SHA256SUMS").write_text(
            "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in sorted(files.items())))
        (self.fixture / "latest").write_text(version)
        return directory

    def run_installer(self, *arguments, script=None, environment=None, stdout=subprocess.PIPE, shell=("sh",)):
        env = {"PATH": f"{self.tools}:/usr/bin:/bin", "HOME": str(self.home), "SHELL": "/bin/bash",
               "FIXTURE": str(self.fixture), **(environment or {})}
        return subprocess.run([*shell, "-s", "--", *arguments], input=INSTALLER.read_text() if script is None else script,
                              env=env, cwd=self.base, text=True, stdout=stdout, stderr=subprocess.PIPE, timeout=60)

    def installed(self):
        link = self.bin / "yzx"
        return subprocess.run([link, "--version"], text=True, capture_output=True, check=True).stdout.strip()

    def test_install_upgrade_rollback_and_uninstall_keep_ownership(self):
        self.release("1.4.0")
        result = self.run_installer()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.installed(), "Yazelix Nova (1.4.0)")
        self.assertEqual(os.readlink(self.root / "current"), "versions/1.4.0")
        self.assertEqual(result.stdout, f"""Yazelix Nova 1.4.0
Downloading 1 MB

✓ Installed Yazelix Nova 1.4.0

  Start   ~/.local/bin/yzx enter          opens Nova in this terminal
          To run plain yzx, add this line to ~/.bashrc:
          export PATH="{self.bin}:$PATH"
  Learn   ~/.local/bin/yzx tutor begin    run it inside Nova
  Docs    https://nova.yazelix.com/docs/
  Help    ~/.local/bin/yzx doctor

  Files in ~/.local/share/yazelix-nova · yzx linked in ~/.local/bin
""")
        self.assertEqual(sorted(p.name for p in self.home.iterdir()), [".local"])
        requests = (self.fixture / "requests").read_text().splitlines()
        self.assertEqual(requests[0], f"{RELEASES}/latest/download/SHA256SUMS")
        self.assertIn(f"{RELEASES}/download/nova-v1.4.0/{NAME.format('1.4.0')}", requests)

        self.release("2.0.0")
        result = self.run_installer(environment={"PATH": f"{self.tools}:{self.bin}:/usr/bin:/bin"})
        self.assertEqual(result.stdout, """Yazelix Nova 2.0.0
Downloading 1 MB

✓ Installed Yazelix Nova 2.0.0

  Start   yzx enter          opens Nova in this terminal
  Learn   yzx tutor begin    run it inside Nova
  Docs    https://nova.yazelix.com/docs/
  Help    yzx doctor

  Files in ~/.local/share/yazelix-nova · yzx linked in ~/.local/bin
""")
        self.assertEqual(self.installed(), "Yazelix Nova (2.0.0)")
        self.assertTrue((self.root / "versions/1.4.0/bin/yzx").exists())
        result = self.run_installer("--version", "1.4.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.installed(), "Yazelix Nova (1.4.0)")
        self.assertEqual(sorted(p.name for p in self.root.iterdir()),
                         [".yazelix-nova-installer", "current", "versions"])

        (self.bin / "other").write_text("user tool\n")
        config = self.home / ".config/yazelix"
        config.mkdir(parents=True)
        result = self.run_installer("--uninstall")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "✓ Removed Yazelix Nova\n\n  Your configuration and state are untouched\n")
        self.assertFalse(self.root.exists() or (self.bin / "yzx").is_symlink())
        self.assertTrue((self.bin / "other").exists() and config.is_dir())

    def test_failures_leave_the_selected_installation_running(self):
        self.release("1.4.0")
        self.assertEqual(self.run_installer().returncode, 0)
        cases = {
            "checksum": ({}, "does not match its published SHA-256"),
            "extraction": ({"archive": b"not a gzip archive"}, "could not extract"),
            "identity": ({"packaged": "9.9.9"}, "does not contain Nova 2.0.2"),
            "smoke": ({"program": "#!/bin/sh\necho 'missing library' >&2\nexit 127\n"}, "missing library"),
            "glibc": ({"minimum": "99.0"}, "needs glibc 99.0 or newer"),
            "compatibility": ({"minimum": "any"}, "no glibc requirement"),
        }
        for number, (fault, (options, message)) in enumerate(cases.items()):
            with self.subTest(fault=fault):
                version = f"2.0.{number}"
                directory = self.release(version, **options)
                if fault == "checksum":
                    (directory / NAME.format(version)).write_bytes(self.archive(version, "#!/bin/sh\necho tampered\n"))
                (self.fixture / "requests").unlink()
                result = self.run_installer()
                self.assertEqual(result.returncode, 1, result.stdout)
                self.assertIn(message, result.stderr)
                self.assertEqual(self.installed(), "Yazelix Nova (1.4.0)")
                self.assertFalse((self.root / "versions" / version).exists())
                self.assertEqual([p.name for p in self.root.glob(".install.*")], [])
                if fault in {"glibc", "compatibility"}:
                    self.assertNotIn(NAME.format(version), (self.fixture / "requests").read_text())

    def test_refusals_touch_nothing(self):
        self.release("1.4.0")
        darwin = self.tools / "uname"
        darwin.write_text('#!/bin/sh\ncase $1 in -s) echo Darwin ;; -m) echo arm64 ;; esac\n')
        darwin.chmod(0o755)
        result = self.run_installer()
        self.assertIn("install Nova with Nix", result.stderr)
        darwin.unlink()

        self.bin.mkdir(parents=True)
        (self.bin / "yzx").symlink_to("/nix/profile/bin/yzx")
        self.assertIn("is not this installer's link (it points to /nix/profile/bin/yzx)", self.run_installer().stderr)
        (self.bin / "yzx").unlink()
        foreign = self.base / "documents"
        foreign.mkdir()
        (foreign / "notes.txt").write_text("keep\n")
        self.assertIn("not created by this installer", self.run_installer("--install-root", str(foreign)).stderr)
        self.assertIn("not created by this installer", self.run_installer("--uninstall", "--install-root", str(foreign)).stderr)
        self.assertEqual([p.name for p in foreign.iterdir()], ["notes.txt"])
        self.assertIn("--version expects", self.run_installer("--version", "latest; rm -rf /").stderr)
        self.assertFalse(self.root.exists())
        self.assertFalse((self.fixture / "requests").exists())

    def test_truncated_download_executes_nothing(self):
        script, call = INSTALLER.read_text().removesuffix("\n").rsplit("\n", 1)
        self.assertEqual(call, 'main "$@"')
        self.release("1.4.0")
        for cut in [*range(0, len(script), max(1, len(script) // 25)), len(script)]:
            with self.subTest(cut=cut):
                self.run_installer(script=script[:cut])
                self.assertFalse((self.home / ".local").exists())
                self.assertFalse((self.fixture / "requests").exists())

    def test_custom_roots_with_spaces_and_shell_guidance(self):
        self.release("1.4.0")
        root, bin = self.base / "nova moved/root", self.base / "my bin"
        result = self.run_installer("--install-root", str(root), "--bin-dir", str(bin), environment={"SHELL": "/usr/bin/nu"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"$env.PATH = ($env.PATH | prepend '{bin}')", result.stdout)
        self.assertEqual(os.readlink(bin / "yzx"), f"{root}/current/bin/yzx")
        on_path = self.run_installer("--install-root", str(root), "--bin-dir", str(bin),
                                     environment={"PATH": f"{self.tools}:{bin}:/usr/bin:/bin"})
        self.assertEqual(on_path.returncode, 0, on_path.stderr)
        self.assertNotIn("To run plain yzx", on_path.stdout)

    def test_names_another_yzx_on_path(self):
        self.release("1.4.0")
        nix = self.base / "nix bin"
        nix.mkdir()
        (nix / "yzx").write_text("#!/bin/sh\n")
        (nix / "yzx").chmod(0o755)
        ahead = self.run_installer(environment={"PATH": f"{self.tools}:{self.bin}:{nix}:/usr/bin:/bin"})
        self.assertIn(f"  Start   yzx enter          opens Nova in this terminal\n"
                      f"          yzx runs this install, ahead of {nix}/yzx\n", ahead.stdout)
        behind = self.run_installer(environment={"PATH": f"{self.tools}:{nix}:{self.bin}:/usr/bin:/bin"})
        self.assertIn(f"  Start   ~/.local/bin/yzx enter          opens Nova in this terminal\n"
                      f"          plain yzx runs {nix}/yzx instead\n", behind.stdout)

    def test_terminal_output_draws_progress_and_honors_no_color(self):
        # An empty NO_COLOR keeps color; an unusable archive size drops the bar.
        for shell, no_color, size in [(("sh",), None, None), (("/bin/bash", "--posix"), None, None),
                                      (("sh",), "1", None), (("sh",), "", "unknown")]:
            with self.subTest(shell=shell, no_color=no_color, size=size):
                shutil.rmtree(self.fixture / "1.4.0", ignore_errors=True)
                self.release("1.4.0", size=size)
                primary, secondary = pty.openpty()
                result = self.run_installer(environment={"NO_COLOR": no_color} if no_color is not None else {},
                                            stdout=secondary, shell=shell)
                os.close(secondary)
                output = b""
                try:
                    while chunk := os.read(primary, 4096):
                        output += chunk
                except OSError:  # Linux reports EIO once the installer has closed the terminal.
                    pass
                os.close(primary)
                self.assertEqual(result.returncode, 0, result.stderr)
                screen = output.decode()
                if size is None:
                    self.assertIn("\rDownloading 1 MB ", screen)
                    self.assertIn("━" * 30, screen)
                    self.assertIn(" 100%\r\n", screen)
                else:
                    self.assertIn("\nDownloading\r\n", screen)
                self.assertEqual("\x1b[" in screen, not no_color)
                lines = re.sub("\x1b\\[[0-9]*m", "", screen).replace("\r\n", "\n").split("\n")
                self.assertLessEqual(max(len(line.rsplit("\r", 1)[-1]) for line in lines), 80)

    @unittest.skipUnless(shutil.which("shellcheck"), "shellcheck is not installed")
    def test_posix_shell_lint(self):
        subprocess.run(["shellcheck", "--shell=sh", str(INSTALLER)], check=True)


if __name__ == "__main__":
    unittest.main()
