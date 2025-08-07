use file_identify::{tags_from_filename, tags_from_interpreter, tags_from_path};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tempfile::{NamedTempFile, tempdir};

struct CompatibilityTester {
    python_script_path: String,
}

impl CompatibilityTester {
    fn new() -> Self {
        let python_script = r#"
import sys
import json
sys.path.insert(0, '../')
from identify import identify

def get_tags_from_path(path):
    try:
        return sorted(list(identify.tags_from_path(path)))
    except ValueError:
        return []

def get_tags_from_filename(filename):
    return sorted(list(identify.tags_from_filename(filename)))

def get_tags_from_interpreter(interpreter):
    return sorted(list(identify.tags_from_interpreter(interpreter)))

if __name__ == "__main__":
    action = sys.argv[1]
    if action == "path":
        print(json.dumps(get_tags_from_path(sys.argv[2])))
    elif action == "filename":
        print(json.dumps(get_tags_from_filename(sys.argv[2])))
    elif action == "interpreter":
        print(json.dumps(get_tags_from_interpreter(sys.argv[2])))
"#;

        let script_path = "/tmp/python_identify_helper.py";
        fs::write(script_path, python_script).expect("Failed to write Python helper script");

        Self {
            python_script_path: script_path.to_string(),
        }
    }

    fn get_python_tags_from_path(&self, path: &str) -> HashSet<String> {
        let output = Command::new("python3")
            .arg(&self.python_script_path)
            .arg("path")
            .arg(path)
            .output()
            .expect("Failed to run Python script");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tags: Vec<String> = serde_json::from_str(&stdout).unwrap_or_default();
            tags.into_iter().collect()
        } else {
            HashSet::new()
        }
    }

    fn get_python_tags_from_filename(&self, filename: &str) -> HashSet<String> {
        let output = Command::new("python3")
            .arg(&self.python_script_path)
            .arg("filename")
            .arg(filename)
            .output()
            .expect("Failed to run Python script");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tags: Vec<String> = serde_json::from_str(&stdout).unwrap_or_default();
            tags.into_iter().collect()
        } else {
            HashSet::new()
        }
    }

    fn get_python_tags_from_interpreter(&self, interpreter: &str) -> HashSet<String> {
        let output = Command::new("python3")
            .arg(&self.python_script_path)
            .arg("interpreter")
            .arg(interpreter)
            .output()
            .expect("Failed to run Python script");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tags: Vec<String> = serde_json::from_str(&stdout).unwrap_or_default();
            tags.into_iter().collect()
        } else {
            HashSet::new()
        }
    }
}

#[test]
fn test_tags_from_path_compatibility() {
    let tester = CompatibilityTester::new();
    let fixtures_dir = Path::new("../compatibility_test_fixtures");

    let test_files = [
        "test_files.py",
        "test_files.sh",
        "test_files.js",
        "test_files.json",
        "Dockerfile",
        "Makefile",
    ];

    for test_file in &test_files {
        let test_path = fixtures_dir.join(test_file);
        if test_path.exists() {
            let python_tags = tester.get_python_tags_from_path(test_path.to_str().unwrap());
            let rust_tags: HashSet<String> = match tags_from_path(&test_path) {
                Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
                Err(_) => HashSet::new(),
            };

            assert_eq!(
                python_tags, rust_tags,
                "Tags mismatch for {}: Python={:?}, Rust={:?}",
                test_file, python_tags, rust_tags
            );
        }
    }
}

#[test]
fn test_tags_from_filename_compatibility() {
    let tester = CompatibilityTester::new();

    let test_files = [
        "test_files.py",
        "test_files.sh",
        "test_files.js",
        "test_files.json",
        "Dockerfile",
        "Makefile",
        "setup.py",
        "package.json",
        "Cargo.toml",
        ".gitignore",
        "README.md",
    ];

    for test_file in &test_files {
        let python_tags = tester.get_python_tags_from_filename(test_file);
        let rust_tags: HashSet<String> = tags_from_filename(test_file)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(
            python_tags, rust_tags,
            "Filename tags mismatch for {}: Python={:?}, Rust={:?}",
            test_file, python_tags, rust_tags
        );
    }
}

#[test]
fn test_tags_from_interpreter_compatibility() {
    let tester = CompatibilityTester::new();

    let interpreters = [
        "python3", "python", "bash", "sh", "node", "perl", "ruby", "php",
    ];

    for interpreter in &interpreters {
        let python_tags = tester.get_python_tags_from_interpreter(interpreter);
        let rust_tags: HashSet<String> = tags_from_interpreter(interpreter)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(
            python_tags, rust_tags,
            "Interpreter tags mismatch for {}: Python={:?}, Rust={:?}",
            interpreter, python_tags, rust_tags
        );
    }
}

#[test]
fn test_nonexistent_file_compatibility() {
    let tester = CompatibilityTester::new();
    let nonexistent = "/path/that/does/not/exist";

    let python_tags = tester.get_python_tags_from_path(nonexistent);
    let rust_tags: HashSet<String> = match tags_from_path(nonexistent) {
        Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
        Err(_) => HashSet::new(),
    };

    assert_eq!(python_tags, rust_tags);
    assert!(python_tags.is_empty() && rust_tags.is_empty());
}

#[test]
fn test_directory_compatibility() {
    let tester = CompatibilityTester::new();
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path();

    let python_tags = tester.get_python_tags_from_path(dir_path.to_str().unwrap());
    let rust_tags: HashSet<String> = match tags_from_path(dir_path) {
        Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
        Err(_) => HashSet::new(),
    };

    assert_eq!(
        python_tags, rust_tags,
        "Directory tags mismatch: Python={:?}, Rust={:?}",
        python_tags, rust_tags
    );
}

#[test]
fn test_symlink_compatibility() {
    let tester = CompatibilityTester::new();
    let temp_dir = tempdir().unwrap();

    let target_file = temp_dir.path().join("target.txt");
    fs::write(&target_file, "hello").unwrap();

    let symlink_path = temp_dir.path().join("link.txt");
    std::os::unix::fs::symlink(&target_file, &symlink_path).unwrap();

    let python_tags = tester.get_python_tags_from_path(symlink_path.to_str().unwrap());
    let rust_tags: HashSet<String> = match tags_from_path(&symlink_path) {
        Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
        Err(_) => HashSet::new(),
    };

    assert_eq!(
        python_tags, rust_tags,
        "Symlink tags mismatch: Python={:?}, Rust={:?}",
        python_tags, rust_tags
    );
}

#[test]
fn test_binary_file_compatibility() {
    let tester = CompatibilityTester::new();
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file
        .write_all(b"\x00\x01\x02\x03binary content")
        .unwrap();
    let binary_path = temp_file.path();

    let python_tags = tester.get_python_tags_from_path(binary_path.to_str().unwrap());
    let rust_tags: HashSet<String> = match tags_from_path(binary_path) {
        Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
        Err(_) => HashSet::new(),
    };

    assert_eq!(
        python_tags, rust_tags,
        "Binary file tags mismatch: Python={:?}, Rust={:?}",
        python_tags, rust_tags
    );
}

#[test]
fn test_executable_script_compatibility() {
    let tester = CompatibilityTester::new();
    let temp_dir = tempdir().unwrap();

    let script_content = "#!/usr/bin/env python3\nprint('hello world')";
    let script_path = temp_dir.path().join("script.py");
    fs::write(&script_path, script_content).unwrap();

    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

    let python_tags = tester.get_python_tags_from_path(script_path.to_str().unwrap());
    let rust_tags: HashSet<String> = match tags_from_path(&script_path) {
        Ok(tags) => tags.into_iter().map(|s| s.to_string()).collect(),
        Err(_) => HashSet::new(),
    };

    assert_eq!(
        python_tags, rust_tags,
        "Executable script tags mismatch: Python={:?}, Rust={:?}",
        python_tags, rust_tags
    );
}
