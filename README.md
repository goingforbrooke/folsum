# Directory Summarizer

Directory Summarizer is a simple application for summarizing the contents of a directory. It counts each filetype by extension and displays those counts in a table. You can preview it [here](https://goingforbrooke.github.io/directory_summarizer/).

## Installation

todo: write installation section in `README.md`

This section is a work in progress, but for now, check out the [Releases Page](https://github.com/goingforbrooke/directory_summarizer/releases).

## Usage

Launch the program, select the directory that you'd like to summarize, and click "Summarize" in the left pane. A table with counts of each filetype will appear in the right pane.

## Release

Use [cross](https://github.com/cross-rs/cross) to create release versions for MacOS and Windows.

Build for MacOS:
```console
$ user@host: cross build --release --target x86_64-apple-darwin
```

Build for Windows:
```console
$ user@host: cross build --release --target x86_64-pc-windows-gnu
```

## Misc.

Format inspired by [Make a README](https://www.makeareadme.com).