import json
import os
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import grammar_sources
import runtime_sources


class RuntimeSourcesTests(unittest.TestCase):
    def test_component_selection_includes_embedded_and_static_sources(self):
        def node(name, **env):
            return {"env": {"name": name, **env}, "outputs": {
                "out": {"path": "/nix/store/" + name}}, "inputDrvs": {}}

        graph = {name + ".drv": node(name) for name in
                 ["archive", "program", "library", "plugin", "compiler", "unrelated"]}
        graph["archive.drv"]["inputDrvs"] = {name: ["out"] for name in graph if name != "archive.drv"}
        graph["archive.drv"]["env"]["novaSourceRevision"] = "a" * 40
        graph["archive.drv"]["env"]["novaSourceRoot"] = "nova-source"
        graph["program.drv"]["env"].update(src="/nix/store/program-src",
            cargoDeps="/nix/store/program-vendor", buildInputs="/nix/store/library",
            nativeBuildInputs="/nix/store/compiler")
        graph["library.drv"]["env"].update(src="/nix/store/library-src", patches="/nix/store/fix.patch")
        graph["plugin.drv"]["env"].update(src="/nix/store/filtered-plugin", cargoVendorDir="/nix/store/plugin-vendor")
        graph["compiler.drv"]["env"]["src"] = "/nix/store/compiler-src"
        graph["composition.drv"] = node("composition", buildCommand='cp /nix/store/plugin/bin/plugin $out; echo /nix/store/nova-source/LICENSE; source "${BASE:-/nix/store/program}/config"')
        graph["composition.drv"]["inputs"] = {"srcs": ["generated-config"]}
        graph["archive.drv"]["inputDrvs"]["composition.drv"] = ["out"]
        declaration = {"schema_version": 1,
            "components": [{"output": "program", "derivation": "program.drv", "role": "command"},
                           {"output": "plugin", "derivation": "plugin.drv", "role": "embedded"}],
            "sources": ["full-plugin-source", "nova-source"]}
        declaration["components"].append({"output": "composition"})
        manifest, sources = runtime_sources.inventory(graph, "archive.drv", declaration, [], {"revision": "a" * 40})
        self.assertEqual(set(sources), {"program-src", "program-vendor", "library-src", "fix.patch",
                                      "filtered-plugin", "plugin-vendor", "full-plugin-source", "nova-source", "generated-config"})
        self.assertEqual({c["name"] for c in manifest["components"]}, {"program", "library", "plugin", "composition"})
        self.assertEqual(manifest["redistribution_review"], "required")
        for fault in ["missing-owner", "wrong-derivation", "unsafe-source", "missing-graph", "wrong-revision"]:
            with self.subTest(fault=fault):
                changed = json.loads(json.dumps(declaration))
                native = []
                candidate = graph
                if fault == "missing-owner": native = ["unknown"]
                elif fault == "wrong-derivation": changed["components"][0]["derivation"] = "plugin.drv"
                elif fault == "unsafe-source": changed["sources"] = ["../escape"]
                elif fault == "missing-graph": candidate = {k: v for k, v in graph.items() if k != "library.drv"}
                else:
                    candidate = json.loads(json.dumps(graph))
                    candidate["archive.drv"]["env"]["novaSourceRevision"] = "b" * 40
                with self.assertRaises(ValueError):
                    runtime_sources.inventory(candidate, "archive.drv", changed, native, {"revision": "a" * 40})

    def test_changed_source_cannot_finalize_a_bundle(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            file = source / "LICENSE"
            file.write_text("original notice\n")
            output = root / "sources.tar.gz"
            original = tarfile.TarFile.addfile

            def mutate(archive, info, content=None):
                if info.name == "SOURCE-MANIFEST.json":
                    file.write_text("changed bytes\n")
                return original(archive, info, content)

            with patch.object(tarfile.TarFile, "addfile", mutate):
                with self.assertRaisesRegex(ValueError, "source.*changed|hash"):
                    grammar_sources.bundle({}, {source.name: source}, output)
            self.assertFalse(output.exists())
            self.assertEqual(sorted(p.name for p in root.iterdir()), ["source"])
            grammar_sources.bundle({}, {source.name: source}, output)
            saved = grammar_sources.verify_bundle(output)
            self.assertEqual(saved["sources"][0]["path"], "sources/source")

    def test_store_optimised_hard_links_are_archived_as_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first, second = root / "first", root / "second"
            first.mkdir()
            second.mkdir()
            (first / "parser.h").write_text("shared header\n")
            os.link(first / "parser.h", second / "parser.h")
            output = root / "sources.tar.gz"
            grammar_sources.bundle({}, {"first": first, "second": second}, output)
            grammar_sources.verify_bundle(output)
            with tarfile.open(output) as archive:
                members = {m.name: m for m in archive.getmembers() if m.name.endswith("parser.h")}
                self.assertEqual(sorted(members), ["sources/first/parser.h", "sources/second/parser.h"])
                self.assertTrue(all(m.isfile() for m in members.values()))
                self.assertEqual(archive.extractfile(members["sources/second/parser.h"]).read(),
                                 b"shared header\n")

    def test_vendor_link_inputs_are_realized_before_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary) / "store"
            vendor = store / "vendor"
            store.mkdir()
            crate = store / "crate-source"
            standalone = store / "fix.patch"
            standalone.write_text("original patch\n")
            sources = {vendor.name: vendor, standalone.name: standalone}
            graph = {"vendor.drv": {"env": {}, "outputs": {
                "out": {"path": vendor.name}, "dev": {"path": "unused-output"}}},
                "crate.drv": {"env": {"out": str(crate)}, "outputs": {"out": {"hash": "fixed"}}}}
            calls = []

            def realize(command, **kwargs):
                calls.append(command)
                if str(store / "vendor.drv") + "!out" in command:
                    vendor.mkdir()
                    (vendor / "dependency").symlink_to(crate)
                elif str(store / "crate.drv") + "!out" in command:
                    crate.mkdir()
                    (crate / "LICENSE").write_text("original crate notice\n")
                else:
                    self.fail("an unavailable generated source needs its producing derivation")
                self.assertNotIn(str(store / "vendor.drv") + "!dev", command)

            output = Path(temporary) / "sources.tar.gz"
            with patch.object(runtime_sources, "STORE", store), patch.object(grammar_sources, "STORE", store), patch.object(
                    grammar_sources.subprocess, "run", realize):
                runtime_sources.prepare(sources, output, graph)
            self.assertEqual(set(sources), {"vendor", "crate-source", "fix.patch"})
            self.assertEqual(len(calls), 2)
            self.assertIn(str(standalone), calls[0])
            self.assertNotEqual(calls[0][calls[0].index("--add-root") + 1], calls[1][calls[1].index("--add-root") + 1])
            grammar_sources.bundle({}, sources, output)
            with tarfile.open(output) as archive:
                self.assertEqual(archive.getmember("sources/vendor/dependency").linkname, "../crate-source")
                self.assertEqual(archive.extractfile("sources/crate-source/LICENSE").read(), b"original crate notice\n")
                self.assertEqual(archive.extractfile("sources/fix.patch").read(), b"original patch\n")


if __name__ == "__main__":
    unittest.main()
