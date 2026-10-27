# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Print crates.io-eligible workspace packages in dependency order from Cargo metadata.

Usage: cargo metadata --no-deps --format-version 1 | python scripts/publish_order.py
Output is tab-separated package name and version; this script never publishes.
"""

import json
import sys
from pathlib import Path


def publish_order(metadata):
    """Return a deterministic plan, including optional and target-specific dependencies."""
    members = set(metadata["workspace_members"])
    packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in members
    }
    selected = {
        name: package
        for name, package in packages.items()
        if package["publish"] is None or "crates-io" in package["publish"]
    }
    by_path = {
        Path(package["manifest_path"]).resolve().parent: name
        for name, package in packages.items()
    }
    dependencies = {name: set() for name in selected}
    for name, package in selected.items():
        for dependency in package["dependencies"]:
            # Cargo drops dev dependencies without versions when packaging.
            if dependency["kind"] == "dev" and dependency["req"] == "*":
                continue
            path = dependency.get("path")
            if not path:
                continue
            target = by_path.get(Path(path).resolve())
            if target is None:
                raise ValueError(
                    f"{name}: path dependency {path} is outside the workspace"
                )
            if target not in selected:
                raise ValueError(
                    f"{name}: dependency {target} is not publishable to crates.io"
                )
            if dependency["req"] == "*":
                raise ValueError(
                    f"{name}: dependency {target} needs a version for publication"
                )
            dependencies[name].add(target)

    ordered = []
    while dependencies:
        ready = sorted(name for name, required in dependencies.items() if not required)
        if not ready:
            raise ValueError(
                "Publication dependency cycle: " + ", ".join(sorted(dependencies))
            )
        for name in ready:
            ordered.append(selected[name])
            del dependencies[name]
        for required in dependencies.values():
            required.difference_update(ready)
    return ordered


def main():
    """Read metadata on stdin and emit the complete plan only after validation."""
    try:
        plan = publish_order(json.load(sys.stdin))
    except (ValueError, KeyError) as error:
        print(f"Cannot plan publication: {error}", file=sys.stderr)
        return 1
    for package in plan:
        print(f"{package['name']}\t{package['version']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
