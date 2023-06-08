# Directory Summarizer

Directory Summarizer is a simple application for summarizing the contents of a directory. It counts each filetype by extension and displays those counts in a table. You can preview it [here](https://goingforbrooke.github.io/directory_summarizer/).

## Installation

todo: write installation section in `README.md`

This section is a work in progress, but for now, check out the [Releases Page](https://github.com/goingforbrooke/directory_summarizer/releases).

## Usage

Launch the program, select the directory that you'd like to summarize, and click "Summarize" in the left pane. A table with counts of each filetype will appear in the right pane.

## Release

Use [cross](https://github.com/cross-rs/cross) to create release versions for MacOS and Windows.

Build for MacOS (Standard Processor):

```console
$ user@host: cross build --release --target aarch64-apple-darwin
Finished release [optimized] target(s) in 0.06s
```

Build for MacOS (ARM64/Apple Silicone):

```console
$ user@host: cross build --release --target x86_64-apple-darwin
Finished release [optimized] target(s) in 0.08s
```

Build for Windows:

```console
$ user@host: cross build --release --target x86_64-pc-windows-gnu
```

<<<<<<< HEAD
## CI/CD

Pushes to the `main` branch automatically trigger a release.

||||||| 743ae68
=======
## CI/CD

The [MacOS build-release pipeline](https://github.com/goingforbrooke/directory_summarizer/blob/cicd/increment_minor/.github/workflows/build_macos.yml) is triggered by pushes to the [`main` branch and any branch that starts with `cicd/`](https://github.com/goingforbrooke/directory_summarizer/blob/1c7f07ecf0671ead726bbca869e4025d4b8131c8/.github/workflows/build_macos.yml#L5-L6). This creates a [universal binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary) for MacOS (`aarch64-apple-darwin` for Apple Silicon and `x86_64-apple-darwin` for Intel) and melds them with [`lipo`](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary#Update-the-Architecture-List-of-Custom-Makefiles).

The resulting binary is placed in a [`*.app` bundle](https://developer.apple.com/documentation/bundleresources/placing_content_in_a_bundle), which is [codesigned and notarized](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/).

Then the workflow increments the application's [SemVer](https://semver.org) minor version in `Cargo.toml` by one and commits the change to the repo. The new version number's used to tag the commit and name the [release](https://github.com/goingforbrooke/directory_summarizer/releases).

>>>>>>> dev
## Misc.

Format inspired by [Make a README](https://www.makeareadme.com).

[Apple's documentation on how to create a certificate signing request on MacOS](https://developer.apple.com/help/account/create-certificates/create-a-certificate-signing-request)

[Reddit post on the difference between `.pkg`, `.dmg`, and `.app`](https://www.reddit.com/r/macsysadmin/comments/px1eae/difference_between_pkg_dmg_and_app_files/)

[Possible `lipo` alternative written in Go](https://github.com/konoui/lipo)
