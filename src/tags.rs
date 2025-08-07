use std::collections::HashSet;

pub const DIRECTORY: &str = "directory";
pub const SYMLINK: &str = "symlink";
pub const SOCKET: &str = "socket";
pub const FILE: &str = "file";
pub const EXECUTABLE: &str = "executable";
pub const NON_EXECUTABLE: &str = "non-executable";
pub const TEXT: &str = "text";
pub const BINARY: &str = "binary";

pub type TagSet = HashSet<&'static str>;

lazy_static::lazy_static! {
    pub static ref TYPE_TAGS: TagSet = HashSet::from([DIRECTORY, FILE, SYMLINK, SOCKET]);
    pub static ref MODE_TAGS: TagSet = HashSet::from([EXECUTABLE, NON_EXECUTABLE]);
    pub static ref ENCODING_TAGS: TagSet = HashSet::from([BINARY, TEXT]);
}

pub fn is_type_tag(tag: &str) -> bool {
    TYPE_TAGS.contains(tag)
}

pub fn is_mode_tag(tag: &str) -> bool {
    MODE_TAGS.contains(tag)
}

pub fn is_encoding_tag(tag: &str) -> bool {
    ENCODING_TAGS.contains(tag)
}
