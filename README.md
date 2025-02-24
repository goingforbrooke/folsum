<img src="folsum/images/icons/resized_icons/folsum_icon_128px.png" align="right" />

# 🗂️ FolSum

FolSum is a simple application for summarizing the contents of a directory. It counts each filetype by extension and displays those counts in a table. You can preview it on [folsum.goingforbrooke.com](https://folsum.goingforbrooke.com/).

## 🖥️ Installation

todo: write installation section in `README.md`

This section is a work in progress, but for now, check out the [Releases Page](https://github.com/goingforbrooke/folsum/releases).

## 🖱️ Usage

Launch the program, select the directory that you'd like to summarize, and click "Summarize" in the left pane. A table with counts of each filetype will appear in the right pane.

## 🛠️ Contributing

### 🌳 Branch Naming Conventions

Branch names follow the pattern **prefix** `/` **branch_name**. In practice, this looks like `goingforbrooke/folsum/tree` `/` **branch_prefix** `/` **branch_name**.

The **branch_prefix** depends on the purpose of the branch, described in the table below. Every branch must have a prefix, except for `main` and `dev`, which can only be merged into.

For example, a branch that fixes a launch issue might look like `fix/launch_broken` and a branch that adds a green button might look like `feat/green_button`. This style is consistent with Git's **repo**`/`**branch** style and GitHub-style URLs.

Each branch prefix triggers different things in the CI/CD workflow.

| ❓ | Prefix | Purpose | CI/CD Trigger |
| ------------- | ------------- | ------------- | ------------- |
| 📦 | `main` | Publish [releases](https://github.com/goingforbrooke/folsum/releases) | [Increment minor/patch version and publish a standard release](https://github.com/goingforbrooke/folsum/blob/cicd/increment_minor/.github/workflows/build_macos.yml) |
| 🛠️ | `dev` | Development | Implicitly bump minor/patch version after `--no-ff` merge to `main` |
| ✨ | `feat/*` | Add features  | Implicitly bump minor/patch version after `--no-ff` merge to `dev`, then `main` |
| 🪲 | `fix/*` | Fix bugs | Explicitly bump patch version after `--no-ff` merge to `dev`, then `main` |
| 👷🏼‍️ | `cicd/*` | Develop and test CI/CD pipelines | Immediately publish a draft release and skip bumping minor/patch version  |
| 📚 | `doc/*` | Change `README.md` | Implicitly bump minor/patch version after `--no-ff` merge to `dev`, then `main` |
| 🧹 | `internal/*` | Refactoring and quality of life improvements | Implicitly bump minor/patch version after `--no-ff` merge to `dev`, then `main` |

### 🔩 Dependencies

Adding dependencies with `xtask` looks a little different than normal.

For example, to add the `chrono` crate as a dependency to Folsum, use `--package folsum`.

```console
$ user@host: cargo add --package folsum chrono
```

For example, to add the `chrono` crate as a dependency to the build tools, use `--package xtask`.

```console
$ user@host: cargo add --package xtask chrono
```

### 🏁 `xtask`

```console
$ user@host: cargo xtask build
```

### ☑️ `cargo check`

Check compilation for all targets.

```console
$ user@host: cargo check --target x86_64-apple-darwin --target aarch64-apple-darwin --target wasm32-unknown-unknown --target x86_64-pc-windows-gnu
```

### 🌎 Preview WASM Builds

```console
$ user@host: trunk serve --open
```

### 📦 `cargo build`

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

**Build for Linux x86_64 (on Apple Silicon):**

1. Install `main` branch of `cross` because it has aarch64 support.

```console
$ user@host: cargo install cross --git https://github.com/cross-rs/cross
```

2. Ensure that Docker's running.

3. Build with `cross`.

```console
$ user@host: cross build --release --target x86_64-unknown-linux-gnu
```

Expect to find the binary at `folsum/target/x86_64-unknown-linux-gnu/release/folsum`.

**Build for Linux aarch64 (on Apple Silicon):**

1. Install `main` branch of `cross` because it has aarch64 support.

```console
$ user@host: cargo install cross --git https://github.com/cross-rs/cross
```

2. Ensure that Docker's running.

3. Build with `cross`.

```console
$ user@host: cross build --release --target aarch64-unknown-linux-gnu
```

**Packaging `.deb`**

For `.deb` packaging on Apple Silicon, use the [`package_deb.Dockerfile`](./xtask/docker/package_deb.Dockerfile).

**Build for WASM:**

```text
$ user@host: trunk build
```

Expect to find static files in `public/`.

## 🏗️ CI/CD

The [MacOS build-release pipeline](https://github.com/goingforbrooke/folsum/blob/cicd/increment_minor/.github/workflows/build_macos.yml) is triggered by pushes to the [`main` branch and any branch that starts with `cicd/`](https://github.com/goingforbrooke/folsum/blob/1c7f07ecf0671ead726bbca869e4025d4b8131c8/.github/workflows/build_macos.yml#L5-L6).

The build creates a [universal binary](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary) for MacOS by creating a binary for each processor architecture (`aarch64-apple-darwin` for Apple Silicon and `x86_64-apple-darwin` for Intel). These binaries are melded with [`lipo`](https://developer.apple.com/documentation/apple-silicon/building-a-universal-macos-binary#Update-the-Architecture-List-of-Custom-Makefiles) to create a MacOS universal binary. The `lipo` command is specific to MacOS runners, but there are [cheaper alternatives](https://github.com/rust-lang/cargo/issues/8875#issuecomment-1583958357).

We use [Cargo Bundle](https://github.com/burtonageo/cargo-bundle) to [create](https://github.com/burtonageo/cargo-bundle/blob/master/src/bundle/osx_bundle.rs) an [`*.app` bundle](https://developer.apple.com/documentation/bundleresources/placing_content_in_a_bundle) and plist file. Then we tell Cargo Bundle to skip building new binaries and place our universal binary (from the previous step) where Cargo Bundle expects to find it.

The final `*.app` bundle is [codesigned and notarized](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/) so MacOS doesn't think that it's malware.

If the commit was pushed to the `main` branch, then we bump the minor version or the patch version. The type of bump depends on whether the last branch merged to `dev` starts with `fix/*`. If it does, then we use [Cargo Edit](https://github.com/killercup/cargo-edit) to increment the [SemVer](https://semver.org) patch version in `Cargo.toml` by one. Otherwise, we default to incrementing the minor version in `Cargo.toml` by one.

Whether the minor or patch version was incremented, the change to `Cargo.toml`'s committed to the repo. Then we tag the commit with the new version number and use that tag to name the new [release](https://github.com/goingforbrooke/folsum/releases). It's imperative that the version changes for each release because release names must be unique. This ensures that previous releases aren't overwritten.

Otherwise, if the commit was pushed to a branch starting with `cicd`, then we skip incrementing the minor version. In addition, pushes to any non-`main` branch (including those starting with `cicd`) will create a "draft" release (which won't be visible to others in the FolSum repo's "Releases" page because it's a draft release) instead of a regular release. Note that this doesn't override the top-level branch filter-- builds are only triggered by pushes to `main` or branches that start with `cicd`. These draft releases won't fail when the release name (defined by the non-incremented SemVer tag) already exists. This makes it easy to hack on the CI/CD pipeline without messing up production builds.

## 📐 Design Decisions

### 👷🏼‍♀️ Xtask for Builds

On branch `internal/xtask_postbuild`, most of the project was moved from the root directory (`folsum/`) into a new subdirectory (`folsum/folsum/`) so the [xtask pattern](https://github.com/matklad/cargo-xtask/tree/master) can be used for post-build actions. Build scripts like [`build.rs` run before compilation](https://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts), so it's not possible to bundle (MacOS universal) binaries into a `.app` deliverable with `cargo build`.

> Placing a file named build.rs in the root of a package will cause Cargo to compile that script and execute it just before building the package. -- [Rust docs](https://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts)

Post-build scripts are an [ongoing discussion](https://github.com/rust-lang/cargo/issues/545#issuecomment-895293171) in the Rust community and xtask looks like the best solutionhttps://doc.rust-lang.org/cargo/reference/build-scripts.html#build-scripts apart from Github Actions. The xtask pattern is defined [here](https://github.com/matklad/cargo-xtask), but we used [this example](https://github.com/nickgerace/cargo-xtask-example) to implement it because it's more up-to-date.

### 🐂 Tauri Bundler/Cargo Bundle for Bundling

Whether Folsum evolves to use [Cargo Bundle](https://crates.io/crates/cargo-bundle) or (continues to use) [Tauri Bundler](https://crates.io/crates/tauri-bundler), post-build scripts will be necessary. Tauri Bundler is more mature with more supported platforms, but Cargo Bundle (from which Tauri Bundler is forked) is more Rust-centric. This is because Cargo Bunndle uses `Cargo.toml` for bundle configuration without using Tauri's CLI to fill missing values

### 🧟‍♀️ Xtask and Tauri Bundler Together

Since we're rolling our own build scripts in Rust, we use [Tauri Bundler](https://crates.io/crates/tauri-bundler)'s API, which is very close to [Cargo Bundle](https://crates.io/crates/cargo-bundle) API, sans `Cargo.toml` configuration extraction. We might've stuck with the (initial) Cargo Bundle implementation if we had figured out the icon sizing issues sooner. Instead, we'll go with Tauri Bundler for now and slowly PR-patch our way back to Cargo Bundle.

As of `v2.0.0`, the Actions CI pipeline uses Cargo Bundle (via CLI) and the (local) xtask pipeline uses Tauri Bundle (via API interface).

Xtask requires no extra dependencies for implementing post-build actions. It uses what Cargo already offers. [In the author's words](https://github.com/nickgerace/cargo-xtask-example#why-cargo-xtask),

> Using external build systems and scripting languages can be useful, but using these technologies can result in inaccessible contributing experiences and potentially locking out valid development environments.

> Since cargo is the tried and true build system for Rust (tested on multiple tiered targets), we can get the best of both worlds by using a small wrapper around it. Thus, cargo xtask exists to fill the gap; allowing for repository automation without needing to install another dependency.

## 🐭 Misc.

[Apple's documentation on how to create a certificate signing request on MacOS](https://developer.apple.com/help/account/create-certificates/create-a-certificate-signing-request)

[Reddit post on the difference between `.pkg`, `.dmg`, and `.app`](https://www.reddit.com/r/macsysadmin/comments/px1eae/difference_between_pkg_dmg_and_app_files/)

[Possible `lipo` alternative written in Go](https://github.com/konoui/lipo)

### 🔌 Compatibility

FolSum requires no external dependencies to run.

### 🙏🏻 Kudos

Readme format inspired by [Make a README](https://www.makeareadme.com) and [awesome-readme](https://github.com/matiassingers/awesome-readme/tree/master).

Changelog format inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## 🛟 Support

If you need to contact the developers, then file an issue with one of these [labels](https://github.com/goingforbrooke/folsum/labels):

- 🪲 Bug: https://github.com/goingforbrooke/folsum/labels/bug
- ✨ Feature: https://github.com/goingforbrooke/folsum/labels/feature
- 🙋🏼‍♀️ Question: https://github.com/goingforbrooke/folsum/labels/question

# 🪪 License

[MIT](./LICENSE.md)
