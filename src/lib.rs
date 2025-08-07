//! # file-identify
//!
//! A Rust library for identifying file types based on extensions, content, and shebangs.
//!
//! This library provides a comprehensive way to identify files by analyzing:
//! - File extensions and special filenames
//! - File content (binary vs text detection)
//! - Shebang lines for executable scripts
//! - File system metadata (permissions, file type)
//!
//! ## Quick Start
//!
//! ```rust
//! use file_identify::{tags_from_path, tags_from_filename};
//!
//! // Identify a Python file
//! let tags = tags_from_filename("script.py");
//! assert!(tags.contains("python"));
//! assert!(tags.contains("text"));
//!
//! // Identify from filesystem path
//! # use std::fs;
//! # use tempfile::tempdir;
//! # let dir = tempdir().unwrap();
//! # let file_path = dir.path().join("test.py");
//! # fs::write(&file_path, "print('hello')").unwrap();
//! let tags = tags_from_path(&file_path).unwrap();
//! assert!(tags.contains("file"));
//! assert!(tags.contains("python"));
//! ```
//!
//! ## Tag System
//!
//! Files are identified using a set of standardized tags:
//!
//! - **Type tags**: `file`, `directory`, `symlink`, `socket`
//! - **Mode tags**: `executable`, `non-executable`
//! - **Encoding tags**: `text`, `binary`
//! - **Language/format tags**: `python`, `javascript`, `json`, `xml`, etc.
//!
//! ## Error Handling
//!
//! Functions that access the filesystem return [`Result`] types. The main error
//! conditions are:
//!
//! - [`IdentifyError::PathNotFound`] - when the specified path doesn't exist
//! - [`IdentifyError::IoError`] - for other I/O related errors

use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

pub mod extensions;
pub mod interpreters;
pub mod tags;

use extensions::{EXTENSIONS, EXTENSIONS_NEED_BINARY_CHECK, NAMES};
use interpreters::INTERPRETERS;
use tags::*;

/// Result type for file identification operations.
///
/// This is a convenience type alias for operations that may fail with
/// file system or parsing errors.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Errors that can occur during file identification.
#[derive(Debug)]
pub enum IdentifyError {
    /// The specified path does not exist on the filesystem.
    PathNotFound(String),
    /// An I/O error occurred while accessing the file.
    IoError(std::io::Error),
}

impl std::fmt::Display for IdentifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentifyError::PathNotFound(path) => write!(f, "{path} does not exist."),
            IdentifyError::IoError(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for IdentifyError {}

impl From<std::io::Error> for IdentifyError {
    fn from(err: std::io::Error) -> Self {
        IdentifyError::IoError(err)
    }
}

/// Identify a file from its filesystem path.
///
/// This is the most comprehensive identification method, providing a superset
/// of information from other methods. It analyzes:
///
/// 1. File type (regular file, directory, symlink, socket)
/// 2. File permissions (executable vs non-executable)
/// 3. Filename and extension patterns
/// 4. File content (binary vs text detection)
/// 5. Shebang lines for executable files
///
/// # Arguments
///
/// * `path` - Path to the file to identify
///
/// # Returns
///
/// A set of tags identifying the file type and characteristics.
///
/// # Errors
///
/// Returns [`IdentifyError::PathNotFound`] if the path doesn't exist, or
/// [`IdentifyError::IoError`] for other I/O failures.
///
/// # Examples
///
/// ```rust
/// use file_identify::tags_from_path;
/// # use std::fs;
/// # use tempfile::tempdir;
///
/// # let dir = tempdir().unwrap();
/// # let file_path = dir.path().join("script.py");
/// # fs::write(&file_path, "#!/usr/bin/env python3\nprint('hello')").unwrap();
/// let tags = tags_from_path(&file_path).unwrap();
/// assert!(tags.contains("file"));
/// assert!(tags.contains("python"));
/// assert!(tags.contains("text"));
/// ```
pub fn tags_from_path<P: AsRef<Path>>(path: P) -> Result<TagSet> {
    let path = path.as_ref();
    let path_str = path.to_string_lossy();

    let metadata = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return Err(Box::new(IdentifyError::PathNotFound(path_str.to_string()))),
    };

    let file_type = metadata.file_type();

    if file_type.is_dir() {
        return Ok([DIRECTORY].iter().cloned().collect());
    }
    if file_type.is_symlink() {
        return Ok([SYMLINK].iter().cloned().collect());
    }

    // Check for socket (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if file_type.is_socket() {
            return Ok([SOCKET].iter().cloned().collect());
        }
    }

    let mut tags = TagSet::new();
    tags.insert(FILE);

    // Check if executable
    let is_executable = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        }
        #[cfg(not(unix))]
        {
            // On non-Unix systems, check file extension for common executables
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd"))
                .unwrap_or(false)
        }
    };

    if is_executable {
        tags.insert(EXECUTABLE);
    } else {
        tags.insert(NON_EXECUTABLE);
    }

    // Check filename-based tags
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        let filename_tags = tags_from_filename(filename);
        if !filename_tags.is_empty() {
            tags.extend(filename_tags);
        } else if is_executable {
            // Parse shebang for executable files without recognized extensions
            if let Ok(shebang_tags) = parse_shebang_from_file(path) {
                tags.extend(shebang_tags);
            }
        }
    }

    // Check if we need to determine binary vs text
    if !tags.iter().any(|tag| ENCODING_TAGS.contains(tag)) {
        if file_is_text(path)? {
            tags.insert(TEXT);
        } else {
            tags.insert(BINARY);
        }
    }

    Ok(tags)
}

/// Identify a file based only on its filename.
///
/// This method analyzes the filename and extension to determine file type,
/// without accessing the filesystem. It's useful when you only have the
/// filename or want to avoid I/O operations.
///
/// # Arguments
///
/// * `filename` - The filename to analyze (can include path)
///
/// # Returns
///
/// A set of tags identifying the file type. Returns an empty set if
/// the filename is not recognized.
///
/// # Examples
///
/// ```rust
/// use file_identify::tags_from_filename;
///
/// let tags = tags_from_filename("script.py");
/// assert!(tags.contains("python"));
/// assert!(tags.contains("text"));
///
/// let tags = tags_from_filename("Dockerfile");
/// assert!(tags.contains("dockerfile"));
///
/// let tags = tags_from_filename("unknown.xyz");
/// assert!(tags.is_empty());
/// ```
pub fn tags_from_filename(filename: &str) -> TagSet {
    let mut tags = TagSet::new();

    // Check exact filename matches first
    for part in std::iter::once(filename).chain(filename.split('.')) {
        if let Some(name_tags) = NAMES.get(part) {
            tags.extend(name_tags.iter().cloned());
            break;
        }
    }

    // Check file extension
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();

        if let Some(ext_tags) = EXTENSIONS.get(ext_lower.as_str()) {
            tags.extend(ext_tags.iter().cloned());
        } else if let Some(ext_tags) = EXTENSIONS_NEED_BINARY_CHECK.get(ext_lower.as_str()) {
            tags.extend(ext_tags.iter().cloned());
        }
    }

    tags
}

/// Identify tags based on a shebang interpreter.
///
/// This function analyzes interpreter names from shebang lines to determine
/// the script type. It handles version-specific interpreters by progressively
/// removing version suffixes.
///
/// # Arguments
///
/// * `interpreter` - The interpreter name or path from a shebang
///
/// # Returns
///
/// A set of tags for the interpreter type. Returns an empty set if
/// the interpreter is not recognized.
///
/// # Examples
///
/// ```rust
/// use file_identify::tags_from_interpreter;
///
/// let tags = tags_from_interpreter("python3.11");
/// assert!(tags.contains("python"));
/// assert!(tags.contains("python3"));
///
/// let tags = tags_from_interpreter("/usr/bin/bash");
/// assert!(tags.contains("shell"));
/// assert!(tags.contains("bash"));
///
/// let tags = tags_from_interpreter("unknown-interpreter");
/// assert!(tags.is_empty());
/// ```
pub fn tags_from_interpreter(interpreter: &str) -> TagSet {
    // Extract the interpreter name from the path
    let interpreter_name = interpreter.split('/').next_back().unwrap_or(interpreter);

    // Try progressively shorter versions (e.g., "python3.5.2" -> "python3.5" -> "python3")
    let mut current = interpreter_name;
    while !current.is_empty() {
        if let Some(tags) = INTERPRETERS.get(current) {
            return tags.clone();
        }

        // Try removing the last dot-separated part
        match current.rfind('.') {
            Some(pos) => current = &current[..pos],
            None => break,
        }
    }

    TagSet::new()
}

/// Determine if a file contains text or binary data.
///
/// This function reads the first 1KB of a file to determine if it contains
/// text or binary data, using a similar algorithm to the `file` command.
///
/// # Arguments
///
/// * `path` - Path to the file to analyze
///
/// # Returns
///
/// `true` if the file appears to contain text, `false` if binary.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
///
/// # Examples
///
/// ```rust
/// use file_identify::file_is_text;
/// # use std::fs;
/// # use tempfile::tempdir;
///
/// # let dir = tempdir().unwrap();
/// # let text_path = dir.path().join("text.txt");
/// # fs::write(&text_path, "Hello, world!").unwrap();
/// assert!(file_is_text(&text_path).unwrap());
///
/// # let binary_path = dir.path().join("binary.bin");
/// # fs::write(&binary_path, &[0x7f, 0x45, 0x4c, 0x46]).unwrap();
/// assert!(!file_is_text(&binary_path).unwrap());
/// ```
pub fn file_is_text<P: AsRef<Path>>(path: P) -> Result<bool> {
    let file = fs::File::open(path)?;
    is_text(file)
}

/// Determine if data from a reader contains text or binary content.
///
/// This function reads up to 1KB from the provided reader and analyzes
/// the bytes to determine if they represent text or binary data.
///
/// # Arguments
///
/// * `reader` - A reader providing the data to analyze
///
/// # Returns
///
/// `true` if the data appears to be text, `false` if binary.
///
/// # Examples
///
/// ```rust
/// use file_identify::is_text;
/// use std::io::Cursor;
///
/// let text_data = Cursor::new(b"Hello, world!");
/// assert!(is_text(text_data).unwrap());
///
/// let binary_data = Cursor::new(&[0x7f, 0x45, 0x4c, 0x46, 0x00]);
/// assert!(!is_text(binary_data).unwrap());
/// ```
pub fn is_text<R: Read>(mut reader: R) -> Result<bool> {
    let mut buffer = [0; 1024];
    let bytes_read = reader.read(&mut buffer)?;

    // Check for null bytes or other non-text indicators
    let text_chars: HashSet<u8> = [
        7, 8, 9, 10, 11, 12, 13, 27, // Control chars
    ]
    .iter()
    .cloned()
    .chain(0x20..0x7F) // ASCII printable
    .chain(0x80..=0xFF) // Extended ASCII
    .collect();

    let is_text = buffer[..bytes_read]
        .iter()
        .all(|&byte| text_chars.contains(&byte));
    Ok(is_text)
}

/// Parse shebang line from an executable file and return interpreter tags.
///
/// This function reads the first line of an executable file to extract
/// shebang information and determine the script interpreter.
///
/// # Arguments
///
/// * `path` - Path to the executable file
///
/// # Returns
///
/// A set of tags for the interpreter found in the shebang line.
/// Returns an empty set if:
/// - The file is not executable
/// - No shebang is found
/// - The interpreter is not recognized
///
/// # Errors
///
/// Returns an error if the file cannot be accessed or read.
///
/// # Examples
///
/// ```rust
/// use file_identify::parse_shebang_from_file;
/// # use std::fs;
/// # use std::os::unix::fs::PermissionsExt;
/// # use tempfile::tempdir;
///
/// # let dir = tempdir().unwrap();
/// # let script_path = dir.path().join("script");
/// # fs::write(&script_path, "#!/usr/bin/env python3\nprint('hello')").unwrap();
/// # let mut perms = fs::metadata(&script_path).unwrap().permissions();
/// # perms.set_mode(0o755);
/// # fs::set_permissions(&script_path, perms).unwrap();
/// let tags = parse_shebang_from_file(&script_path).unwrap();
/// assert!(tags.contains("python"));
/// ```
pub fn parse_shebang_from_file<P: AsRef<Path>>(path: P) -> Result<TagSet> {
    let path = path.as_ref();

    // Only check executable files
    let metadata = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Ok(TagSet::new());
        }
    }

    let file = fs::File::open(path)?;
    parse_shebang(file)
}

/// Parse a shebang line from a reader and return interpreter tags.
///
/// This function reads the first line from the provided reader and parses
/// it as a shebang line to determine the script interpreter.
///
/// # Arguments
///
/// * `reader` - A reader providing the file content
///
/// # Returns
///
/// A set of tags for the interpreter found in the shebang line.
/// Returns an empty set if no valid shebang is found.
///
/// # Examples
///
/// ```rust
/// use file_identify::parse_shebang;
/// use std::io::Cursor;
///
/// let shebang = Cursor::new(b"#!/usr/bin/env python3\nprint('hello')");
/// let tags = parse_shebang(shebang).unwrap();
/// assert!(tags.contains("python"));
/// assert!(tags.contains("python3"));
///
/// let no_shebang = Cursor::new(b"print('hello')");
/// let tags = parse_shebang(no_shebang).unwrap();
/// assert!(tags.is_empty());
/// ```
pub fn parse_shebang<R: Read>(reader: R) -> Result<TagSet> {
    let mut buf_reader = BufReader::new(reader);
    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line)?;

    if !first_line.starts_with("#!") {
        return Ok(TagSet::new());
    }

    // Remove the #! and clean up the line
    let shebang_line = first_line[2..].trim();

    // Parse the shebang command
    let parts: Vec<&str> = shebang_line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(TagSet::new());
    }

    let cmd = if parts.len() >= 2 && parts[0] == "/usr/bin/env" {
        if parts[1] == "-S" && parts.len() > 2 {
            &parts[2..]
        } else {
            &parts[1..]
        }
    } else {
        &parts
    };

    if cmd.is_empty() {
        return Ok(TagSet::new());
    }

    // Extract interpreter name and get tags
    let interpreter = cmd[0].split('/').next_back().unwrap_or(cmd[0]);
    Ok(tags_from_interpreter(interpreter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::{NamedTempFile, tempdir};

    // Test tag system completeness
    #[test]
    fn test_all_basic_tags_exist() {
        assert!(TYPE_TAGS.contains("file"));
        assert!(TYPE_TAGS.contains("directory"));
        assert!(MODE_TAGS.contains("executable"));
        assert!(ENCODING_TAGS.contains("text"));
    }

    #[test]
    fn test_tag_groups_are_disjoint() {
        assert!(TYPE_TAGS.is_disjoint(&MODE_TAGS));
        assert!(TYPE_TAGS.is_disjoint(&ENCODING_TAGS));
        assert!(MODE_TAGS.is_disjoint(&ENCODING_TAGS));
    }

    // Test tags_from_filename with various scenarios
    #[test]
    fn test_tags_from_filename_basic() {
        let tags = tags_from_filename("file.py");
        assert!(tags.contains("text"));
        assert!(tags.contains("python"));
    }

    #[test]
    fn test_tags_from_filename_special_names() {
        let tags = tags_from_filename("Dockerfile");
        assert!(tags.contains("dockerfile"));
        assert!(tags.contains("text"));

        let tags = tags_from_filename("Makefile");
        assert!(tags.contains("makefile"));
        assert!(tags.contains("text"));

        let tags = tags_from_filename("Cargo.toml");
        assert!(tags.contains("toml"));
        assert!(tags.contains("cargo"));
    }

    #[test]
    fn test_tags_from_filename_case_insensitive_extension() {
        let tags = tags_from_filename("image.JPG");
        assert!(tags.contains("binary"));
        assert!(tags.contains("image"));
        assert!(tags.contains("jpeg"));
    }

    #[test]
    fn test_tags_from_filename_precedence() {
        // setup.cfg should match by name, not .cfg extension
        let tags = tags_from_filename("setup.cfg");
        assert!(tags.contains("ini"));
    }

    #[test]
    fn test_tags_from_filename_complex_names() {
        let tags = tags_from_filename("Dockerfile.xenial");
        assert!(tags.contains("dockerfile"));

        let tags = tags_from_filename("README.md");
        assert!(tags.contains("markdown"));
        assert!(tags.contains("plain-text"));
    }

    #[test]
    fn test_tags_from_filename_unrecognized() {
        let tags = tags_from_filename("unknown.xyz");
        assert!(tags.is_empty());

        let tags = tags_from_filename("noextension");
        assert!(tags.is_empty());
    }

    // Test tags_from_interpreter
    #[test]
    fn test_tags_from_interpreter_basic() {
        let tags = tags_from_interpreter("python3");
        assert!(tags.contains("python"));
        assert!(tags.contains("python3"));
    }

    #[test]
    fn test_tags_from_interpreter_versioned() {
        let tags = tags_from_interpreter("python3.11.2");
        assert!(tags.contains("python"));
        assert!(tags.contains("python3"));

        let tags = tags_from_interpreter("php8.1");
        assert!(tags.contains("php"));
        assert!(tags.contains("php8"));
    }

    #[test]
    fn test_tags_from_interpreter_with_path() {
        let tags = tags_from_interpreter("/usr/bin/python3");
        assert!(tags.contains("python"));
        assert!(tags.contains("python3"));
    }

    #[test]
    fn test_tags_from_interpreter_unrecognized() {
        let tags = tags_from_interpreter("unknown-interpreter");
        assert!(tags.is_empty());

        let tags = tags_from_interpreter("");
        assert!(tags.is_empty());
    }

    // Test is_text function
    #[test]
    fn test_is_text_basic() {
        assert!(is_text(Cursor::new(b"hello world")).unwrap());
        assert!(is_text(Cursor::new(b"")).unwrap());
        assert!(!is_text(Cursor::new(b"hello\x00world")).unwrap());
    }

    #[test]
    fn test_is_text_unicode() {
        assert!(is_text(Cursor::new("éóñəå  ⊂(◉‿◉)つ(ノ≥∇≤)ノ".as_bytes())).unwrap());
        assert!(is_text(Cursor::new(r"¯\_(ツ)_/¯".as_bytes())).unwrap());
        assert!(is_text(Cursor::new("♪┏(・o･)┛♪┗ ( ･o･) ┓♪".as_bytes())).unwrap());
    }

    #[test]
    fn test_is_text_binary_data() {
        // ELF header
        assert!(!is_text(Cursor::new(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01])).unwrap());
        // Random binary data
        assert!(!is_text(Cursor::new(&[0x43, 0x92, 0xd9, 0x0f, 0xaf, 0x32, 0x2c])).unwrap());
    }

    // Test parse_shebang function
    #[test]
    fn test_parse_shebang_basic() {
        let tags = parse_shebang(Cursor::new(b"#!/usr/bin/python")).unwrap();
        assert!(tags.contains("python"));

        let tags = parse_shebang(Cursor::new(b"#!/usr/bin/env python")).unwrap();
        assert!(tags.contains("python"));
    }

    #[test]
    fn test_parse_shebang_env_with_flags() {
        let tags = parse_shebang(Cursor::new(b"#!/usr/bin/env -S python -u")).unwrap();
        assert!(tags.contains("python"));
    }

    #[test]
    fn test_parse_shebang_spaces() {
        let tags = parse_shebang(Cursor::new(b"#! /usr/bin/python")).unwrap();
        assert!(tags.contains("python"));

        let tags = parse_shebang(Cursor::new(b"#!/usr/bin/foo  python")).unwrap();
        // Should get first interpreter
        assert!(tags.is_empty()); // "foo" is not recognized
    }

    #[test]
    fn test_parse_shebang_no_shebang() {
        let tags = parse_shebang(Cursor::new(b"import sys")).unwrap();
        assert!(tags.is_empty());

        let tags = parse_shebang(Cursor::new(b"")).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_parse_shebang_invalid_utf8() {
        let result = parse_shebang(Cursor::new(&[0x23, 0x21, 0xf9, 0x93, 0x01, 0x42, 0xcd]));
        match result {
            Ok(tags) => assert!(tags.is_empty()),
            Err(_) => (), // I/O errors are acceptable for invalid UTF-8 data
        }
    }

    // File system tests using tempfiles
    #[test]
    fn test_tags_from_path_file_not_found() {
        let result = tags_from_path("/nonexistent/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_tags_from_path_regular_file() {
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, "print('hello')").unwrap();

        let tags = tags_from_path(file.path()).unwrap();
        assert!(tags.contains("file"));
        assert!(tags.contains("non-executable"));
        assert!(tags.contains("text"));
    }

    #[test]
    fn test_tags_from_path_executable_file() {
        let dir = tempdir().unwrap();
        let script_path = dir.path().join("script.py");
        fs::write(&script_path, "#!/usr/bin/env python3\nprint('hello')").unwrap();

        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        let tags = tags_from_path(&script_path).unwrap();
        assert!(tags.contains("file"));
        assert!(tags.contains("executable"));
        assert!(tags.contains("python"));
        assert!(tags.contains("text"));
    }

    #[test]
    fn test_tags_from_path_directory() {
        let dir = tempdir().unwrap();
        let tags = tags_from_path(dir.path()).unwrap();
        assert_eq!(tags, HashSet::from(["directory"]));
    }

    #[test]
    fn test_tags_from_path_binary_file() {
        let dir = tempdir().unwrap();
        let binary_path = dir.path().join("binary");
        fs::write(&binary_path, &[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01]).unwrap();

        let tags = tags_from_path(&binary_path).unwrap();
        assert!(tags.contains("file"));
        assert!(tags.contains("binary"));
        assert!(tags.contains("non-executable"));
    }

    #[test]
    fn test_file_is_text_simple() {
        let dir = tempdir().unwrap();
        let text_path = dir.path().join("text.txt");
        fs::write(&text_path, "Hello, world!").unwrap();
        assert!(file_is_text(&text_path).unwrap());
    }

    #[test]
    fn test_file_is_text_does_not_exist() {
        let result = file_is_text("/nonexistent/file");
        assert!(result.is_err());
    }

    // Test extensions that need binary check
    #[test]
    fn test_plist_binary_detection() {
        let dir = tempdir().unwrap();
        let plist_path = dir.path().join("test.plist");

        // Binary plist
        let binary_plist = [
            0x62, 0x70, 0x6c, 0x69, 0x73, 0x74, 0x30, 0x30, // "bplist00"
            0xd1, 0x01, 0x02, 0x5f, 0x10, 0x0f,
        ];
        fs::write(&plist_path, &binary_plist).unwrap();

        let tags = tags_from_path(&plist_path).unwrap();
        assert!(tags.contains("plist"));
        assert!(tags.contains("binary"));
    }

    #[test]
    fn test_plist_text_detection() {
        let dir = tempdir().unwrap();
        let plist_path = dir.path().join("test.plist");

        let text_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>TestKey</key>
    <string>TestValue</string>
</dict>
</plist>"#;
        fs::write(&plist_path, text_plist).unwrap();

        let tags = tags_from_path(&plist_path).unwrap();
        assert!(tags.contains("plist"));
        assert!(tags.contains("text"));
    }

    // Additional edge case tests
    #[test]
    fn test_empty_file() {
        let dir = tempdir().unwrap();
        let empty_path = dir.path().join("empty");
        fs::write(&empty_path, "").unwrap();

        let tags = tags_from_path(&empty_path).unwrap();
        assert!(tags.contains("file"));
        assert!(tags.contains("text")); // Empty files are considered text
        assert!(tags.contains("non-executable"));
    }

    #[test]
    fn test_shebang_incomplete() {
        let shebang_incomplete = parse_shebang(Cursor::new(b"#!   \n")).unwrap();
        assert!(shebang_incomplete.is_empty());
    }

    #[test]
    fn test_multiple_extensions() {
        let tags = tags_from_filename("backup.tar.gz");
        assert!(tags.contains("binary"));
        assert!(tags.contains("gzip"));
    }

    // Additional comprehensive tests from Python version
    #[test]
    fn test_comprehensive_shebang_parsing() {
        let test_cases = vec![
            ("", vec![]),
            ("#!/usr/bin/python", vec!["python"]),
            ("#!/usr/bin/env python", vec!["python"]),
            ("#! /usr/bin/python", vec!["python"]),
            ("#!/usr/bin/foo  python", vec![]), // "foo" not recognized
            ("#!/usr/bin/env -S python -u", vec!["python"]),
            ("#!/usr/bin/env", vec![]),
            ("#!/usr/bin/env -S", vec![]),
        ];

        for (input, expected) in test_cases {
            let tags = parse_shebang(Cursor::new(input.as_bytes())).unwrap();
            let expected_set: TagSet = expected.iter().cloned().collect();
            assert_eq!(tags, expected_set, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_invalid_utf8_shebang() {
        // Test that invalid UTF-8 in shebang doesn't crash
        let invalid_utf8_cases = vec![
            &[0xf9, 0x93, 0x01, 0x42, 0xcd][..],
            &[0x23, 0x21, 0xf9, 0x93, 0x01, 0x42, 0xcd][..],
            &[0x23, 0x21, 0x00, 0x00, 0x00, 0x00][..],
        ];

        for input in invalid_utf8_cases {
            // Should not panic, should return empty set for invalid UTF-8
            let result = parse_shebang(Cursor::new(input));
            match result {
                Ok(tags) => assert!(tags.is_empty()),
                Err(_) => (), // I/O errors are acceptable for invalid data
            }
        }
    }
}
