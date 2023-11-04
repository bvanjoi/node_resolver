//! # `nodejs_resolver`
//!
//! ## How to use?
//!
//! ```rust
//! // |-- node_modules
//! // |---- foo
//! // |------ index.js
//! // | src
//! // |-- foo.ts
//! // |-- foo.js
//! // | tests
//!
//! use nodejs_resolver::Resolver;
//!
//! let cwd = std::env::current_dir().unwrap();
//! let resolver = Resolver::new(Default::default());
//!
//! resolver.resolve(&cwd.join("./src"), "foo");
//! // -> ResolveResult::Info(ResolveInfo {
//! //    path: PathBuf::from("<cwd>/node_modules/foo/index.js")
//! //    request: Request {
//! //       target: "",
//! //       fragment: "",
//! //       query: ""
//! //    }
//! //  })
//! //
//!
//! resolver.resolve(&cwd.join("./src"), "./foo");
//! // -> ResolveResult::Info(ResolveInfo {
//! //    path: PathBuf::from("<cwd>/src/foo.js")
//! //    request: Request {
//! //       target: "",
//! //       fragment: "",
//! //       query: ""
//! //    }
//! //  })
//! //
//! ```
//!

mod cache;
mod context;
mod description;
mod entry;
mod error;
mod fs;
mod info;
mod kind;
mod log;
mod map;
mod options;
mod parse;
mod plugin;
mod resolve;
mod resource;
mod state;
mod tsconfig;
mod tsconfig_path;

pub use cache::Cache;
use context::Context;
pub use description::DescriptionData;
pub use error::Error;
use info::Info;
use kind::PathKind;
use log::{color, depth};
use options::EnforceExtension::{Auto, Disabled, Enabled};
pub use options::{AliasMap, EnforceExtension, Options};
use plugin::{
    AliasPlugin, BrowserFieldPlugin, ImportsFieldPlugin, ParsePlugin, Plugin, PreferRelativePlugin,
    SymlinkPlugin,
};
pub use resource::Resource;
use state::State;

#[derive(Debug)]
pub struct Resolver {
    pub options: Options,
    pub(crate) cache: std::sync::Arc<Cache>,
}

#[derive(Debug, Clone)]
pub enum ResolveResult<T: Clone> {
    Resource(T),
    Ignored,
}

pub type RResult<T> = Result<T, Error>;

impl Resolver {
    #[must_use]
    pub fn new(options: Options) -> Self {
        log::enable_by_env();

        let cache = if let Some(external_cache) = options.external_cache.as_ref() {
            external_cache.clone()
        } else {
            std::sync::Arc::new(Cache::default())
        };

        let enforce_extension = match options.enforce_extension {
            Auto => {
                if options.extensions.iter().any(|ext| ext.is_empty()) {
                    Enabled
                } else {
                    Disabled
                }
            }
            _ => options.enforce_extension,
        };

        let tsconfig = match options.tsconfig {
            Some(config) => {
                // if is relative path, then resolve it to absolute path
                if config.is_absolute() {
                    Some(config)
                } else {
                    let cwd = std::env::current_dir().unwrap();
                    // concat cwd and config, but remove ./ prefix
                    Some(cwd.join(config.strip_prefix("./").unwrap_or(&config)))
                }
            }
            None => None,
        };

        let options = Options {
            enforce_extension,
            tsconfig,
            ..options
        };
        Self { options, cache }
    }

    pub fn resolve(
        &self,
        path: &std::path::Path,
        request: &str,
    ) -> RResult<ResolveResult<Resource>> {
        tracing::debug!(
            "{:-^30}\nTry to resolve '{}' in '{}'",
            color::green(&"[RESOLVER]"),
            color::cyan(&request),
            color::cyan(&path.display().to_string())
        );
        // let start = std::time::Instant::now();
        let parsed = Self::parse(request);
        let info = Info::new(path, parsed);
        let mut context = Context::new(
            self.options.fully_specified,
            self.options.resolve_to_context,
        );
        let result = if let Some(tsconfig_location) = self.options.tsconfig.as_ref() {
            self._resolve_with_tsconfig(info, tsconfig_location, &mut context)
        } else {
            self._resolve(info, &mut context)
        };

        let result = result.map_failed(|info| {
            type FallbackPlugin<'a> = AliasPlugin<'a>;
            FallbackPlugin::new(&self.options.fallback).apply(self, info, &mut context)
        });
        let result =
            result.map_success(|info| SymlinkPlugin::default().apply(self, info, &mut context));

        // let duration = start.elapsed().as_millis();
        // println!("time cost: {:?} us", duration); // us
        // if duration > 10 {
        //     println!(
        //         "{:?}ms, path: {:?}, request: {:?}",
        //         duration,
        //         path.display(),
        //         request,
        //     );
        // }

        match result {
            State::Success(ResolveResult::Ignored) => Ok(ResolveResult::Ignored),
            State::Success(ResolveResult::Resource(info)) => {
                let resource = Resource::new(info, self);
                Ok(ResolveResult::Resource(resource))
            }
            State::Error(err) => Err(err),
            State::Resolving(_) | State::Failed(_) => Err(Error::ResolveFailedTag),
        }
    }

    fn _resolve(&self, info: Info, context: &mut Context) -> State {
        tracing::debug!(
            "Resolving '{request}' in '{path}'",
            request = color::cyan(&info.request().target()),
            path = color::cyan(&info.normalized_path().as_ref().display())
        );

        context.depth.increase();
        if context.depth.cmp(127).is_ge() {
            return State::Error(Error::Overflow);
        }

        let state = ParsePlugin::default()
            .apply(self, info, context)
            .then(|info| AliasPlugin::new(&self.options.alias).apply(self, info, context))
            .then(|info| PreferRelativePlugin::default().apply(self, info, context))
            .then(|info| {
                let request = info.to_resolved_path();
                let entry = self.load_entry(&request);
                let pkg_info = match entry.pkg_info(self) {
                    Ok(pkg_info) => pkg_info,
                    Err(error) => return State::Error(error),
                };
                if let Some(pkg_info) = pkg_info {
                    ImportsFieldPlugin::new(pkg_info)
                        .apply(self, info, context)
                        .then(|info| {
                            BrowserFieldPlugin::new(pkg_info, false).apply(self, info, context)
                        })
                } else {
                    State::Resolving(info)
                }
            })
            .then(|info| {
                if matches!(
                    info.request().kind(),
                    PathKind::AbsolutePosix | PathKind::AbsoluteWin | PathKind::Relative
                ) {
                    self.resolve_as_context(info, context)
                        .then(|info| self.resolve_as_fully_specified(info, context))
                        .then(|info| self.resolve_as_file(info, context))
                        .then(|info| self.resolve_as_dir(info, context))
                } else {
                    self.resolve_as_modules(info, context)
                }
            });

        context.depth.decrease();
        state
    }
}

#[cfg(debug_assertions)]
pub mod test_helper {
    #[must_use]
    pub fn p(paths: Vec<&str>) -> std::path::PathBuf {
        paths.iter().fold(
            std::env::current_dir()
                .unwrap()
                .join("tests")
                .join("fixtures"),
            |acc, path| acc.join(path),
        )
    }

    #[must_use]
    pub fn vec_to_set(vec: Vec<&str>) -> std::collections::HashSet<String> {
        std::collections::HashSet::from_iter(vec.into_iter().map(|s| s.to_string()))
    }
}
