# Copyright (C) 2026 Dylan Jones
# SPDX-License-Identifier: GPL-3.0-only

"""Publication planning honors Cargo eligibility and internal dependency edges."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "publish_order", Path(__file__).parents[1] / "publish_order.py"
)
publish = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publish)


def package(name, allowed=None, dependencies=()):
    return dict(
        name=name,
        id=name,
        version="1.2.3",
        publish=allowed,
        manifest_path=str(Path.cwd() / name / "Cargo.toml"),
        dependencies=list(dependencies),
    )


def dependency(name, kind=None, req="^1.2.3", **extra):
    return dict(name=name, path=str(Path.cwd() / name), kind=kind, req=req, **extra)


def plan(*packages, members=None):
    return publish.publish_order(
        dict(
            packages=list(packages),
            workspace_members=members
            if members is not None
            else [p["id"] for p in packages],
        )
    )


class PublishOrderTests(unittest.TestCase):
    def test_eligibility_and_workspace_membership(self):
        result = plan(
            package("default"),
            package("explicit", ["crates-io"]),
            package("disabled", []),
            package("private", ["internal"]),
            package("external"),
            members=["default", "explicit", "disabled", "private"],
        )
        self.assertEqual([p["name"] for p in result], ["default", "explicit"])
        self.assertEqual(result[0]["version"], "1.2.3")

    def test_optional_build_target_and_renamed_dependencies(self):
        result = plan(
            package(
                "app",
                dependencies=[dependency("library", optional=True, rename="alias")],
            ),
            package(
                "library",
                dependencies=[dependency("macro", "build", target="cfg(windows)")],
            ),
            package("macro"),
        )
        self.assertEqual([p["name"] for p in result], ["macro", "library", "app"])

    def test_unversioned_dev_dependency_is_omitted(self):
        result = plan(
            package("app", dependencies=[dependency("tests", "dev", "*")]),
            package("tests", []),
        )
        self.assertEqual([p["name"] for p in result], ["app"])

    def test_versioned_dev_dependency_is_ordered(self):
        result = plan(
            package("app", dependencies=[dependency("tests", "dev")]), package("tests")
        )
        self.assertEqual([p["name"] for p in result], ["tests", "app"])

    def test_invalid_dependencies_fail_before_publishing(self):
        for dep, target, message in [
            (dependency("other"), package("other", []), "not publishable"),
            (dependency("other", req="*"), package("other"), "needs a version"),
            (dependency("missing"), package("other"), "outside the workspace"),
        ]:
            with (
                self.subTest(message=message),
                self.assertRaisesRegex(ValueError, message),
            ):
                plan(package("app", dependencies=[dep]), target)

    def test_cycle_fails(self):
        with self.assertRaisesRegex(ValueError, "cycle"):
            plan(
                package("a", dependencies=[dependency("b")]),
                package("b", dependencies=[dependency("a")]),
            )


if __name__ == "__main__":
    unittest.main()
