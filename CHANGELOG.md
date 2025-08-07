# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-07

### Added
- Initial Rust implementation of file identification library
- Core functionality ported from Python `identify` library
- Support for 316+ file extensions and format detection
- Shebang parsing for executable script identification
- Binary vs text content detection
- File system metadata analysis (permissions, file types)
- Command-line interface for file identification
- Comprehensive test suite with 59+ tests
- Complete API documentation with examples
- Support for Unix sockets, symlinks, and special file types

### Features
- `tags_from_path()` - comprehensive file analysis from filesystem
- `tags_from_filename()` - fast filename-only identification
- `tags_from_interpreter()` - interpreter-based script identification
- `file_is_text()` / `is_text()` - binary vs text detection
- `parse_shebang()` / `parse_shebang_from_file()` - shebang parsing
- CLI tool with JSON output and filename-only mode

### Supported File Types
- Programming languages: Python, JavaScript, Rust, Go, Java, C/C++, PHP, Ruby, Shell, etc.
- Configuration formats: JSON, YAML, TOML, XML, INI, etc.
- Documentation: Markdown, reStructuredText, AsciiDoc, etc.
- Build systems: Makefile, CMake, Bazel, Meson, etc.
- Container formats: Docker, Podman, etc.
- And 300+ more file types and extensions

[0.1.0]: https://github.com/pre-commit/identify/releases/tag/v0.1.0