use crate::Resolver;

pub(crate) enum PathKind {
    Empty,
    Relative,
    AbsoluteWin,
    AbsolutePosix,
    Internal,
    BuildInModule,
    Normal,
}

use daachorse::{DoubleArrayAhoCorasick, DoubleArrayAhoCorasickBuilder, MatchKind};
use once_cell::sync::Lazy;
use phf::{phf_set, Set};

const BUILT_IN_MODULE_SET: Set<&'static str> = phf_set! {
   "_http_agent",
   "_http_client",
   "_http_common",
   "_http_incoming",
   "_http_outgoing",
   "_http_server",
   "_stream_duplex",
   "_stream_passthrough",
   "_stream_readable",
   "_stream_transform",
   "_stream_wrap",
   "_stream_writable",
   "_tls_common",
   "_tls_wrap",
   "assert",
   "assert/,strict",
   "async_hooks",
   "buffer",
   "child_process",
   "cluster",
   "console",
   "constants",
   "crypto",
   "dgram",
   "diagnostics_channel",
   "dns",
   "dns/promises",
   "domain",
   "events",
   "fs",
   "fs/promises",
   "http",
   "http2",
   "https",
   "inspector",
   "module",
   "net",
   "os",
   "path",
   "path/posix",
   "path/win32",
   "perf_hooks",
   "process",
   "punycode",
   "querystring",
   "readline",
   "repl",
   "stream",
   "stream/consumers",
   "stream/promises",
   "stream/web",
   "string_decoder",
   "sys",
   "timers",
   "timers/promises",
   "tls",
   "trace_events",
   "tty",
   "url",
   "util",
   "util/types",
   "v8",
   "vm",
   "wasi",
   "worker_threads",
   "zlib",
};
static ABSOLUTE_WIN_PATTERN_LENGTH_TWO: [&str; 52] = [
    "a:", "b:", "c:", "d:", "e:", "f:", "g:", "h:", "i:", "j:", "k:", "l:", "m:", "n:", "o:", "p:",
    "q:", "r:", "s:", "t:", "u:", "v:", "w:", "x:", "y:", "z:", "A:", "B:", "C:", "D:", "E:", "F:",
    "G:", "H:", "I:", "J:", "K:", "L:", "M:", "N:", "O:", "P:", "Q:", "R:", "S:", "T:", "U:", "V:",
    "W:", "X:", "Y:", "Z:",
];
static ABSOLUTE_WIN_PATTERN_REST: [&str; 104] = [
    "a:\\", "b:\\", "c:\\", "d:\\", "e:\\", "f:\\", "g:\\", "h:\\", "i:\\", "j:\\", "k:\\", "l:\\",
    "m:\\", "n:\\", "o:\\", "p:\\", "q:\\", "r:\\", "s:\\", "t:\\", "u:\\", "v:\\", "w:\\", "x:\\",
    "y:\\", "z:\\", "A:\\", "B:\\", "C:\\", "D:\\", "E:\\", "F:\\", "G:\\", "H:\\", "I:\\", "J:\\",
    "K:\\", "L:\\", "M:\\", "N:\\", "O:\\", "P:\\", "Q:\\", "R:\\", "S:\\", "T:\\", "U:\\", "V:\\",
    "W:\\", "X:\\", "Y:\\", "Z:\\", "a:/", "b:/", "c:/", "d:/", "e:/", "f:/", "g:/", "h:/", "i:/",
    "j:/", "k:/", "l:/", "m:/", "n:/", "o:/", "p:/", "q:/", "r:/", "s:/", "t:/", "u:/", "v:/",
    "w:/", "x:/", "y:/", "z:/", "A:/", "B:/", "C:/", "D:/", "E:/", "F:/", "G:/", "H:/", "I:/",
    "J:/", "K:/", "L:/", "M:/", "N:/", "O:/", "P:/", "Q:/", "R:/", "S:/", "T:/", "U:/", "V:/",
    "W:/", "X:/", "Y:/", "Z:/",
];
static PMA: Lazy<DoubleArrayAhoCorasick> = Lazy::new(|| {
    DoubleArrayAhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&ABSOLUTE_WIN_PATTERN_REST)
        .unwrap()
});

impl Resolver {
    pub(crate) fn get_target_kind(target: &str) -> PathKind {
        if target.is_empty() {
            PathKind::Empty
        } else if Self::is_build_in_module(target) {
            PathKind::BuildInModule
        } else if target.starts_with('#') {
            PathKind::Internal
        } else if target.starts_with('/') {
            PathKind::AbsolutePosix
        } else if target == "."
            || target.starts_with("./")
            || target.starts_with("../")
            || target == ".."
        {
            PathKind::Relative
        } else {
            if target.len() == 2 && ABSOLUTE_WIN_PATTERN_LENGTH_TWO.contains(&target) {
                return PathKind::AbsoluteWin;
            }
            let mut it = PMA.leftmost_find_iter(target);
            if let Some(mat) = it.next() {
                let match_pattern_len = ABSOLUTE_WIN_PATTERN_REST[mat.value()].len();
                if mat.start() == 0 && mat.end() - mat.start() == match_pattern_len {
                    return PathKind::AbsoluteWin;
                }
            }
            PathKind::Normal
        }
    }

    pub fn is_build_in_module(target: &str) -> bool {
        BUILT_IN_MODULE_SET.contains(target)
    }
}

#[test]
fn test_resolver() {
    assert!(Resolver::is_build_in_module("fs"));
    assert!(!Resolver::is_build_in_module("a"));
    assert!(matches!(Resolver::get_target_kind(""), PathKind::Empty));
    assert!(matches!(
        Resolver::get_target_kind("D:"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("C:path"),
        PathKind::Normal
    ));
    assert!(matches!(
        Resolver::get_target_kind("C:\\a"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("c:/a"),
        PathKind::AbsoluteWin
    ));
    assert!(matches!(
        Resolver::get_target_kind("cc:/a"),
        PathKind::Normal
    ));
}
