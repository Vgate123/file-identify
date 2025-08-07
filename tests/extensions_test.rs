use file_identify::extensions::{EXTENSIONS, NAMES, EXTENSIONS_NEED_BINARY_CHECK};
use std::collections::HashSet;

#[test]
fn test_extensions_have_binary_or_text() {
    for (extension, tags) in EXTENSIONS.iter() {
        let text_binary_tags: HashSet<&str> = ["text", "binary"].iter().cloned().collect();
        let intersection: HashSet<_> = tags.intersection(&text_binary_tags).collect();
        assert_eq!(intersection.len(), 1, 
            "Extension '{}' should have exactly one of 'text' or 'binary', got: {:?}", 
            extension, tags);
    }
}

#[test]
fn test_names_have_binary_or_text() {
    for (name, tags) in NAMES.iter() {
        let text_binary_tags: HashSet<&str> = ["text", "binary"].iter().cloned().collect();
        let intersection: HashSet<_> = tags.intersection(&text_binary_tags).collect();
        assert_eq!(intersection.len(), 1, 
            "Name '{}' should have exactly one of 'text' or 'binary', got: {:?}", 
            name, tags);
    }
}

#[test]
fn test_need_binary_check_do_not_specify_text_binary() {
    for (extension, tags) in EXTENSIONS_NEED_BINARY_CHECK.iter() {
        let text_binary_tags: HashSet<&str> = ["text", "binary"].iter().cloned().collect();
        let intersection: HashSet<_> = tags.intersection(&text_binary_tags).collect();
        assert_eq!(intersection.len(), 0, 
            "Extension '{}' in EXTENSIONS_NEED_BINARY_CHECK should not specify 'text' or 'binary', got: {:?}", 
            extension, tags);
    }
}

#[test]
fn test_mutually_exclusive_check_types() {
    let extensions_keys: HashSet<_> = EXTENSIONS.keys().collect();
    let need_binary_keys: HashSet<_> = EXTENSIONS_NEED_BINARY_CHECK.keys().collect();
    
    let intersection: HashSet<_> = extensions_keys.intersection(&need_binary_keys).collect();
    assert!(intersection.is_empty(), 
        "EXTENSIONS and EXTENSIONS_NEED_BINARY_CHECK should be mutually exclusive, found overlap: {:?}", 
        intersection);
}