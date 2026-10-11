# Contributing

If you encounter an issue or want to contribute to rbox, please feel free to get in touch,
[open an issue][issue] or create a new [pull request][pulls]!
For general questions or discussions about the library or Rekordbox,
please use [GitHub Discussions][discussions] instead of opening an issue.

rbox is tested on Windows, MacOS and Linux, however some features can't be tested in
the CI setup since it requires a working Rekordbox installation.

## Tests and releases

Run the Rust workspace tests from the repository root:

```sh
cargo test --workspace --exclude rbox-py --exclude rbox-js --lib --bins --tests
```

The bindings have separate API tests in `bindings/python/tests` and
`bindings/javascript/__test__`. CI runs on pull requests targeting `main`,
and the Tests workflow can also be started manually on `dev` or another branch.
Changes to shared crates trigger tests for both bindings.

Use Python 3.10 or later to bump the workspace release version.
First install the TOML parser:

```sh
python -m pip install tomlkit
```

Then preview or apply the version bump:

```sh
python scripts/bump_version.py 0.1.8 --dry-run
python scripts/bump_version.py 0.1.8
```

The script updates workspace metadata, versioned dependencies between workspace
crates, binding package versions, and existing version references in documentation.
To bump `rbox-derives` and its versioned dependency requirements separately:

```sh
python scripts/bump_version.py 0.2.1 --derives --dry-run
python scripts/bump_version.py 0.2.1 --derives
```

This leaves the workspace version, other crate versions, and binding package
versions unchanged. Dependencies that only specify a local path remain unchanged.

Rust releases publish the public sub-crates in dependency order, followed by
`rbox`. Already-published versions are skipped so a failed release can be retried.
`codegen`, `rbox-testing`, and the binding crates are not published to crates.io.
Rust publication waits for the reusable Tests workflow; npm and PyPI publication
wait for tests of their built artifacts. PyPI releases also verify that the source
archive builds without the repository checkout.

## Pre-commit Hooks

We are using the [pre-commit framework] to automatically run a linter and code formatter
at commit time. This ensures that every commit fulfills the basic requirements to be
mergeable and follows the coding style of the project.

The pre-commit hooks can be installed via
````sh
$ pre-commit install
````

## Commit Message Format

This project uses the [Conventional Commits] specification for commit messages.

```text
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Type

Choose the type based on the purpose of the change:

| Type | Use for |
| --- | --- |
| `feat` | New functionality or an extension to an existing API |
| `fix` | Correcting incorrect behavior |
| `perf` | Improving performance without changing intended behavior |
| `refactor` | Restructuring production code without adding features or fixing bugs |
| `docs` | Documentation, docstrings, and examples |
| `test` | Adding or improving tests, fixtures, or benchmarks |
| `build` | Build tooling, packaging, dependencies, or Cargo feature configuration |
| `ci` | GitHub Actions workflows and CI configuration |
| `chore` | Other maintenance, such as version bumps or repository housekeeping |

A dependency update normally uses `build`;
a dependency update made to fix a specific bug can use `fix`.
Tests and documentation accompanying a feature or fix belong under that change's
type; use `test` or `docs` when they are the main purpose of the commit.

### Scope (optional)
Use the affected crate's name without the `rbox-` prefix, or the relevant binding
or tooling area:

| Scope | Area |
| --- | --- |
| `rbox` | Public facade crate, re-exports, and feature forwarding |
| `core` | Shared utilities, paths, Rekordbox discovery, and options |
| `db` | Shared database connections, ordering, and relationship infrastructure |
| `masterdb` | Rekordbox master database handling |
| `onelib` | OneLibrary / Device Library Plus database handling |
| `anlz` | Analysis file handling |
| `xml` | Rekordbox XML database handling |
| `setting` | MySettings file handling |
| `derives` | Procedural derive macros |
| `testing` | Shared test support in `rbox-testing` |
| `py` | Python bindings and their tests or packaging |
| `js` | JavaScript bindings and their tests or packaging |
| `bindings` | Changes shared by both language bindings |
| `scripts` | Repository maintenance scripts |
| `deps` | Dependency updates across the workspace |
| `release` | Version bumps and release tooling |

Choose one scope that best describes the change. Omit it for broad changes that
have no clear primary area. The scope describes the affected component, not the
kind of work: use `test(masterdb)` for master database tests and `ci(js)` for the
JavaScript workflow. Reserve `db` for shared infrastructure; use `masterdb` or
`onelib` for a specific database.

Examples:

```text
feat(anlz): add lazy section loading
fix(onelib): return a typed error for invalid dates
refactor(db): extract shared ordering traits
test(masterdb): cover playlist movement edge cases
build(deps): update diesel
ci(js): enable npm trusted publishing
docs: explain the workspace crate layout
chore(release): bump the workspace version
```

### Subject

Use the summary field to provide a succinct description of the change:

- use the imperative, present tense: "change" not "changed" nor "changes"
- don't capitalize the first letter
- no dot (.) at the end

### Body (optional)

Just as in the summary, use the imperative, present tense: "fix" not "fixed" nor "fixes".

Explain the motivation for the change in the commit message body.
This commit message should explain why you are making the change. You can include a comparison of
the previous behavior with the new behavior in order to illustrate the impact of the change.

### Footer (optional)

The footer can contain information about breaking changes and deprecations and is also the place
to reference GitHub issues, Jira tickets, and other PRs that this commit closes or is related to.
For example:
```
<type>(<scope>): <description>

[optional body]

BREAKING CHANGE: <breaking change summary>

<breaking change description + migration instructions>

Fixes #<issue number>
```

or

```
<type>(<scope>): <description>

[optional body]

DEPRECATED: <what is deprecated>

<deprecation description + recommended update path>
```

Breaking Change section should start with the phrase "BREAKING CHANGE: " followed by a summary of
the breaking change, a blank line, and a detailed description of the breaking change that also
includes migration instructions.

Similarly, a Deprecation section should start with "DEPRECATED: " followed by a short description
of what is deprecated, a blank line, and a detailed description of the deprecation that also
mentions the recommended update path.

[issue]: https://github.com/dylanljones/rbox/issues
[pulls]: https://github.com/dylanljones/rbox/pulls
[discussions]: https://github.com/dylanljones/rbox/discussions
[pre-commit framework]: https://pre-commit.com/
[Conventional Commits]: https://www.conventionalcommits.org/
