# Contributing

Thank you for considering to contribute to `devrc`!

We welcome any form of contribution:

  * Grammar corrections
  * New issues (feature requests, bug reports, ideas)
  * Pull requests (documentation improvements, code improvements, new features)


*Note:* Please open a ticket first, before you open a pull request. This will give us the chance to discuss any potential changes first.

## First time setup

Make sure you have installed stable version `devrc` in your system. There are few tasks that helps your to check code before PR.

## Check code

Ensure that your contribution changes don't have any code style issues or break tests:
```bash
devrc check
```

## Running linters

Run rustfmt and clippy:

```bash
devrc lint
```

## Running tests

Run test cases:

```bash
devrc test
```

## Add an entry to the changelog

If your contribution changes the behavior of `devrc` (as opposed to a typo-fix
in the documentation), please update the [`CHANGELOG.md`](CHANGELOG.md) file
and describe your changes. This makes the release proess much easier and
therefore helps to get your changes into a new `devrc` release faster.

The top of the `CHANGELOG` contains a *"unreleased"* section with a few
subsections. Please add your entry to the subsection
that best describes your change

Entries follow this format:

```
- Short description of what has been changed, see #123 (@user)
```
Here, `#123` is the number of the original issue and/or your pull request.
Please replace `@user` by your GitHub username.
