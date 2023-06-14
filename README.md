# FolSum

FolSum is a simple application for summarizing the contents of a directory. It counts each filetype by extension and displays those counts in a table. You can preview it [here](https://goingforbrooke.github.io/folsum/).

## Installation

todo: write installation section in `README.md`

This section is a work in progress, but for now, check out the [Releases Page](https://github.com/goingforbrooke/folsum/releases).

## Usage

Launch the program, select the directory that you'd like to summarize, and click "Summarize" in the left pane. A table with counts of each filetype will appear in the right pane.

## Release

### `xtask`

```console
$ user@host: cargo xtask build
```

### `cargo build`

Build for MacOS (Intel x86_64):

```console
$ user@host: cargo build --release --target x86_64-apple-darwin
Finished release [optimized] target(s) in 0.06s
```

Build for MacOS (ARM64/Apple Silicone):

```console
$ user@host: cargo build --release --target aarch64-apple-darwin
Finished release [optimized] target(s) in 0.08s
```

Build for MacOS (Intel x86_64 and ARM64/Apple Silicone):

```console
$ user@host: cargo build --release --target x86_64-apple-darwin --target aarch64-apple-darwin
Finished release [optimized] target(s) in 0.08s
```

Create universal MacOS binary (MacOS only):

```console
$ user@host: lipo -create -output target/release/folsum -arch x86_64 target/x86_64-apple-darwin/release/folsum -arch arm64 target/aarch64-apple-darwin/release/folsum
```

Check architectures on universal MacOS binary:

```console
$ user@host lipo -archs target/release/bundle/osx/folsum.app/Contents/MacOS/folsum
x86_64 arm64
```

Build for Windows:

```console
$ user@host: cargo build --release --target x86_64-pc-windows-gnu
```

## CI/CD

The [MacOS build-release pipeline](https://github.com/goingforbrooke/folsum/blob/cicd/increment_minor/.github/workflows/build_macos.yml) is triggered by pushes to the [`main` branch and any branch that starts with `cicd/`](https://github.com/goingforbrooke/folsum/blob/1c7f07ecf0671ead726bbca869e4025d4b8131c8/.github/workflows/build_macos.yml#L5-L6).

The build creates a [universal binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary) for MacOS by creating a binary for each processor architecture (`aarch64-apple-darwin` for Apple Silicon and `x86_64-apple-darwin` for Intel). These binaries are melded with [`lipo`](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary#Update-the-Architecture-List-of-Custom-Makefiles) to create a MacOS universal binary. The `lipo` command is specific to MacOS runners, but there are [cheaper alternatives](https://github.com/rust-lang/cargo/issues/8875#issuecomment-1583958357).

We use [Cargo Bundle](https://github.com/burtonageo/cargo-bundle) to [create](https://github.com/burtonageo/cargo-bundle/blob/master/src/bundle/osx_bundle.rs) an [`*.app` bundle](https://developer.apple.com/documentation/bundleresources/placing_content_in_a_bundle) and plist file. Then we tell Cargo Bundle to skip building new binaries and place our universal binary (from the previous step) where Cargo Bundle expects to find it.

The final `*.app` bundle is [codesigned and notarized](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/) so MacOS doesn't think that it's malware.

If the commit was pushed to the `main` branch, then we use [Cargo Edit](https://github.com/killercup/cargo-edit) to increment the [SemVer](https://semver.org) minor version in `Cargo.toml` by one and commit the change to the repo. The new version number's used to tag the commit and name the [release](https://github.com/goingforbrooke/folsum/releases).

Otherwise, if the commit was pushed to a branch starting with `cicd`, then we skip incrementing the minor version. In addition, pushes to any non-`main` branch (including those starting with `cicd`) will create a "draft" release instead of a regular release. Note that this doesn't override the top-level branch filter-- builds are only triggered by pushes to `main` or branches that start with `cicd`. These draft releases won't fail when the release name (defined by the non-incremented SemVer tag) already exists. This makes it easy to hack on the CI/CD pipeline without messing up production builds.

## Misc.

Format inspired by [Make a README](https://www.makeareadme.com).

[Apple's documentation on how to create a certificate signing request on MacOS](https://developer.apple.com/help/account/create-certificates/create-a-certificate-signing-request)

[Reddit post on the difference between `.pkg`, `.dmg`, and `.app`](https://www.reddit.com/r/macsysadmin/comments/px1eae/difference_between_pkg_dmg_and_app_files/)

[Possible `lipo` alternative written in Go](https://github.com/konoui/lipo)
