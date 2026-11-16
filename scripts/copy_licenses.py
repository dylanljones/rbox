# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Synchronize the root LICENSE into workspace crates and binding packages.

Run with Python before packaging, or use --check to detect missing/stale copies without writing.
"""

import argparse
import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def license_targets(root: Path) -> list[Path]:
    """Find workspace members and npm packages using the workspace license."""
    root = root.resolve()
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    workspace = manifest["workspace"]
    license_id = workspace["package"]["license"]
    excluded = {
        path.resolve()
        for pattern in workspace.get("exclude", [])
        for path in root.glob(pattern)
    }
    members = {
        path.resolve()
        for pattern in workspace["members"]
        for path in root.glob(pattern)
        if (path / "Cargo.toml").is_file()
    } - excluded
    targets = set()
    for member in sorted(members):
        if not member.is_relative_to(root):
            raise ValueError(f"Workspace member is outside the repository: {member}")
        package = tomllib.loads((member / "Cargo.toml").read_text(encoding="utf-8"))[
            "package"
        ]
        if package.get("license") in ({"workspace": True}, license_id):
            targets.add(member / "LICENSE")

    js_root = root / "bindings/javascript"
    npm_manifests = [
        js_root / "package.json",
        *sorted((js_root / "npm").glob("*/package.json")),
    ]
    for path in npm_manifests:
        if path.is_file():
            package = json.loads(path.read_text(encoding="utf-8"))
            if package.get("license") == license_id:
                targets.add(path.parent / "LICENSE")

    for target in targets:
        if not target.resolve().is_relative_to(root):
            raise ValueError(f"License destination is outside the repository: {target}")
    return sorted(targets - {root / "LICENSE"})


def synchronize(root: Path, *, check: bool = False) -> list[Path]:
    """Return missing/stale destinations, copying bytes unless checking only."""
    content = (root / "LICENSE").read_bytes()
    if not content:
        raise ValueError("The root LICENSE is empty")
    targets = license_targets(root)
    changed = []
    for target in targets:
        if target.is_file() and target.read_bytes() == content:
            continue
        changed.append(target)
        if not check:
            target.write_bytes(content)
    return changed


def main() -> int:
    """Copy licenses or report differences for CI."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true", help="fail if any copy is missing or stale"
    )
    args = parser.parse_args()
    try:
        changed = synchronize(ROOT, check=args.check)
    except (OSError, ValueError, KeyError) as error:
        print(f"Cannot synchronize licenses: {error}", file=sys.stderr)
        return 1
    for path in changed:
        print(
            f"{'Needs update' if args.check else 'Copied'}: {path.relative_to(ROOT)}",
            file=sys.stderr,
        )
    if args.check and changed:
        print(
            "Run python scripts/copy_licenses.py and commit the license copies.",
            file=sys.stderr,
        )
        return 1
    print(
        f"License copies {'verified' if args.check else 'synchronized'}.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
