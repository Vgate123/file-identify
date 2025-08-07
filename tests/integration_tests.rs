use file_identify::{
    ShebangTuple, file_is_text, parse_shebang_from_file, tags_from_filename, tags_from_interpreter,
    tags_from_path,
};
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use tempfile::tempdir;

// Helper macro to create ShebangTuple from string slices for testing
macro_rules! shebang_tuple {
    () => {
        ShebangTuple::new()
    };
    ($($item:expr),+) => {
        ShebangTuple::from_vec(vec![$($item.to_string()),+])
    };
}

#[test]
fn test_comprehensive_file_scenarios() {
    let dir = tempdir().unwrap();

    // Test Python file
    let py_path = dir.path().join("test.py");
    fs::write(&py_path, "#!/usr/bin/env python3\nprint('hello')").unwrap();
    let mut perms = fs::metadata(&py_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&py_path, perms).unwrap();

    let tags = tags_from_path(&py_path).unwrap();
    assert!(tags.contains("file"));
    assert!(tags.contains("executable"));
    assert!(tags.contains("python"));
    assert!(tags.contains("text"));

    // Test non-executable Python file
    let py_ne_path = dir.path().join("module.py");
    fs::write(&py_ne_path, "def hello(): pass").unwrap();

    let tags = tags_from_path(&py_ne_path).unwrap();
    assert!(tags.contains("file"));
    assert!(tags.contains("non-executable"));
    assert!(tags.contains("python"));
    assert!(tags.contains("text"));
}

#[test]
fn test_special_filenames() {
    let test_cases = vec![
        ("Dockerfile", vec!["text", "dockerfile"]),
        ("Makefile", vec!["text", "makefile"]),
        ("Cargo.toml", vec!["text", "toml", "cargo"]),
        ("package.json", vec!["text", "json"]),
        (".gitignore", vec!["text", "gitignore"]),
        ("README.md", vec!["text", "markdown", "plain-text"]),
    ];

    for (filename, expected_tags) in test_cases {
        let tags = tags_from_filename(filename);
        for expected in expected_tags {
            assert!(
                tags.contains(expected),
                "File '{}' should contain tag '{}', got: {:?}",
                filename,
                expected,
                tags
            );
        }
    }
}

#[test]
fn test_binary_vs_text_detection() {
    let dir = tempdir().unwrap();

    // Create text file
    let text_path = dir.path().join("text.txt");
    fs::write(&text_path, "Hello, world! 🌍").unwrap();
    assert!(file_is_text(&text_path).unwrap());

    // Create binary file (ELF header)
    let binary_path = dir.path().join("binary");
    fs::write(&binary_path, &[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01]).unwrap();
    assert!(!file_is_text(&binary_path).unwrap());
}

#[test]
fn test_interpreter_parsing_edge_cases() {
    let test_cases = vec![
        ("python", vec!["python"]),
        ("python3", vec!["python", "python3"]),
        ("python3.11.2", vec!["python", "python3"]),
        ("/usr/bin/python3.11", vec!["python", "python3"]),
        ("node", vec!["javascript"]),
        ("bash", vec!["shell", "bash"]),
        ("unknown-interpreter", vec![]),
        ("", vec![]),
    ];

    for (interpreter, expected) in test_cases {
        let tags = tags_from_interpreter(interpreter);
        for exp_tag in expected {
            assert!(
                tags.contains(exp_tag),
                "Interpreter '{}' should contain '{}', got: {:?}",
                interpreter,
                exp_tag,
                tags
            );
        }
    }
}

#[test]
fn test_extension_case_sensitivity() {
    // Extensions should be case-insensitive
    let tags_lower = tags_from_filename("image.jpg");
    let tags_upper = tags_from_filename("image.JPG");
    let tags_mixed = tags_from_filename("image.JpG");

    assert_eq!(tags_lower, tags_upper);
    assert_eq!(tags_lower, tags_mixed);
    assert!(tags_lower.contains("jpeg"));
    assert!(tags_lower.contains("image"));
    assert!(tags_lower.contains("binary"));
}

#[test]
fn test_socket_identification() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_socket");

    // Create a Unix socket
    let _listener = UnixListener::bind(&socket_path).unwrap();

    let tags = tags_from_path(&socket_path).unwrap();
    assert_eq!(tags, HashSet::from(["socket"]));
}

#[test]
fn test_symlink_identification() {
    let dir = tempdir().unwrap();
    let target_path = dir.path().join("target_file");
    let symlink_path = dir.path().join("symlink");

    fs::write(&target_path, "target content").unwrap();
    std::os::unix::fs::symlink(&target_path, &symlink_path).unwrap();

    let tags = tags_from_path(&symlink_path).unwrap();
    assert_eq!(tags, HashSet::from(["symlink"]));
}

#[test]
fn test_broken_symlink_identification() {
    let dir = tempdir().unwrap();
    let nonexistent_target = dir.path().join("nonexistent");
    let symlink_path = dir.path().join("broken_symlink");

    std::os::unix::fs::symlink(&nonexistent_target, &symlink_path).unwrap();

    let tags = tags_from_path(&symlink_path).unwrap();
    assert_eq!(tags, HashSet::from(["symlink"]));
}

#[test]
fn test_filename_precedence_over_extension() {
    // Special filenames should take precedence over extension matching
    let tags = tags_from_filename("setup.cfg");
    assert!(tags.contains("ini")); // From NAMES mapping
    assert!(tags.contains("text"));

    let tags = tags_from_filename("random.cfg");
    assert!(tags.contains("text")); // Just from extension
    assert!(!tags.contains("ini")); // No special name mapping
}

#[test]
fn test_shebang_with_arguments() {
    let dir = tempdir().unwrap();

    // Test shebang with -S flag
    let script_path = dir.path().join("script");
    fs::write(&script_path, "#!/usr/bin/env -S python3 -u\nprint('hello')").unwrap();
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

    let components = parse_shebang_from_file(&script_path).unwrap();
    assert_eq!(components, shebang_tuple!["python3", "-u"]);

    // Test that the first component converts to expected tags
    let tags = tags_from_interpreter(&components[0]);
    assert!(tags.contains("python"));
    assert!(tags.contains("python3"));
}

#[test]
fn test_non_executable_shebang_ignored() {
    let dir = tempdir().unwrap();
    let script_path = dir.path().join("script");
    fs::write(&script_path, "#!/usr/bin/env python3\nprint('hello')").unwrap();
    // Don't make it executable

    let components = parse_shebang_from_file(&script_path).unwrap();
    assert!(components.is_empty()); // Should be empty for non-executable files
}

#[test]
fn test_complex_filename_patterns() {
    let test_cases = vec![
        ("docker-compose.yml", vec!["text", "yaml"]),
        ("requirements.txt", vec!["text", "plain-text"]),
        (".pre-commit-config.yaml", vec!["text", "yaml"]),
        ("meson.build", vec!["text", "meson"]),
        ("BUILD.bazel", vec!["text", "bazel"]),
    ];

    for (filename, expected) in test_cases {
        let tags = tags_from_filename(filename);
        for exp_tag in expected {
            assert!(
                tags.contains(exp_tag),
                "File '{}' should contain '{}', got: {:?}",
                filename,
                exp_tag,
                tags
            );
        }
    }
}
