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
`rbox-derives` has an independent version; update its manifest and dependency
requirements separately when releasing changes to the macro.

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

Must be one of the following:

- **feat:** A new feature
- **fix:** Bug fixes or improvements
- **perf:** A code change that improves performance
- **refactor:** Code refactoring
- **ci:** Changes to CI configuration files and scripts
- **docs:** Documention changes
- **build:** Updating Makefile etc, no production code changes
- **test:** Adding missing tests or correcting existing tests
- **update** Other configurations updates

### Scope (optional)

The scope should specify the affected part of the project.
The following is a list of possible scopes:

- **config**: Configuration handling
- **xml**: Rekordbox XML database handling
- **db**: Rekordbox v6 database handling
- **anlz**: ANLZ file handling
- **setting**: MySettings file handling
- **binding**: General language binding changes
- **py**: Python bindings
- **js**: JavaScript bindings


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
