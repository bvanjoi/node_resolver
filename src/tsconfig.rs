// copy from https://github.com/drivasperez/tsconfig

use std::{path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use crate::context::Context;
use crate::{Error, Info, RResult, ResolveResult, Resolver, State};

#[derive(Debug, Clone, Default)]
pub struct TsConfig {
    pub extends: Option<String>,
    pub compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub base_url: Option<String>,
    pub paths: Option<FxHashMap<String, Vec<String>>>,
}

impl TsConfig {
    pub fn parse(json_str: &str, location: &Path) -> RResult<serde_json::Value> {
        let serde_value = jsonc_parser::parse_to_serde_value(json_str, &Default::default())
            .map_err(|err| {
                Error::UnexpectedValue(format!("Parse {} failed. Error: {err}", location.display()))
            })?
            .unwrap_or_else(|| panic!("Transfer {} to serde value failed", location.display()));
        Ok(serde_value)
    }
}

impl Resolver {
    pub(super) async fn parse_ts_file(
        &self,
        location: &Path,
        context: &mut Context,
    ) -> RResult<TsConfig> {
        let json = self.parse_file_to_value(location, context).await?;
        let compiler_options = json.get("compilerOptions").map(|options| {
            // TODO: should optimized
            let base_url = options.get("baseUrl").map(|v| v.as_str().unwrap().to_string());
            let paths = options.get("paths").map(|v| {
                let mut map = FxHashMap::default();
                // TODO: should optimized
                for (key, obj) in v.as_object().unwrap() {
                    map.insert(
                        key.to_string(),
                        obj.as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap().to_string())
                            .collect(),
                    );
                }
                map
            });
            CompilerOptions { base_url, paths }
        });
        let extends: Option<String> = json.get("extends").map(|v| v.to_string());
        Ok(TsConfig { extends, compiler_options })
    }

    #[async_recursion::async_recursion]
    async fn parse_file_to_value(
        &self,
        location: &Path,
        context: &mut Context,
    ) -> RResult<serde_json::Value> {
        let entry = self.load_entry(location);
        if !self.is_file(&entry).await {
            // Its role is to ensure that `stat` exists
            return Err(Error::CantFindTsConfig(entry.path().into()));
        }

        let value =
            self.cache.fs.read_tsconfig(self, location, self.cached_stat(&entry).await).await?;
        let mut json = Arc::as_ref(&value).clone();

        // merge `extends`.
        if let serde_json::Value::String(s) = &json["extends"] {
            // `location` pointed to `dir/tsconfig.json`
            let dir = location.parent().unwrap().to_path_buf();
            let request = Self::parse(s);
            let prev_resolve_to_context = context.resolve_to_context.get();
            if prev_resolve_to_context {
                context.resolve_to_context.set(false);
            }
            let state = self._resolve(Info::new(dir, request), context).await;
            if prev_resolve_to_context {
                context.resolve_to_context.set(true);
            }
            // Is it better to use cache?
            if let State::Success(result) = state {
                let extends_tsconfig_json = match result {
                    ResolveResult::Resource(info) => {
                        self.parse_file_to_value(&info.to_resolved_path(), context).await
                    }
                    ResolveResult::Ignored => {
                        return Err(Error::UnexpectedValue(format!(
                            "{s} had been ignored in {}",
                            location.display()
                        )));
                    }
                }?;
                merge(&mut json, extends_tsconfig_json);
            }
        }
        Ok(json)
    }
}

fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (&mut serde_json::Value::Object(ref mut a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => {
            if let serde_json::Value::Null = a {
                *a = b;
            }
        }
    }
}
