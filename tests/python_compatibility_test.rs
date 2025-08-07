use file_identify::{parse_shebang_from_file, tags_from_interpreter, parse_shebang, ShebangTuple};
use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;

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
fn test_parse_shebang_from_file_python_compatibility() {
    let test_cases = vec![
        ("#!/usr/bin/env python3", 0o755, shebang_tuple!["python3"]),
        ("#!/bin/bash", 0o755, shebang_tuple!["/bin/bash"]), 
        ("#!/usr/bin/env node", 0o755, shebang_tuple!["node"]),
        ("#!/bin/sh", 0o755, shebang_tuple!["/bin/sh"]),
        ("#!/usr/bin/python", 0o755, shebang_tuple!["/usr/bin/python"]),
        ("#!/usr/bin/env -S python -u", 0o755, shebang_tuple!["python", "-u"]),
        ("#!/usr/bin/env", 0o755, shebang_tuple!()),
        ("#!/usr/bin/env -S", 0o755, shebang_tuple!()),
        ("print('no shebang')", 0o755, shebang_tuple!()),  // No shebang but executable
        ("#!/usr/bin/env python3", 0o644, shebang_tuple!()), // Shebang but non-executable
    ];
    
    for (shebang, mode, expected) in test_cases {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", shebang).unwrap();
        writeln!(temp_file, "# test content").unwrap();
        
        let temp_path = temp_file.path();
        
        // Set permissions
        let mut perms = fs::metadata(temp_path).unwrap().permissions();
        perms.set_mode(mode);
        fs::set_permissions(temp_path, perms).unwrap();
        
        let result = parse_shebang_from_file(temp_path).unwrap();
        
        assert_eq!(
            result, expected,
            "Failed for shebang '{}' with mode {:o}",
            shebang, mode
        );
    }
}

#[test]
fn test_tags_from_interpreter_python_compatibility() {
    let test_cases = vec![
        ("python3", vec!["python", "python3"]),
        ("python", vec!["python"]),
        ("/usr/bin/python", vec!["python"]),
        ("/usr/bin/python3", vec!["python", "python3"]),
        ("python3.11", vec!["python", "python3"]),
        ("python3.11.2", vec!["python", "python3"]),
        ("bash", vec!["bash", "shell"]),
        ("/bin/bash", vec!["bash", "shell"]),
        ("sh", vec!["sh", "shell"]),
        ("/bin/sh", vec!["sh", "shell"]),
        ("node", vec!["javascript"]),
        ("nodejs", vec!["javascript"]),
        ("perl", vec!["perl"]),
        ("ruby", vec!["ruby"]),
        ("php", vec!["php"]),
        ("php7", vec!["php", "php7"]),
        ("php8", vec!["php", "php8"]),
        ("unknown-interpreter", vec![]),
        ("", vec![]),  // Edge case
    ];
    
    for (interpreter, expected_vec) in test_cases {
        let result = tags_from_interpreter(interpreter);
        let expected: HashSet<&str> = expected_vec.into_iter().collect();
        
        assert_eq!(
            result, expected,
            "Failed for interpreter '{}': expected {:?}, got {:?}",
            interpreter, expected, result
        );
    }
}

#[test]
fn test_parse_shebang_edge_cases_python_compatibility() {
    let malformed_cases = vec![
        "#! ",  // Shebang with just space
        "#!",   // Shebang with nothing
        "#!/usr/bin/env\t\t",  // Shebang with tabs
        "#!/usr/bin/env\n",    // Shebang with immediate newline
        "#!/usr/bin/\x00binary",  // Shebang with null byte (will fail UTF-8 check)
    ];
    
    for shebang in malformed_cases {
        let result = parse_shebang(Cursor::new(shebang.as_bytes())).unwrap();
        
        // All malformed shebangs should return empty vector
        assert!(
            result.is_empty(),
            "Malformed shebang '{}' should return empty vector, got {:?}",
            shebang.escape_debug(), result
        );
    }
}

#[test]
fn test_specific_python_behaviors() {
    // Test specific Python identify behaviors that must match exactly
    
    // Test env handling
    let env_cases = vec![
        ("#!/usr/bin/env python3", shebang_tuple!["python3"]),
        ("#!/usr/bin/env -S python3 -u", shebang_tuple!["python3", "-u"]),
        ("#!/usr/bin/env -S python3", shebang_tuple!["python3"]),
        ("#!/usr/bin/env", shebang_tuple!()),
        ("#!/usr/bin/env -S", shebang_tuple!()),
    ];
    
    for (shebang, expected) in env_cases {
        let result = parse_shebang(Cursor::new(shebang.as_bytes())).unwrap();
        assert_eq!(
            result, expected,
            "env handling failed for '{}': expected {:?}, got {:?}",
            shebang, expected, result
        );
    }
    
    // Test interpreter versioning
    let version_cases = vec![
        ("python3", vec!["python", "python3"]),
        ("python3.11", vec!["python", "python3"]),
        ("python3.11.2", vec!["python", "python3"]),
        ("python2", vec!["python", "python2"]),
        ("php7", vec!["php", "php7"]),
        ("php8", vec!["php", "php8"]),
    ];
    
    for (interpreter, expected_vec) in version_cases {
        let result = tags_from_interpreter(interpreter);
        let expected: HashSet<&str> = expected_vec.into_iter().collect();
        
        assert_eq!(
            result, expected,
            "Version handling failed for '{}': expected {:?}, got {:?}",
            interpreter, expected, result
        );
    }
    
    // Test path stripping
    let path_cases = vec![
        ("/usr/bin/python", vec!["python"]),
        ("/bin/bash", vec!["bash", "shell"]),
        ("/usr/local/bin/node", vec!["javascript"]), // node -> javascript
        ("ruby", vec!["ruby"]),
    ];
    
    for (interpreter, expected_vec) in path_cases {
        let result = tags_from_interpreter(interpreter);
        let expected: HashSet<&str> = expected_vec.into_iter().collect();
        
        assert_eq!(
            result, expected,
            "Path stripping failed for '{}': expected {:?}, got {:?}",
            interpreter, expected, result
        );
    }
}

#[test]
fn test_ascii_printable_requirement() {
    // Python requires only printable ASCII in shebang lines
    let non_printable_cases: Vec<&[u8]> = vec![
        b"#!/usr/bin/python\x01",  // Control character
        b"#!/usr/bin/python\x7f",  // DEL character
        b"#!/usr/bin/python\xff",  // Non-ASCII
        b"#!/usr/bin/python\x00",  // Null
    ];
    
    for shebang_bytes in non_printable_cases {
        let result = parse_shebang(Cursor::new(shebang_bytes)).unwrap();
        assert!(
            result.is_empty(),
            "Non-printable shebang should return empty: '{:?}', got {:?}",
            shebang_bytes, result
        );
    }
    
    // Valid printable ASCII should work
    let result = parse_shebang(Cursor::new(b"#!/usr/bin/python")).unwrap();
    assert_eq!(result, shebang_tuple!["/usr/bin/python"]);
}

use std::io::Write;

#[test]
fn test_comprehensive_real_world_cases() {
    // Real-world shebang patterns that must work identically to Python
    let real_world_cases = vec![
        ("#!/usr/bin/env python3", shebang_tuple!["python3"]),
        ("#!/usr/bin/python3", shebang_tuple!["/usr/bin/python3"]),
        ("#!/bin/bash", shebang_tuple!["/bin/bash"]),
        ("#!/bin/sh", shebang_tuple!["/bin/sh"]),
        ("#!/usr/bin/env node", shebang_tuple!["node"]),
        ("#!/usr/bin/env ruby", shebang_tuple!["ruby"]),
        ("#!/usr/bin/env perl", shebang_tuple!["perl"]),
        ("#!/usr/bin/env php", shebang_tuple!["php"]),
        ("#!/usr/bin/env -S python3 -u", shebang_tuple!["python3", "-u"]),
        ("#!/usr/bin/env -S python3 -O", shebang_tuple!["python3", "-O"]),
        ("#!/usr/bin/env -S node --experimental-modules", shebang_tuple!["node", "--experimental-modules"]),
    ];
    
    for (shebang, expected) in real_world_cases {
        let result = parse_shebang(Cursor::new(shebang.as_bytes())).unwrap();
        assert_eq!(
            result, expected,
            "Real-world case failed for '{}': expected {:?}, got {:?}",
            shebang, expected, result
        );
    }
}