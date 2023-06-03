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

## Misc.

Format inspired by [Make a README](https://www.makeareadme.com).

[Blog post on how to sign binaries](https://federicoterzi.com/blog/automatic-code-signing-and-notarization-for-macos-apps-using-github-actions/)

[Apple's documentation on how to create a certificate signing request on MacOS](https://developer.apple.com/help/account/create-certificates/create-a-certificate-signing-request)
