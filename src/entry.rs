use once_cell::sync::OnceCell;
use std::{
    borrow::Cow,
    fs::FileType,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use crate::{description::PkgInfo, normalize::NormalizePath, Error, RResult, Resolver};

#[derive(Debug, Default, Clone, Copy)]
pub struct EntryStat {
    /// `None` for non-existing file
    file_type: Option<FileType>,

    /// `None` for existing file but without system time.
    modified: Option<SystemTime>,
}

impl EntryStat {
    fn new(file_type: Option<FileType>, modified: Option<SystemTime>) -> Self {
        Self {
            file_type,
            modified,
        }
    }

    /// Returns `None` for non-existing file
    pub fn file_type(&self) -> Option<FileType> {
        self.file_type
    }

    /// Returns `None` for existing file but without system time.
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    fn stat(path: &Path) -> Self {
        if let Ok(meta) = path.metadata() {
            // This field might not be available on all platforms,
            // and will return an Err on platforms where it is not available.
            let modified = meta.modified().ok();
            Self::new(Some(meta.file_type()), modified)
        } else {
            Self::new(None, None)
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    parent: Option<Arc<Entry>>,
    path: Box<Path>,
    // None: package.json does not exist
    pkg_info: OnceCell<Option<Arc<PkgInfo>>>,
    stat: OnceCell<EntryStat>,
    // None: `self.path` is not a symlink
    symlink: OnceCell<Option<Arc<Path>>>,
}

impl Entry {
    fn new<P: AsRef<Path>>(parent: Option<Arc<Entry>>, path: P) -> Self {
        Self {
            parent,
            path: path.as_ref().into(),
            pkg_info: OnceCell::default(),
            stat: OnceCell::default(),
            symlink: OnceCell::default(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn parent(&self) -> Option<&Arc<Entry>> {
        self.parent.as_ref()
    }

    pub fn pkg_info(&self, resolver: &Resolver) -> RResult<&Option<Arc<PkgInfo>>> {
        self.pkg_info.get_or_try_init(|| {
            let pkg_name = &resolver.options.description_file;
            let path = self.path();
            let is_pkg_suffix = path.ends_with(pkg_name);
            if self.is_dir() || is_pkg_suffix {
                let pkg_path = if is_pkg_suffix {
                    Cow::Borrowed(path)
                } else {
                    Cow::Owned(path.join(pkg_name))
                };
                match resolver
                    .cache
                    .fs
                    .read_description_file(&pkg_path, EntryStat::default())
                {
                    Ok(info) => {
                        return Ok(Some(info));
                    }
                    Err(error @ (Error::UnexpectedJson(_) | Error::UnexpectedValue(_))) => {
                        // Return bad json
                        return Err(error);
                    }
                    Err(Error::Io(_)) => {
                        // package.json not found
                    }
                    _ => unreachable!(),
                };
            }
            if let Some(parent) = &self.parent() {
                return parent.pkg_info(resolver).cloned();
            }
            Ok(None)
        })
    }

    pub fn is_file(&self) -> bool {
        self.cached_stat()
            .file_type()
            .map_or(false, |ft| ft.is_file())
    }

    pub fn is_dir(&self) -> bool {
        self.cached_stat()
            .file_type()
            .map_or(false, |ft| ft.is_dir())
    }

    pub fn exists(&self) -> bool {
        self.cached_stat().file_type().is_some()
    }

    pub fn cached_stat(&self) -> EntryStat {
        *self.stat.get_or_init(|| EntryStat::stat(&self.path))
    }

    /// Returns the canonicalized path of `self.path` if it is a symlink.
    /// Returns None if `self.path` is not a symlink.
    pub fn symlink(&self) -> &Option<Arc<Path>> {
        self.symlink.get_or_init(|| {
            if self.path.read_link().is_err() {
                return None;
            }
            match dunce::canonicalize(&self.path) {
                Ok(symlink_path) => Some(Arc::from(symlink_path)),
                Err(_) => None,
            }
        })
    }
}

impl Resolver {
    pub(super) fn load_entry(&self, path: &Path) -> Arc<Entry> {
        let key = path.normalize();
        if let Some(entry) = self.cache.entries.get(key.as_ref()) {
            return entry.clone();
        }
        let parent = path.parent().map(|parent| self.load_entry(parent));
        self.cache
            .entries
            .entry(key.to_path_buf().into_boxed_path())
            .or_insert_with(|| Arc::new(Entry::new(parent, key.as_ref())))
            .value()
            .clone()
    }

    // TODO: should put entries as a parament.
    pub fn clear_entries(&self) {
        self.cache.entries.clear();
    }

    #[must_use]
    pub fn get_dependency_from_entry(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        todo!("get_dependency_from_entry")
    }
}

#[test]
#[ignore]
fn dependency_test() {
    let case_path = super::test_helper::p(vec!["full", "a"]);
    let request = "package2";
    let resolver = Resolver::new(Default::default());
    resolver.resolve(&case_path, request).unwrap();
    let (file, missing) = resolver.get_dependency_from_entry();
    assert_eq!(file.len(), 3);
    assert_eq!(missing.len(), 1);
}
