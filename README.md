Fast Rust File Identification — CLI & Library for Detection 🦀🧾
https://github.com/Vgate123/file-identify/releases

[![Release](https://img.shields.io/badge/releases-download-blue?logo=github&style=for-the-badge)](https://github.com/Vgate123/file-identify/releases) [![Crates.io](https://img.shields.io/crates/v/file-identify?style=for-the-badge)](https://crates.io) [![License](https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-lightgrey?style=for-the-badge)](LICENSE)

Hero image:  
![file-identify hero](https://raw.githubusercontent.com/github/explore/main/topics/file/file.png)

What this project does
- Provide a Rust library and a CLI to identify file types.
- Use file extensions, content analysis (magic bytes), and shebang parsing.
- Output MIME types, common extensions, and detection confidence.
- Offer a small, fast API for embedding detection in apps and scripts.

If you want a prebuilt binary, download the asset from the releases page and run it. Visit the releases page, pick the file for your OS, download, and execute it: https://github.com/Vgate123/file-identify/releases

Why use this tool
- Detect files where extensions lie.
- Map content to MIME types.
- Parse shebang lines to find interpreters.
- Ship a compact Rust crate for low-level systems work.
- Run a single binary from scripts or CI.

Key features
- Extension lookup table with common and rare types.
- Magic-byte detection for many formats (PNG, JPEG, PDF, ELF, ZIP, etc).
- Shebang parsing for scripts (bash, python, node, ruby, perl).
- JSON, plain, and table output modes for automation.
- Fast scanning with buffered reads and minimal allocations.
- Small binary size for containers and embedded systems.
- Library API that fits async and sync code.

Install (CLI)
- Visit the Releases page and download a build for your OS.
- Choose the binary for your platform on https://github.com/Vgate123/file-identify/releases
- Make the binary executable and run it.

Example Linux/macOS steps
```bash
# 1. Visit the Releases page: https://github.com/Vgate123/file-identify/releases
# 2. Download the binary for your platform from that page.
# 3. Make it executable and move to /usr/local/bin
chmod +x file-identify-linux
sudo mv file-identify-linux /usr/local/bin/file-identify
file-identify --help
```

Example Windows
- Download the .exe from the releases page.
- Run from PowerShell:
```powershell
.\file-identify-windows.exe --help
```

Quick CLI examples
- Detect a single file
```bash
file-identify detect README.md
# output
# name: README.md
# mime: text/markdown
# extension: md
# method: extension+content
```

- Recursive scan and JSON output
```bash
file-identify scan ./project --recursive --json > report.json
```

- Show MIME only
```bash
file-identify detect --mime file.bin
# application/octet-stream
```

Library (Rust) — quick start
- Add to Cargo.toml
```toml
[dependencies]
file-identify = "0.1"
```

- Basic detection
```rust
use file_identify::Detector;
use std::fs::File;

let mut f = File::open("script.sh")?;
let detector = Detector::default();
let info = detector.detect(&mut f)?;
println!("mime: {}", info.mime);
println!("extension: {}", info.ext.unwrap_or("unknown".into()));
```

API overview
- Detector::default() — load built-in tables.
- detect(reader) -> FileInfo — read a slice and return detection data.
- detect_path(path) -> FileInfo — convenience for path-based ops.
- detect_bytes(&[u8]) -> FileInfo — run on memory buffers.
- parse_shebang(line) -> ShebangInfo — parse exact shebang lines.

FileInfo fields
- mime: String (e.g., image/png)
- ext: Option<String> (e.g., png)
- confidence: f32 (0.0 to 1.0)
- method: String (extension, content, shebang, combination)
- description: Option<String> (human text)

Shebang parsing
- The tool reads the first line of text files.
- It recognizes common interpreters.
- It returns the interpreter name and any flags.

Example shebang detection
```bash
# script.sh content:
# !/usr/bin/env python3
file-identify detect script.sh
# method: shebang
# mime: text/x-python
# extension: py
```

Detection methods
- Extension: Map file extension to known types.
- Content: Read the first bytes and match magic signatures. This matches PNG (89 50 4E 47), PDF (25 50 44 46), ZIP/ODF, ELF, Mach-O, JPEG, and others.
- Shebang: Parse "#!" lines to map to script types.
- Heuristics: Text vs binary detection, XML vs HTML, CSV vs TSV.

Detection pipeline
1. If the file has a shebang and the content looks like text, prefer shebang for script types.
2. If a strong magic-byte match exists, use content.
3. Use extension as a tiebreaker.
4. Combine methods and assign a confidence score.

Output formats
- table (default) — human readable.
- json — machine friendly.
- mime — only MIME string.
- brief — one-line summary per file.

Examples
- Table output
```bash
file-identify detect test.png
# test.png  image/png  png  content:magic
```

- JSON output
```bash
file-identify detect --json code.js
# {
#   "path": "code.js",
#   "mime": "application/javascript",
#   "ext": "js",
#   "confidence": 0.95,
#   "method": "extension+content"
# }
```

Batch mode and performance
- Use scan to process folders. The tool uses a thread pool by default.
- Control threads with --threads N.
- It reads only the bytes needed for detection. It does not load full files.
- It avoids copies where possible to save memory.

Integration tips
- Use JSON output for CI and automation.
- Pipe file lists to xargs for large scans:
```bash
find . -type f | xargs -n 1 file-identify detect --json
```
- Use as a library to scan streams in network apps or file servers.

Common use cases
- Validate upload MIME vs extension.
- Build file indexers and search tools.
- Filter files in CI pipelines.
- Detect script interpreters in repositories.

Testing and CI
- Run unit tests
```bash
cargo test
```
- Run style checks
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

How to contribute
- Fork and open a PR.
- Add tests for new magic signatures.
- Update extension maps in data files under data/.
- Keep PRs small and focused.
- Run the test suite before submitting.

Data and signatures
- The project stores signature tables in a compact, human-editable format.
- Add signatures as short hex patterns and offsets.
- Map MIME types to one or more common extensions.

Logging and debug
- Use --verbose for more output.
- The library exposes a log hook so you can integrate it with env_logger or tracing.

Examples and scripts
- examples/cli shows real use cases.
- examples/lib shows how to use Detector with streams and async readers.
- See the examples folder for sample code.

Security notes
- The library reads file headers only by default.
- It does not run or execute file content.
- If you use the exec feature, audit any third-party assets before running.

Changelog and releases
- Find prebuilt binaries and release notes on the Releases page. Download the file for your OS and run it: https://github.com/Vgate123/file-identify/releases

License
- Dual licensed: MIT OR Apache-2.0. Pick the license that fits your project.

Contact and support
- Open an issue on the GitHub repo for bugs and feature requests.
- Send PRs for signature updates or new mappings.

Roadmap
- Add more magic signatures for archive and office formats.
- Improve CSV/TSV heuristics.
- Add optional WASM build for browser use.

Credits
- Built with Rust and the community crates that make binary I/O, testing, and CLI work simple.
- Icons and badges come from public sources.

Badge links
[![Download releases](https://img.shields.io/github/downloads/Vgate123/file-identify/total?style=for-the-badge)](https://github.com/Vgate123/file-identify/releases)

This README should give you all the steps to use the CLI and the library. Visit the releases page, download the binary for your platform, and execute it to try the tool: https://github.com/Vgate123/file-identify/releases