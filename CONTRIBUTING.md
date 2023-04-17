# Contributing

Thank you for your interest in contributing! The goal
of ibc-rs is to provide a high quality, formally verified implementation of
the IBC protocol.

All work on the code base should be motivated by a Github
issue. Before opening a new issue, first do a search of open and closed issues
to make sure that yours will not be a duplicate.
If you would like to work on an issue which already exists, please indicate so
by leaving a comment. If what you'd like to work on hasn't already been covered
by an issue, then open a new one to get the process going.

The rest of this document outlines the best practices for contributing to this
repository:

- [Decision Making](#decision-making) - process for agreeing to changes
- [Issues](#issues) - what makes a good issue
- [Pull Requests](#pull-requests) - what makes a good pull request
- [Forking](#forking) - fork the repo to make pull requests
- [Changelog](#changelog) - changes must be recorded in the changelog
- [Releases](#releases) - how to release new version of the crates

## Decision Making

When contributing to the project, the following process leads to the best chance of
landing the changes in `main`.

All new contributions should start with a Github issue which captures the
problem you're trying to solve. Starting off with an issue allows for early
feedback. Once the issue is created, maintainers may request that more detailed
documentation be written in the form of a Request for Comment (RFC) or an
Architectural Decision Record
([ADR](https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/README.md)).

Discussion at the RFC stage will build collective understanding of the dimensions
of the problem and help structure conversations around trade-offs.

When the problem is well understood but the solution leads to large
structural changes to the code base, these changes should be proposed in
the form of an [Architectural Decision Record
(ADR)](./docs/architecture/). The ADR will help build consensus on an
overall strategy to ensure that the code base maintains coherence
in the larger context. If you are not comfortable with writing an ADR,
you can open a regular issue and the maintainers will help you
turn it into an ADR.

When the problem and the proposed solution are well understood,
changes should start with a [draft
pull request](https://github.blog/2019-02-14-introducing-draft-pull-requests/)
against `main`. The draft status signals that work is underway. When the work
is ready for feedback, hitting "Ready for Review" will signal to the
maintainers to take a look.

Implementation trajectories should aim to proceed where possible as a series
of smaller incremental changes, in the form of small PRs that can be merged
quickly. This helps manage the load for reviewers and reduces the likelihood
that PRs will sit open for long periods of time.

![Contributing flow](https://github.com/tendermint/tendermint/blob/v0.33.6/docs/imgs/contributing.png?raw=true)

Each stage of the process is aimed at creating feedback cycles which align
contributors and maintainers in order to ensure that:

- Contributors don’t waste their time implementing/proposing features which won’t land in `main`
- Maintainers have the necessary context in order to support and review contributions

## Issues

We welcome bug reports, feature requests, and other contributions to our project. To open an issue, please follow these guidelines:

1. **Search existing issues**: Before opening a new issue, please search existing issues to ensure that is not a duplicates.

2. **Provide a clear and descriptive title**: This helps others understand the nature of the issue at a glance.

3. **Provide detailed information**: In the issue description, clearly state the purpose of the issue and follow the guidelines of the issue template

4. A maintainer will take care of assigning the appropriate labels to your issue.

We use the following convention for issue label names:
    - (WHY) The purpose or objective of this issue with Objective-level "O" labels like `O: security`, `O: new-feature`, etc.
    - (WHICH) The part of the system this issue relates to using:
      - External-level "E" labels if the issue fall outside the current scope of the system and is related to external dependencies or projects like `E: non-cosmos`, `E: no-std` etc.
      - or "Internal-level "I" labels for anything related to the current scope of the product like `I: errors`, `I: documentation`, etc.
    - (HOW) If any administrative considerations should be taken into account use Administrative-level "A" labels like `A: help-wanted`, `A: critical`, etc.

## Pull Requests

If you have write access to the ibc-rs repo, you can directly branch off of `main`.
This makes it easier for project maintainers to directly make changes to your
branch should the need arise. Otherwise, check [Forking](#forking) section for instructions.

Branch names should be prefixed with the author's name followed by a short description
of the feature, eg. `name/feature-x`.

Pull requests are made against `main` and are squash-merged into main.

**PRs must:**

- make reference to an issue outlining the context
- update any relevant documentation and include tests
- add a corresponding entry in the `.changelog` directory using `unclog`,
  see the [Changelog](#changelog) section for more details.

Pull requests should aim to be small and self-contained to facilitate quick
review and merging. Larger change sets should be broken up across multiple PRs.
Commits should be concise but informative, and moderately clean. Commits will be squashed into a
single commit for the PR with all the commit messages.

If the issue you worked on was tagged `A: low-priority`, we'll do our best to
review it in a timely manner, but please expect longer wait times for a review
in general. If a low priority issue is important to you, please leave a comment
explaining why, and we will reprioritize it!

## Responsibilities of a PR Reviewer

If you're tagged as the reviewer of a PR, you are responsible for shepherding it
through to completion. This includes fixing issues with the PR and taking the
lead on decisions that need to be resolved in order to get the PR merged.

If you're tagged as a reviewer on a PR that affects a part of the code base that
you are unfamiliar with, you can hand it off to someone (with their
consent) who is more appropriate to shepherd the PR through to completion.

## Forking

If you do not have write access to the repository, your contribution should be
made through a fork on Github. Fork the repository, contribute to your fork
(either in the `main` branch of the fork or in a separate branch), and then
make a pull request back upstream.

When forking, add your fork's URL as a new git remote in your local copy of the
repo. For instance, to create a fork and work on a branch of it:

- Create the fork on GitHub, using the fork button.
- `cd` to the original clone of the repo on your machine
- `git remote rename origin upstream`
- `git remote add origin git@github.com:<location of fork>`

Now `origin` refers to your fork and `upstream` refers to the original version.
Now `git push -u origin main` to update the fork, and make pull requests
against the original repo.

To pull in updates from the origin repo, run `git fetch upstream` followed by
`git rebase upstream/main` (or whatever branch you're working in).

## Changelog

Every non-trivial PR must update the [CHANGELOG](CHANGELOG.md). This is
accomplished indirectly by adding entries to the `.changelog` folder in
[`unclog`](https://github.com/informalsystems/unclog) format using the `unclog` CLI tool.
`CHANGELOG.md` will be built by whomever is responsible for performing a release just
prior to release - this is to avoid changelog conflicts prior to releases.

### Install `unclog`

```bash
cargo install unclog
```

### Examples

Add a `.changelog` entry to signal that a bug was fixed, without mentioning any
component.

```bash
unclog add -i update-unclog-instructions -s bug -n 1634 -m "Update CONTRIBUTING.md for latest version of unclog" --editor vim
```

Add a .changelog entry for the `ibc` crate.

```bash
unclog add -c ibc -s features --id a-new-feature --issue-no 1235 -m "msg about this new-feature" --editor vim
```

### Preview unreleased changes

```bash
unclog build -u
```

The Changelog is *not* a record of what Pull Requests were merged;
the commit history already shows that. The Changelog is a notice to users
about how their expectations of the software should be modified.
It is part of the UX of a release and is a *critical* user facing integration point.
The Changelog must be clean, inviting, and readable, with concise, meaningful entries.
Entries must be semantically meaningful to users. If a change takes multiple
Pull Requests to complete, it should likely have only a single entry in the
Changelog describing the net effect to the user. Instead of linking PRs directly, we
instead prefer to log issues, which tend to be higher-level, hence more relevant for users.

When writing Changelog entries, ensure they are targeting users of the software,
not fellow developers. Developers have much more context and care about more
things than users do. Changelogs are for users.

Changelog structure is modeled after
[Tendermint Core](https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md)
and [Hashicorp Consul](http://github.com/hashicorp/consul/tree/master/CHANGELOG.md).
See those changelogs for examples.

We currently split changes for a given release between these four sections: Breaking
Changes, Features, Improvements, Bug Fixes.

Entries in the changelog should initially be logged in the __Unreleased__ section, which
represents a "staging area" for accumulating all the changes throughout a
release (see [Pull Requests](#pull-requests) below). With each release,
the entries then move from this section into their permanent place under a
specific release number in Changelog.

Changelog entries should be formatted as follows:

```md
- [pkg] Some description about the change ([#xxx](https://github.com/cosmos/ibc-rs/issues/xxx)) (optional @contributor)
```

Here, `pkg` is the part of the code that changed (typically a
top-level crate, but could be `<crate>/<module>`), `xxx` is the issue number, and `contributor`
is the author/s of the change.

It's also acceptable for `xxx` to refer to the relevant pull request, but issue
numbers are preferred.
Note this means issues (or pull-requests) should be opened first so the changelog can then
be updated with the corresponding number.

Changelog entries should be ordered alphabetically according to the
`pkg`, and numerically according to their issue/PR number.

Changes with multiple classifications should be doubly included (eg. a bug fix
that is also a breaking change should be recorded under both).

Breaking changes are further subdivided according to the APIs/users they impact.
Any change that effects multiple APIs/users should be recorded multiply - for
instance, a change to some core protocol data structure might need to be
reflected both as breaking the core protocol but also breaking any APIs where core data structures are
exposed.

## Releases

Our release process is as follows:

1. In a new branch `release/vX.Y.Z`, update the [changelog](#changelog) to reflect and summarize all changes in
   the release. This involves:
   1. Running `unclog build -u` and copy pasting the output at the top
      of the `CHANGELOG.md` file, making sure to update the header with
      the new version.
   2. Running `unclog release --editor <editor> vX.Y.Z` to create a summary of all of the changes
      in this release.
      1. Your text editor will open. Write the release summary, and close the editor.
         1. Make sure to include a comment on whether or not the release contains consensus-breaking changes.
      2. Add this same summary to `CHANGELOG.md` as well.
   3. Committing the updated `CHANGELOG.md` file and `.changelog` directory to the repo.
2. Push this to a branch `release/vX.Y.Z` according to the version number of
   the anticipated release (e.g. `release/v0.18.0`) and open a **draft PR**.
3. Bump all relevant versions in the `crates/ibc-types/Cargo.toml` file (making sure
      dependencies' versions are updated too) to the new version and push these
      changes to the release PR.
4. In the `crates/ibc-types/` directory, run `cargo doc --all-features --open` locally to double-check that all the
   documentation compiles and seems up-to-date and coherent. Fix any potential
   issues here and push them to the release PR.
5. In the `crates/ibc-types/` directory, run `cargo publish --dry-run` to double-check that publishing will work. Fix
   any potential issues here and push them to the release PR.
6. Mark the PR as **Ready for Review** and incorporate feedback on the release.
7. Once approved, merge the PR, and pull the `main` branch.
8. Once all crates have been successfully released, create a signed tag and push it to
   GitHub: `git tag -s -a vX.Y.Z`. In the tag message, write the version and the link
   to the corresponding section of the changelog. Pushing the tag will trigger the [Release workflow](https://github.com/cosmos/ibc-rs/actions/workflows/release.yml),
   which publishes the crate on crates.io.
9. Once the tag is pushed, create a GitHub release and append
   `[📖CHANGELOG](https://github.com/cosmos/ibc-rs/blob/main/CHANGELOG.md#vXYZ)` 
   to the release description.
10. Make sure that the [Release workflow](https://github.com/cosmos/ibc-rs/actions/workflows/release.yml) to crates.io has completed successfully.
11. All done! 🎉
