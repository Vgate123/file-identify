use std::collections::{HashMap, HashSet};
use crate::tags::TagSet;

lazy_static::lazy_static! {
    pub static ref INTERPRETERS: HashMap<&'static str, TagSet> = {
        let mut map = HashMap::new();
        
        map.insert("ash", HashSet::from(["shell", "ash"]));
        map.insert("awk", HashSet::from(["awk"]));
        map.insert("bash", HashSet::from(["shell", "bash"]));
        map.insert("bats", HashSet::from(["shell", "bash", "bats"]));
        map.insert("cbsd", HashSet::from(["shell", "cbsd"]));
        map.insert("csh", HashSet::from(["shell", "csh"]));
        map.insert("dash", HashSet::from(["shell", "dash"]));
        map.insert("expect", HashSet::from(["expect"]));
        map.insert("ksh", HashSet::from(["shell", "ksh"]));
        map.insert("node", HashSet::from(["javascript"]));
        map.insert("nodejs", HashSet::from(["javascript"]));
        map.insert("perl", HashSet::from(["perl"]));
        map.insert("php", HashSet::from(["php"]));
        map.insert("php7", HashSet::from(["php", "php7"]));
        map.insert("php8", HashSet::from(["php", "php8"]));
        map.insert("python", HashSet::from(["python"]));
        map.insert("python2", HashSet::from(["python", "python2"]));
        map.insert("python3", HashSet::from(["python", "python3"]));
        map.insert("ruby", HashSet::from(["ruby"]));
        map.insert("sh", HashSet::from(["shell", "sh"]));
        map.insert("tcsh", HashSet::from(["shell", "tcsh"]));
        map.insert("zsh", HashSet::from(["shell", "zsh"]));

        map
    };
}