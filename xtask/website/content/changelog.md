+++
title = "Changelog"
description = "Changelog"
weight = 2
+++

# v2.2.1 - 2025-04-04

### Fixed

- Fix a bug where FolSum would crash if no manifest files existed

# v2.2.0 - 2025-04-03

### Added

- Add MD5 hash to GUI table
 
### Changed

- Remove "export" button and make manifests creation implicit (in-directory)

# v2.1.0 - 2025-03-13

### Added

- Add directory verification with MD5 hash and file path (presence) as checkers
- Add performance benchmarking

## Changed

- Change instances of "directory" to "folder" in GUI

## v2.0.3 - 2024-12-05

### Added

- Export table results to CSV format.
- User-friendly website

### Fixed

- Fix MacOS builds
- Don't crash when "summarize" is clicked with no target folder specified.

### Changed

- Pivot: Move from counting file extensions to verifying files
- Upgraded dependencies.
