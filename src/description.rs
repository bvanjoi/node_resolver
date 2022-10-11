use crate::map::{ExportsField, Field, ImportsField, PathTreeNode};
use crate::{AliasMap, RResult, Resolver, ResolverError};
use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SideEffects {
    Bool(bool),
    Array(Vec<String>),
}

#[derive(Debug)]
pub struct PkgJSON {
    pub name: Option<String>,
    pub version: Option<String>,
    pub alias_fields: IndexMap<String, AliasMap>,
    pub exports_field_tree: Option<PathTreeNode>,
    pub imports_field_tree: Option<PathTreeNode>,
    pub side_effects: Option<SideEffects>,
    pub raw: serde_json::Value,
}

#[derive(Debug)]
pub struct PkgInfo {
    pub json: Arc<PkgJSON>,
    /// The path to the directory where the description file located.
    /// It not a property in package.json.
    pub dir_path: PathBuf,
}

impl PkgJSON {
    pub(crate) fn parse(content: &str, file_path: &Path) -> RResult<Self> {
        let json: serde_json::Value =
            tracing::debug_span!("serde_json_from_str").in_scope(|| {
                serde_json::from_str(content).map_err(|error| {
                    ResolverError::UnexpectedJson((file_path.to_path_buf(), error))
                })
            })?;

        let mut alias_fields = IndexMap::new();

        if let Some(value) = json.get("browser") {
            if let Some(map) = value.as_object() {
                for (key, value) in map {
                    if let Some(b) = value.as_bool() {
                        assert!(!b);
                        alias_fields.insert(key.to_string(), AliasMap::Ignored);
                    } else if let Some(s) = value.as_str() {
                        alias_fields.insert(key.to_string(), AliasMap::Target(s.to_string()));
                    }
                }
            }
        }

        let exports_field_tree = if let Some(value) = json.get("exports") {
            let tree = ExportsField::build_field_path_tree(value)?;
            Some(tree)
        } else {
            None
        };

        let imports_field_tree = if let Some(value) = json.get("imports") {
            let tree = ImportsField::build_field_path_tree(value)?;
            Some(tree)
        } else {
            None
        };

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let side_effects: Option<SideEffects> =
            json.get("sideEffects").map_or(Ok(None), |value| {
                // TODO: should optimized
                if let Some(b) = value.as_bool() {
                    Ok(Some(SideEffects::Bool(b)))
                } else if let Some(vec) = value.as_array() {
                    let mut ans = vec![];
                    for value in vec {
                        if let Some(str) = value.as_str() {
                            ans.push(str.to_string());
                        } else {
                            return Err(ResolverError::UnexpectedValue(format!(
                                "sideEffects in {} had unexpected value {}",
                                file_path.display(),
                                value
                            )));
                        }
                    }
                    Ok(Some(SideEffects::Array(ans)))
                } else {
                    Err(ResolverError::UnexpectedValue(format!(
                        "sideEffects in {} had unexpected value {}",
                        file_path.display(),
                        value
                    )))
                }
            })?;

        let version = json
            .get("version")
            .and_then(|value| value.as_str())
            .map(|str| str.to_string());

        Ok(Self {
            name,
            version,
            alias_fields,
            exports_field_tree,
            imports_field_tree,
            side_effects,
            raw: json,
        })
    }
}

impl Resolver {
    pub fn load_side_effects(
        &self,
        path: &Path,
    ) -> RResult<Option<(PathBuf, Option<SideEffects>)>> {
        let entry = self.load_entry(path)?;
        let ans = if let Some(pkg_info) = &entry.pkg_info {
            Some((
                pkg_info.dir_path.join(&self.options.description_file),
                pkg_info.json.side_effects.clone(),
            ))
        } else {
            None
        };
        Ok(ans)
    }
}
