"""Preserve source material selected by the Linux archive's component origins."""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "packaging"))
from archive_metadata import verify
from grammar_sources import STORE, bundle, reachable, scan_error


def store_references(value):
    if not isinstance(value, str):
        value = json.dumps(value)
    return set(re.findall(r"/nix/store/([A-Za-z0-9+._?=-]+)", value))


def store_root(value):
    path = Path(value)
    if path.name == value and value not in ["", ".", ".."]:
        path = STORE / value
    if not path.is_relative_to(STORE) or len(path.parts) < len(STORE.parts) + 1:
        raise ValueError(f"invalid store origin: {value}")
    return STORE / path.parts[len(STORE.parts)]


def inventory(document, root, declaration, origins, identity):
    if type(declaration["schema_version"]) is not int or declaration["schema_version"] != 1:
        raise ValueError("unsupported component schema")
    graph = reachable(document, root)
    if graph[Path(root).name]["env"].get("novaSourceRevision") != (identity["revision"] or ""):
        raise ValueError("archive and source graph revisions disagree")
    outputs = {}
    direct_sources = set()
    for key, node in graph.items():
        for output in node.get("outputs", {}).values():
            if output.get("path"):
                outputs[Path(output["path"]).name] = key
        direct_sources.update(store_root(path).name for path in
                              node.get("inputSrcs", node.get("inputs", {}).get("srcs", [])))
    pending, selected, sources = [], {}, {}

    def source(name):
        if Path(name).name != name or name in ["", ".", ".."]:
            raise ValueError(f"invalid source origin: {name}")
        sources[name] = STORE / name

    def component(output):
        if output in outputs:
            pending.append(outputs[output])
        elif output in direct_sources:
            source(output)
        else:
            raise ValueError(f"missing component owner: {output}")

    for record in declaration["components"]:
        component(record["output"])
        if "derivation" in record and outputs.get(record["output"]) != record["derivation"]:
            raise ValueError(f"component derivation mismatch: {record['output']}")
    for origin in origins:
        component(origin)
    for name in declaration["sources"]:
        source(name)
    source(graph[Path(root).name]["env"]["novaSourceRoot"])
    while pending:
        key = pending.pop()
        if key in selected:
            continue
        node = graph[key]
        env = node["env"] | node.get("structuredAttrs", {})
        materials = set()
        for field in ["src", "srcs", "patches", "cargoDeps", "cargoVendorDir", "goModules"]:
            materials.update(store_references(env.get(field, "")))
        if not materials:
            for path in node.get("inputSrcs", node.get("inputs", {}).get("srcs", [])):
                materials.add(store_root(path).name)
            for field in ["buildCommand", "text"]:
                for origin in store_references(env.get(field, "")):
                    if origin in outputs:
                        component(origin)
                    else:
                        materials.add(origin)
        for name in materials:
            source(name)
        for field in ["buildInputs", "propagatedBuildInputs"]:
            for origin in store_references(env.get(field, "")):
                component(origin)
        selected[key] = {"name": env.get("name", node.get("name", key)),
                         "derivation": key, "materials": sorted(materials)}
    if not selected:
        raise ValueError("empty component inventory")
    return {"schema_version": 1, "package": identity,
            "archive_derivation": Path(root).name, "redistribution_review": "required",
            "coverage": "declared copied/embedded components, private native origins and their buildInputs; source/license review required",
            "declaration": declaration, "components": sorted(selected.values(), key=lambda c: c["derivation"]),
            "build_graph": graph}, sources


def prepare(sources, output):
    pending, batch = set(sources), 0
    while pending:
        subprocess.run(["nix-store", "--realise", "--add-root", str(output.absolute()) + f".inputs-{batch}",
                        "--indirect", *map(str, sorted(sources[name] for name in pending))],
                       check=True, stdout=subprocess.DEVNULL)
        found = set()
        for name in sorted(pending):
            if not sources[name].is_dir():
                continue
            for parent, directories, names in os.walk(sources[name], onerror=scan_error):
                for entry in directories + names:
                    path = Path(parent) / entry
                    if path.is_symlink():
                        target = Path(os.path.realpath(path))
                        if target.is_relative_to(STORE):
                            root = store_root(target)
                            if root.name not in sources:
                                sources[root.name] = root
                                found.add(root.name)
        pending, batch = found, batch + 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("graph", type=Path)
    parser.add_argument("archive_derivation")
    parser.add_argument("installed", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    if args.output.exists() or args.output.is_symlink():
        raise FileExistsError(args.output)
    identity = verify(args.installed, args.release)
    directory = args.installed / "share/yazelix"
    declaration = json.loads((directory / "archive-components.json").read_text())
    origins = (directory / "archive-origins.txt").read_text().splitlines()
    manifest, sources = inventory(json.loads(args.graph.read_text()), args.archive_derivation,
                                  declaration, origins, identity)
    prepare(sources, args.output)
    bundle(manifest, sources, args.output)
    print(f"Verified {len(manifest['components'])} component records and {len(sources)} source roots; redistribution review required.")
