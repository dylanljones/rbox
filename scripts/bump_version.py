# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Bump the workspace release and its internal dependency requirements.

Requires Python 3.10+ and tomlkit. Independently versioned crates (rbox-derives)
retain both their package version and dependency requirements. Only Git-visible files
are visited, so build outputs, environments, and node_modules are excluded.
"""

import argparse
from pathlib import Path
import re
import subprocess

try:
    import tomlkit
except ModuleNotFoundError:
    raise SystemExit("Install tomlkit with 'python -m pip install tomlkit'.") from None

ROOT = Path(__file__).resolve().parent.parent
SEMVER = re.compile(
    r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)


def update_dependencies(text: str, names: set[str], version: str) -> str:
    """Update versioned internal dependencies, preserving manifest formatting."""
    data = tomlkit.loads(text)
    sections = re.split(r"(?m)^(\[[^\r\n]+\][ \t]*(?:#[^\r\n]*)?\r?\n)", text)
    for index in range(1, len(sections), 2):
        header, body = sections[index : index + 2]
        if "dependencies" not in header:
            continue
        # Parse the table header to resolve quoted target selectors and aliases.
        table = tomlkit.loads(header + "__marker = true\n")
        keys = []
        while "__marker" not in table:
            key = next(iter(table))
            keys.append(key)
            table = table[key]
        if not any(
            key in ("dependencies", "dev-dependencies", "build-dependencies")
            for key in keys
        ):
            continue
        values = data
        for key in keys:
            values = values[key]
        if keys[-1] in ("dependencies", "dev-dependencies", "build-dependencies"):
            for alias, dependency in values.items():
                package = (
                    dependency.get("package", alias)
                    if isinstance(dependency, dict)
                    else alias
                )
                if package not in names:
                    continue
                if isinstance(dependency, dict):
                    if "version" not in dependency:
                        continue
                    pattern = rf'(?ms)(^\s*{re.escape(alias)}\s*=\s*\{{[^}}]*?\bversion\s*=\s*)["\'][^"\']+["\']'
                else:
                    pattern = rf'(?m)(^\s*{re.escape(alias)}\s*=\s*)["\'][^"\']+["\']'
                body = re.sub(pattern, lambda m: m[1] + f'"{version}"', body)
        elif values.get("package", keys[-1]) in names and "version" in values:
            body = re.sub(
                r'(?m)^(version\s*=\s*)["\'][^"\']+["\']',
                lambda m: m[1] + f'"{version}"',
                body,
            )
        sections[index + 1] = body
    return "".join(sections)


def bump_version(new_version: str, dry: bool = False, encoding: str = "utf-8") -> None:
    """Update release metadata and versioned dependencies without bumping derives."""
    prerelease = new_version.split("+", 1)[0].partition("-")[2]
    if not SEMVER.fullmatch(new_version) or any(
        part.isdigit() and len(part) > 1 and part.startswith("0")
        for part in prerelease.split(".")
    ):
        raise ValueError(f"Invalid version: {new_version}")
    workspace_path = ROOT / "Cargo.toml"
    workspace = tomlkit.loads(workspace_path.read_text(encoding=encoding))
    old_version = workspace["workspace"]["package"]["version"]
    output = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=ROOT,
    )
    files = sorted(
        {ROOT / name.decode("utf-8") for name in output.split(b"\0") if name}
    )
    manifests = [workspace_path]
    excluded = {
        path.resolve()
        for pattern in workspace["workspace"].get("exclude", [])
        for path in ROOT.glob(pattern)
    }
    for pattern in workspace["workspace"].get("members", []):
        manifests.extend(
            path / "Cargo.toml"
            for path in ROOT.glob(pattern)
            if path.resolve() not in excluded and (path / "Cargo.toml").is_file()
        )
    names = set()
    for path in manifests:
        package = tomlkit.loads(path.read_text(encoding=encoding)).get("package", {})
        if package.get("version") == {"workspace": True}:
            names.add(package["name"])

    replacements = (
        'rbox = "{v}"',
        'version = "{v}"',
        '"version": "{v}"',
        "bindingPackageVersion !== '{v}'",
        "expected {v} but got",
    )
    changes = {}
    for path in files:
        if not path.is_file() or (
            path not in manifests and path.suffix not in (".md", ".js", ".json")
        ):
            continue
        # Keep the original line endings when changing versions.
        original = path.read_bytes().decode(encoding)
        text = original
        if path in manifests:
            text = update_dependencies(text, names, new_version)
            if path == workspace_path:
                text = re.sub(
                    r'(?ms)(^\[workspace\.package\]\s*.*?^version\s*=\s*)"[^"]+"',
                    lambda m: m[1] + f'"{new_version}"',
                    text,
                )
        else:
            for template in replacements:
                text = text.replace(
                    template.format(v=old_version), template.format(v=new_version)
                )
        if text != original:
            changes[path] = text
    # Compute and validate every edit before writing any file.
    for path, text in changes.items():
        if not dry:
            path.write_bytes(text.encode(encoding))
        print(f"{'Would update' if dry else 'Updated'} {path.relative_to(ROOT)}")
    print(
        f"{'Would bump' if dry else 'Bumped'} {old_version} -> {new_version}: {len(changes)} files"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="New workspace version, e.g. 0.1.8")
    parser.add_argument(
        "--dry-run", action="store_true", help="Show changes without writing files"
    )
    args = parser.parse_args()
    bump_version(args.version, dry=args.dry_run)


if __name__ == "__main__":
    main()
