# file-identify

[![Crates.io](https://img.shields.io/crates/v/file-identify.svg)](https://crates.io/crates/file-identify)
[![Documentation](https://docs.rs/file-identify/badge.svg)](https://docs.rs/file-identify)
[![CI](https://github.com/grok-rs/file-identify/workflows/CI/badge.svg)](https://github.com/grok-rs/file-identify/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

File identification library for Rust.

Given a file (or some information about a file), return a set of standardized tags identifying what the file is.

This is a Rust port of the Python [identify](https://github.com/pre-commit/identify) library.

## Features

- 🚀 **Fast**: Built in Rust with Perfect Hash Functions (PHF) for compile-time optimization
- 📁 **Comprehensive**: Identifies 315+ file types and formats
- 🔍 **Smart detection**: Uses file extensions, content analysis, and shebang parsing
- 📦 **Library + CLI**: Use as a Rust library or command-line tool
- ⚡ **Zero overhead**: PHF provides O(1) lookups with no runtime hash computation
- 🎯 **Memory efficient**: Static data structures with no lazy initialization
- ✅ **Well-tested**: Extensive test suite ensuring reliability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
file-identify = "0.1.1"
```

## Usage

### Library Usage

```rust
use file_identify::{tags_from_path, tags_from_filename, tags_from_interpreter};

// Identify a file from its path
let tags = tags_from_path("/path/to/file.py").unwrap();
println!("{:?}", tags); // {"file", "text", "python", "non-executable"}

// Identify from filename only
let tags = tags_from_filename("script.sh");
println!("{:?}", tags); // {"text", "shell", "bash"}

// Identify from interpreter
let tags = tags_from_interpreter("python3");
println!("{:?}", tags); // {"python", "python3"}
```

### Command Line Usage

```bash
# Install the CLI tool
cargo install file-identify

# Identify a file
file-identify setup.py
["file", "non-executable", "python", "text"]

# Use filename only (don't read file contents)
file-identify --filename-only setup.py
["python", "text"]

# Get help
file-identify --help
```

## How it works

A call to `tags_from_path` does this:

1. What is the type: file, symlink, directory? If it's not file, stop here.
2. Is it executable? Add the appropriate tag.
3. Do we recognize the file extension? If so, add the appropriate tags, stop here. These tags would include binary/text.
4. Peek at the first 1KB of the file. Use these to determine whether it is binary or text, add the appropriate tag.
5. If identified as text above, try to read and interpret the shebang, and add appropriate tags.

By design, this means we don't need to partially read files where we recognize the file extension.

## Development

### Setup

```bash
# Clone the repository
git clone git@github.com:grok-rs/file-identify.git
cd file-identify

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- path/to/file
```

### Pre-commit hooks

This project uses pre-commit hooks to ensure code quality:

```bash
pip install pre-commit
pre-commit install
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```

### Releasing

To create a new release:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with new version notes
3. Commit changes: `git commit -am "Bump version to x.y.z"`
4. Create and push tag: `git tag vx.y.z && git push origin vx.y.z`

The release workflow will automatically:
- Run full test suite
- Publish to crates.io using `RELEASE_TOKEN` secret
- Create GitHub release with auto-generated notes

## License

MIT