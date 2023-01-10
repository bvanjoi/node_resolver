use nodejs_resolver::{
    test_helper::{p, vec_to_set},
    AliasKind, AliasMap, Cache, EnforceExtension, Error, Options, ResolveResult, Resolver,
    SideEffects,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;

fn should_equal(resolver: &Resolver, path: &Path, request: &str, expected: PathBuf) {
    match resolver.resolve(path, request) {
        Ok(ResolveResult::Info(info)) => {
            assert_eq!(info.join(), expected);
        }
        Ok(ResolveResult::Ignored) => panic!("should not ignored"),
        Err(error) => panic!("{:?}", error),
    }
}

fn should_ignored(resolver: &Resolver, path: &Path, request: &str) {
    match resolver.resolve(path, request) {
        Ok(ResolveResult::Ignored) => {}
        _ => unreachable!(),
    }
}

fn should_resolve_failed(resolver: &Resolver, path: &Path, request: &str) {
    let result = resolver.resolve(path, request);
    if !matches!(result, Err(Error::ResolveFailedTag)) {
        println!("{:?}", result);
        unreachable!();
    }
}

fn should_overflow(resolver: &Resolver, path: &Path, request: &str) {
    let result = resolver.resolve(path, request);
    if !matches!(result, Err(Error::Overflow)) {
        println!("{:?}", result);
        unreachable!();
    }
}

fn should_unexpected_json_error(
    resolver: &Resolver,
    path: &Path,
    request: &str,
    error_file_path: PathBuf,
) {
    match resolver.resolve(path, request) {
        Err(err) => match err {
            Error::UnexpectedJson((actual_error_file_path, _)) => {
                assert_eq!(error_file_path, actual_error_file_path)
            }
            _ => {
                println!("{:?}", err);
                unreachable!();
            }
        },
        Ok(result) => {
            println!("{:?}", result);
            unreachable!();
        }
    }
}

fn should_unexpected_value_error(
    resolver: &Resolver,
    path: &Path,
    request: &str,
    expected_err_msg: String,
) {
    match resolver.resolve(path, request) {
        Err(err) => match err {
            Error::UnexpectedValue(err) => {
                if err.contains(&expected_err_msg) {
                } else {
                    assert_eq!(err, expected_err_msg);
                }
            }
            _ => {
                println!("{:?}", err);
                unreachable!();
            }
        },
        Ok(result) => {
            println!("{:?}", result);
            unreachable!();
        }
    }
}

#[test]
fn extensions_test() {
    let extensions_cases_path = p(vec!["extensions"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".ts"), String::from(".js")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions", "a.js"]),
    );
    should_resolve_failed(&resolver, &extensions_cases_path, "./a.js/");

    should_equal(
        &resolver,
        &extensions_cases_path.join("./a"),
        "",
        p(vec!["extensions", "a.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions", "a.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./dir",
        p(vec!["extensions", "dir", "index.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index"),
        ".",
        p(vec!["extensions", "index.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index.js"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_resolve_failed(&resolver, &extensions_cases_path.join("index."), ".");
    should_resolve_failed(&resolver, &extensions_cases_path.join("inde"), ".");

    should_equal(
        &resolver,
        &extensions_cases_path,
        "m",
        p(vec!["extensions", "node_modules", "m.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "m/",
        p(vec!["extensions", "node_modules", "m", "index.ts"]),
    );
    should_resolve_failed(&resolver, &extensions_cases_path, "./b.js");
    should_resolve_failed(&resolver, &extensions_cases_path, "fs");
    should_resolve_failed(&resolver, &extensions_cases_path, "./a.js/");
    should_resolve_failed(&resolver, &extensions_cases_path, "m.js/");
    let resolver = Resolver::new(Options {
        extensions: vec![String::from("ts"), String::from(".js")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &extensions_cases_path.join("./a"),
        "",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./dir",
        p(vec!["extensions", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index.js"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index."),
        ".",
        p(vec!["extensions", "index.ts"]),
    );
    should_resolve_failed(&resolver, &extensions_cases_path.join("inde"), ".");

    should_equal(
        &resolver,
        &extensions_cases_path,
        "m",
        p(vec!["extensions", "node_modules", "m.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "m/",
        p(vec!["extensions", "node_modules", "m", "index.js"]),
    );
    should_resolve_failed(&resolver, &extensions_cases_path, "./b.js");
    should_resolve_failed(&resolver, &extensions_cases_path, "fs");
    should_resolve_failed(&resolver, &extensions_cases_path, "./a.js/");
    should_resolve_failed(&resolver, &extensions_cases_path, "m.js/");

    let extensions_cases_path = p(vec!["extensions2"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js"), String::from(""), String::from(".ts")], // `extensions` can start with `.` or not.
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./index",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./b",
        p(vec!["extensions2", "b"]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js"), String::from(""), String::from(".ts")], // `extensions` can start with `.` or not.
        enforce_extension: EnforceExtension::Disabled,
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions2", "a"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./index",
        p(vec!["extensions2", "index"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./b",
        p(vec!["extensions2", "b"]),
    );
}

#[test]
fn alias_test() {
    let alias_cases_path = p(vec!["alias"]);
    let resolver = Resolver::new(Options {
        alias: AliasMap::from_iter(vec![
            (
                String::from("aliasA"),
                AliasKind::Target(String::from("./a")),
            ),
            // (
            //     String::from("./b$"),
            //     AliasKind::Target(String::from("./a/index")),
            // ), // TODO: should we use trailing?
            (
                String::from("fs"),
                AliasKind::Target(
                    alias_cases_path
                        .join("node_modules")
                        .join("browser")
                        .join("index.js")
                        .to_string_lossy()
                        .to_string(),
                ),
            ),
            (String::from("./e"), AliasKind::Target(String::from("./d"))),
            (String::from("./d"), AliasKind::Target(String::from("./e"))),
            (String::from("./f"), AliasKind::Target(String::from("./g"))),
            (String::from("./g"), AliasKind::Target(String::from("./h"))),
            (
                String::from("recursive"),
                AliasKind::Target(String::from("./recursive/dir")),
            ),
            (
                String::from("#"),
                AliasKind::Target(String::from("./c/dir")),
            ),
            (
                String::from("@"),
                AliasKind::Target(String::from("./c/dir")),
            ),
            (
                String::from("@start"),
                AliasKind::Target(p(vec!["alias"]).display().to_string()),
            ),
            (
                String::from("@recursive/pointed"),
                AliasKind::Target(String::from("@recursive/general/index.js")),
            ),
            (
                String::from("@recursive/general"),
                AliasKind::Target(String::from("@recursive/general/redirect.js")),
            ),
            (
                String::from("@recursive"),
                AliasKind::Target(String::from("@recursive/general")),
            ),
            (String::from("./c"), AliasKind::Target(String::from("./c"))),
            (String::from("ignore"), AliasKind::Ignored),
        ]),
        ..Default::default()
    });
    should_resolve_failed(&resolver, &alias_cases_path, "ignored/a");
    should_ignored(&resolver, &alias_cases_path, "ignore/a");
    should_equal(
        &resolver,
        &alias_cases_path,
        "ignore-a",
        p(vec!["alias", "node_modules", "ignore-a", "index.js"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path.join("node_modules").join("@recursive"),
        "fs",
        p(vec!["alias", "node_modules", "browser", "index.js"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "fs",
        p(vec!["alias", "node_modules", "browser", "index.js"]),
    );
    should_overflow(&resolver, &alias_cases_path, "./e");
    should_equal(
        &resolver,
        &alias_cases_path,
        "./f",
        p(vec!["alias", "h", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/index",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/general",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "redirect.js",
        ]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/pointed",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &p(vec!["in_exist_path"]),
        "@start/a",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/index",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/dir",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/dir/index",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/index",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/dir",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/dir/index",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "#",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "#/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_resolve_failed(&resolver, &alias_cases_path, "@/a.js");
    should_equal(
        &resolver,
        &alias_cases_path,
        "recursive",
        p(vec!["alias", "recursive", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "recursive/index",
        p(vec!["alias", "recursive", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c",
        p(vec!["alias", "c", "index"]),
    );
    should_equal(
        &resolver,
        &p(vec!["in_exist_path"]),
        "@start/a",
        p(vec!["alias", "a", "index"]),
    );
    should_resolve_failed(&resolver, &Path::new("@start/a"), "");
    // TODO: exact alias
    // should_equal(resolver, &alias_cases_path, "./b?aa#bb?cc", fixture!("alias/a/index?aa#bb?cc"));
    // should_equal(resolver, &alias_cases_path, "./b/?aa#bb?cc", fixture!("alias/a/index?aa#bb?cc"));
    // should_equal(resolver, &alias_cases_path, "./b", fixture!("alias/a/index"));
    // should_equal(resolver, &alias_cases_path, "./b/", fixture!("alias/a/index"));
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/index",
        p(vec!["alias", "b", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/dir",
        p(vec!["alias", "b", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/dir/index",
        p(vec!["alias", "b", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/index",
        p(vec!["alias", "c", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/dir",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/dir/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_ignored(&resolver, &alias_cases_path, "ignore");
    // test alias ordered
    let resolver = Resolver::new(Options {
        alias: AliasMap::from_iter(vec![
            (
                String::from("@A/index"),
                AliasKind::Target(String::from("./a")),
            ),
            (String::from("@A"), AliasKind::Target(String::from("./b"))),
        ]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &alias_cases_path,
        "@A/index",
        p(vec!["alias", "a", "index"]),
    );
    let resolver = Resolver::new(Options {
        alias: AliasMap::from_iter(vec![
            (String::from("@A"), AliasKind::Target(String::from("./b"))),
            (
                String::from("@A/index"),
                AliasKind::Target(String::from("./a")),
            ),
        ]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &alias_cases_path,
        "@A/index",
        p(vec!["alias", "b", "index"]),
    );
}

#[test]
fn symlink_test() {
    let symlink_cases_path = p(vec!["symlink"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./this/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./node.relative.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./node.relative.sym.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./this/this/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/this/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/this/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/linked/that/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path,
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this"),
        "./outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/lib"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/outer/linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/outer/linked"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that"),
        "./outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/lib"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/outer/linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/outer/linked"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    let linked_path = symlink_cases_path.join("linked");
    let resolver = Resolver::new(Options {
        symlinks: false,
        ..Default::default()
    });

    should_equal(
        &resolver,
        &linked_path,
        "./index.js",
        p(vec!["symlink", "linked", "index.js"]),
    );
    should_equal(
        &resolver,
        &linked_path,
        "./this/this/index.js",
        p(vec!["symlink", "linked", "this", "this", "index.js"]),
    );
}

#[test]
fn simple_test() {
    let simple_case_path = p(vec!["simple"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &p(vec!["in-exist-path"]),
        &p(vec!["simple", "lib", "index"]).display().to_string(),
        p(vec!["simple", "lib", "index.js"]),
    );
    // directly
    should_equal(
        &resolver,
        &simple_case_path,
        "./lib/index",
        p(vec!["simple", "lib", "index.js"]),
    );
    // as directory
    should_equal(
        &resolver,
        &simple_case_path,
        ".",
        p(vec!["simple", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &simple_case_path,
        "",
        p(vec!["simple", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &simple_case_path.join(".."),
        "./simple",
        p(vec!["simple", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &simple_case_path.join(".."),
        "./simple/lib/index",
        p(vec!["simple", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &p(vec!["in-exist-path"]),
        &p(vec!["simple", "lib", "index"]).display().to_string(),
        p(vec!["simple", "lib", "index.js"]),
    );
}

#[test]
fn pnpm_structure_test() {
    let case_path = p(vec!["pnpm-structure", "node_modules"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-aa",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-aa",
            "index.js",
        ]),
    );
    should_resolve_failed(&resolver, &case_path.join("exports-field-c"), "./b");
    should_equal(
        &resolver,
        &case_path.join("exports-field-c").join("lib"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-c").join("lib"),
        "./b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-c",
            "lib",
            "b.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-a",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-a",
            "lib",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a"),
        "./lib",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-a",
            "lib",
            "index.js",
        ]),
    );
    should_unexpected_value_error(
        &resolver,
        &case_path.join("exports-field-a"),
        "exports-field-b",
        "Package path exports-field-b is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &p(vec!["pnpm-structure", "module-a"]),
        "module-b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "module-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("module-a"),
        "module-b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "module-b",
            "index.js",
        ]),
    )
}

#[test]
fn resolve_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );

    should_equal(
        &resolver,
        &fixture_path,
        p(vec!["main1.js"]).to_str().unwrap(),
        p(vec!["main1.js"]),
    );
    should_equal(&resolver, &fixture_path, "./main1.js", p(vec!["main1.js"]));
    should_equal(&resolver, &fixture_path, "./main1", p(vec!["main1.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js?query",
        p(vec!["main1.js?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js#fragment",
        p(vec!["main1.js#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js#fragment?query",
        p(vec!["main1.js#fragment?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js?#fragment",
        p(vec!["main1.js?#fragment"]),
    );
    should_equal(&resolver, &fixture_path, "./a.js", p(vec!["a.js"]));
    should_equal(&resolver, &fixture_path, "./a", p(vec!["a.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a?query",
        p(vec!["node_modules", "m1", "a.js?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a#fragment",
        p(vec!["node_modules", "m1", "a.js#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a#fragment?query",
        p(vec!["node_modules", "m1", "a.js#fragment?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a?#fragment",
        p(vec!["node_modules", "m1", "a.js?#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./dirOrFile",
        p(vec!["dirOrFile.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./dirOrFile/",
        p(vec!["dirOrFile", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "complexm/step1",
        p(vec!["node_modules", "complexm", "step1.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m2/b.js",
        p(vec!["node_modules", "m2", "b.js"]),
    );
    // edge case
    // should_equal(&resolver, "./no#fragment/#/#", fixture!("no\0#fragment/\0#.\0#.js"));
    should_equal(
        &resolver,
        &fixture_path,
        "./no#fragment/#/",
        p(vec!["no.js#fragment", "#"]),
    );
    should_resolve_failed(&resolver, &p(vec!["no#fragment"]), "#/#");
    should_resolve_failed(&resolver, &p(vec!["no#fragment", "#"]), "#");
    should_resolve_failed(&resolver, &p(vec!["no#fragment", "#"]), "#.js");
    should_equal(
        &resolver,
        &p(vec!["no#fragment", "#"]),
        "../../a",
        p(vec!["a.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment", "#"]),
        "./#",
        p(vec!["no#fragment", "#", "#.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment"]),
        "./#/#",
        p(vec!["no#fragment", "#", "#.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment"]),
        "./#/#.js",
        p(vec!["no#fragment", "#", "#.js"]),
    );
    let web_modules_path = fixture_path.join("node_modules/complexm/web_modules/m1");
    should_equal(
        &resolver,
        &web_modules_path,
        "m2/b.js",
        p(vec!["node_modules", "m2", "b.js"]),
    );
    let multiple_modules_path = fixture_path.join("multiple_modules");
    should_equal(
        &resolver,
        &multiple_modules_path,
        "m1/a.js",
        p(vec!["multiple_modules", "node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &multiple_modules_path,
        "m1/b.js",
        p(vec!["node_modules", "m1", "b.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path.join("browser-module/node_modules"),
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );
}

#[test]
fn browser_filed_test() {
    let browser_module_case_path = p(vec!["browser-module"]);

    let resolver = Resolver::new(Default::default());
    should_equal(
        &resolver,
        &browser_module_case_path,
        "browser-string",
        p(vec![
            "browser-module",
            "node_modules",
            "browser-string",
            "index.js",
        ]),
    );

    let resolver = Resolver::new(Options {
        browser_field: true,
        ..Default::default()
    });

    should_ignored(&resolver, &p(vec![]), "./browser-after-main");
    should_ignored(&resolver, &browser_module_case_path, ".");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore.js");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect.js");
    should_equal(
        &resolver,
        &browser_module_case_path,
        "browser-string",
        p(vec![
            "browser-module",
            "node_modules",
            "browser-string",
            "target.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "recursive-module",
        p(vec![
            "browser-module",
            "node_modules",
            "recursive-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/replaced",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/replaced.js",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-a",
        p(vec!["browser-module", "browser", "module-a.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-b",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-d",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./toString",
        p(vec!["browser-module", "lib", "toString.js"]),
    );
    should_resolve_failed(&resolver, &browser_module_case_path, "toString");
    should_resolve_failed(&resolver, &browser_module_case_path, "./toString.js");
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect",
        p(vec!["browser-module", "lib", "sub.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect3",
        p(vec![
            "browser-module",
            "lib",
            "redirect3-target",
            "dir",
            "index.js",
        ]),
    );

    // browser with alias
    let resolver = Resolver::new(Options {
        browser_field: true,
        alias: AliasMap::from_iter(vec![(
            String::from("./lib/toString.js"),
            AliasKind::Target(String::from("module-d")),
        )]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &browser_module_case_path,
        "./toString",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );

    let lib_path = browser_module_case_path.join("lib");
    should_ignored(&resolver, &lib_path, "./ignore");
    should_ignored(&resolver, &lib_path, "./ignore.js");
    should_equal(
        &resolver,
        &lib_path,
        "./toString",
        p(vec!["browser-module", "lib", "toString.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./toString.js",
        p(vec!["browser-module", "lib", "toString.js"]),
    );

    should_equal(
        &resolver,
        &lib_path,
        "./replaced",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./replaced.js",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-a",
        p(vec!["browser-module", "browser", "module-a.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-b",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-d",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect",
        p(vec!["browser-module", "lib", "sub.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect3",
        p(vec![
            "browser-module",
            "lib",
            "redirect3-target",
            "dir",
            "index.js",
        ]),
    );

    let browser_after_main_path = p(vec!["browser-after-main"]);
    should_ignored(&resolver, &browser_after_main_path, ".");
    should_ignored(&resolver, &p(vec![]), "./browser-after-main");
    should_ignored(&resolver, &browser_after_main_path, ".");

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = p(vec!["dependencies"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".json"), String::from(".js")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "some-module/index",
        p(vec![
            "dependencies",
            "a",
            "b",
            "node_modules",
            "some-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "module/file",
        p(vec![
            "dependencies",
            "a",
            "node_modules",
            "module",
            "file.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "other-module/file.js",
        p(vec![
            "dependencies",
            "node_modules",
            "other-module",
            "file.js",
        ]),
    );

    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "some-module/index",
        p(vec![
            "dependencies",
            "a",
            "b",
            "node_modules",
            "some-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "module/file",
        p(vec![
            "dependencies",
            "a",
            "node_modules",
            "module",
            "file.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "other-module/file.js",
        p(vec![
            "dependencies",
            "node_modules",
            "other-module",
            "file.js",
        ]),
    );
    // TODO: how passing on context?
    // TODO: Maybe it should use (`getPath`)[https://github.com/webpack/enhanced-resolve/blob/main/lib/getPaths.js]
}

#[test]
fn full_specified_test() {
    // TODO: should I need add `fullSpecified` flag?
    let full_cases_path = p(vec!["full", "a"]);
    let resolver = Resolver::new(Options {
        alias: AliasMap::from_iter(vec![
            (
                String::from("alias1"),
                AliasKind::Target(String::from("./abc")),
            ),
            (
                String::from("alias2"),
                AliasKind::Target(String::from("./")),
            ),
        ]),
        browser_field: true,
        ..Default::default()
    });
    should_resolve_failed(&resolver, &full_cases_path.join(".."), ".");
    should_equal(
        &resolver,
        &full_cases_path,
        "package4/a.js",
        p(vec!["full", "a", "node_modules", "package4", "b.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./abc.js",
        p(vec!["full", "a", "abc.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1/file.js",
        p(vec!["full", "a", "node_modules", "package1", "file.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1",
        p(vec!["full", "a", "node_modules", "package1", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package2",
        p(vec!["full", "a", "node_modules", "package2", "a.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias1",
        p(vec!["full", "a", "abc.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias2",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package3",
        p(vec![
            "full",
            "a",
            "node_modules",
            "package3",
            "dir",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package3/dir",
        p(vec![
            "full",
            "a",
            "node_modules",
            "package3",
            "dir",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package4/a.js",
        p(vec!["full", "a", "node_modules", "package4", "b.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        ".",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir",
        p(vec!["full", "a", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir/",
        p(vec!["full", "a", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir?123#456",
        p(vec!["full", "a", "dir", "index.js?123#456"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir/?123#456",
        p(vec!["full", "a", "dir", "index.js?123#456"]),
    );
}

#[test]
fn missing_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    // TODO: optimize error
    // TODO: path
    should_resolve_failed(&resolver, &fixture_path, "./missing-file");
    should_resolve_failed(&resolver, &fixture_path, "./missing-file.js");
    should_resolve_failed(&resolver, &fixture_path, "missing-module");
    should_resolve_failed(&resolver, &fixture_path, "missing-module/missing-file");
    should_resolve_failed(&resolver, &fixture_path, "m1/missing-file");
    should_resolve_failed(&resolver, &fixture_path, "m1/");
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );
}

#[test]
fn incorrect_package_test() {
    let incorrect_package_path = p(vec!["incorrect-package"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_unexpected_json_error(
        &resolver,
        &incorrect_package_path.join("pack1"),
        ".",
        incorrect_package_path.join("pack1").join("package.json"),
    );
    should_unexpected_json_error(
        &resolver,
        &incorrect_package_path.join("pack2"),
        ".",
        incorrect_package_path.join("pack2").join("package.json"),
    );
}

#[test]
fn scoped_packages_test() {
    let scoped_path = p(vec!["scoped"]);
    let resolver = Resolver::new(Options {
        browser_field: true,
        ..Default::default()
    });
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack1",
        p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack1/main",
        p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2",
        p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2/main",
        p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2/lib",
        p(vec![
            "scoped",
            "node_modules",
            "@scope",
            "pack2",
            "lib",
            "index.js",
        ]),
    );
}

#[test]
fn exports_fields_test() {
    // TODO: [`exports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.js#L2280) flag
    let export_cases_path = p(vec!["exports-field"]);
    let resolver = Resolver::new(Options {
        condition_names: vec_to_set(vec!["import"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require/a",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "a",
            "index.js",
        ]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["require"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require/a",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "cjs",
            "a",
            "index.js",
        ]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });
    should_resolve_failed(&resolver, &export_cases_path, "@exports-field/coreaaaa");
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/x.js",
        "Package path exports-field/x.js is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/a.js",
        "Package path exports-field/dist/a.js is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/../../../a.js",
        "Package path exports-field/dist/../../../a.js is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/package.json",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "package.json",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/dist/main",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@exports-field/core",
        p(vec!["exports-field", "a.js"]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./b",
        p(vec!["exports-field", "b.js"]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./a",
        p(vec!["exports-field", "a.js"]),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "@exports-field/core/a",
        "Package path @exports-field/core/a is not exported".to_string(),
    );
    // `exports` only used in `Normal` target.
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/lib/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "main.js",
        ]),
    );
    should_resolve_failed(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/dist/main",
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/anything/else",
        "Package path exports-field/anything/else is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/",
        "Only requesting file allowed".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist",
        "Package path exports-field/dist is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/lib",
        "Package path exports-field/lib is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "invalid-exports-field",
        "Export field key can't mixed relative path and conditional object".to_string(),
    );
    // `exports` filed take precedence over `main`
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "x.js",
        ]),
    );
    let export_cases_path2 = p(vec!["exports-field2"]);
    // TODO: maybe we need provide `full_specified` flag.
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js?foo",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js?foo",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/main",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    // TODO: error stack
    // TODO: should `exports-field?foo is not exported`.
    should_unexpected_value_error(
        &resolver,
        &export_cases_path2,
        "exports-field?foo",
        "Package path exports-field is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path2,
        "exports-field#foo",
        "Package path exports-field is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js#foo",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js#foo",
        ]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/lib/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        condition_names: vec_to_set(vec!["node"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &p(vec!["exports-field3"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1", "index.js"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1"]),
        "m1",
        p(vec![
            "exports-field3",
            "pkg1",
            "node_modules",
            "m1",
            "m1.js",
        ]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });
    should_unexpected_value_error(
        &resolver,
        &p(vec!["exports-field-error"]),
        "exports-field",
        "Package path exports-field is not exported".to_string(),
    );
}

#[test]
fn imports_fields_test() {
    // TODO: ['imports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.js#L1228)
    let import_cases_path = p(vec!["imports-field"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &import_cases_path,
        "#c-redirect/index",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#imports-field",
        p(vec!["imports-field", "b.js"]),
    );
    should_unexpected_value_error(
        &resolver,
        &import_cases_path,
        "#b",
        "Package path ../b.js can't imported in".to_string(),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#a/dist/main.js",
        p(vec![
            "imports-field",
            "node_modules",
            "a",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#ccc/index.js",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_unexpected_value_error(
        &resolver,
        &import_cases_path,
        "#a",
        "Package path #a can't imported in".to_string(),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path.join("dir"),
        "#imports-field",
        p(vec!["imports-field", "b.js"]),
    );
}

#[test]
fn prefer_relative_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
        prefer_relative: true,
        ..Default::default()
    });
    should_equal(&resolver, &fixture_path, "main1.js", p(vec!["main1.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );
}

#[test]
fn cache_fs() {
    use std::fs::write;
    use std::thread::sleep;
    use std::time::Duration;

    let fixture_path = p(vec!["cache-fs"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "index.js"]),
    );

    write(
        &fixture_path.join("package.json"),
        "{\"main\": \"./src/module.js\"}",
    )
    .expect("write failed");

    resolver.clear_entries();
    sleep(Duration::from_secs(1));

    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "module.js"]),
    );

    write(
        &fixture_path.join("package.json"),
        "{\"main\": \"./src/index.js\"}",
    )
    .expect("write failed");

    resolver.clear_entries();
    sleep(Duration::from_secs(1));

    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "index.js"]),
    );
}

#[test]
fn main_fields_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-self",
        p(vec!["main-field-self", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-self2",
        p(vec!["main-field-self2", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field",
        p(vec!["main-field", "src", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-filed-no-relative",
        p(vec!["main-filed-no-relative", "c.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path.join("main-filed-no-relative"),
        ".",
        p(vec!["main-filed-no-relative", "c.js"]),
    );

    let resolver = Resolver::new(Options {
        main_fields: vec![String::from("module")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-filed-no-relative",
        p(vec!["main-filed-no-relative", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path.join("main-filed-no-relative"),
        ".",
        p(vec!["main-filed-no-relative", "index.js"]),
    );

    let resolver = Resolver::new(Options {
        main_fields: vec![String::from("main"), String::from("module")],

        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );

    let resolver = Resolver::new(Options {
        main_fields: vec![String::from("module"), String::from("main")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );
}

#[test]
fn tsconfig_paths_test() {
    let tsconfig_paths = p(vec!["tsconfig-paths"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_paths.join("tsconfig.json")),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &tsconfig_paths,
        "test2/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );

    should_equal(
        &resolver,
        &tsconfig_paths,
        "test0",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test1/foo",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test2/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_resolve_failed(&resolver, &tsconfig_paths, "te*t3/foo");
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test3/foo",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test4/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test5/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );

    // normal
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test0-success",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test1-success",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test2-success/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test3-success",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test4-first/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test5-second/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./actual/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_nested() {
    let tsconfig_paths = p(vec!["tsconfig-paths-nested"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_paths.join("tsconfig.json")),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &tsconfig_paths,
        "test0",
        p(vec!["tsconfig-paths-nested", "nested", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test1/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test2/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test2-success",
            "foo.ts",
        ]),
    );
    should_resolve_failed(&resolver, &tsconfig_paths, "te*t3/foo");
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test3/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test4/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test4-first",
            "foo.ts",
        ]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test5/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test5-second",
            "foo.ts",
        ]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_without_base_url_test() {
    let case_path = p(vec!["tsconfig-paths-without-baseURL"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_resolve_failed(&resolver, &case_path, "should-not-be-imported")
}

#[test]
fn tsconfig_paths_overridden_base_url() {
    let case_path = p(vec!["tsconfig-paths-override-baseURL"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "#/test",
        p(vec!["tsconfig-paths-override-baseURL", "src", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_missing_base_url() {
    let case_path = p(vec!["tsconfig-paths-missing-baseURL"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_resolve_failed(&resolver, &case_path, "#/test");
}

#[test]
fn tsconfig_paths_extends_from_node_modules() {
    let case_path = p(vec!["tsconfig-paths-extends-from-module"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "foo",
        p(vec!["tsconfig-paths-extends-from-module", "src", "test.ts"]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.scope.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "foo",
        p(vec!["tsconfig-paths-extends-from-module", "src", "test.ts"]),
    );
}

#[test]
fn tsconfig_inexist() {
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(p(vec![])),
        ..Default::default()
    });
    assert!(matches!(
        resolver.resolve(&p(vec![]), "./a.js"),
        Err(Error::CantFindTsConfig)
    ))
}

#[test]
fn load_side_effects_test() {
    let case_path = p(vec!["exports-field"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    let scope_import_require_path = if let ResolveResult::Info(info) = resolver
        .resolve(&case_path, "@scope/import-require")
        .unwrap()
    {
        info.path
    } else {
        panic!("error")
    };

    assert_eq!(
        resolver
            .load_side_effects(&scope_import_require_path)
            .unwrap()
            .unwrap()
            .0,
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "package.json"
        ])
    );

    assert!(matches!(
        resolver
            .load_side_effects(&scope_import_require_path)
            .unwrap()
            .unwrap()
            .1,
        Some(SideEffects::Array(_))
    ));

    let exports_field_path =
        if let ResolveResult::Info(info) = resolver.resolve(&case_path, "exports-field").unwrap() {
            info.path
        } else {
            panic!("error")
        };

    assert_eq!(
        resolver
            .load_side_effects(&exports_field_path)
            .unwrap()
            .unwrap()
            .0,
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "package.json"
        ])
    );

    assert!(matches!(
        resolver
            .load_side_effects(&exports_field_path)
            .unwrap()
            .unwrap()
            .1,
        Some(SideEffects::Bool(false))
    ));

    // match resolver
    //     .load_side_effects(&p(vec!["incorrect-package", "sideeffects-map"]))
    //     .unwrap_err()
    // {
    //     Error::UnexpectedValue(error) => assert_eq!(
    //         error,
    //         format!(
    //             "sideEffects in {} had unexpected value {{}}",
    //             p(vec!["incorrect-package", "sideeffects-map", "package.json"]).display()
    //         )
    //     ),
    //     _ => unreachable!(),
    // }

    // match resolver
    //     .load_side_effects(&p(vec!["incorrect-package", "sideeffects-other-in-array"]))
    //     .unwrap_err()
    // {
    //     Error::UnexpectedValue(error) => assert_eq!(
    //         error,
    //         format!(
    //             "sideEffects in {} had unexpected value 1",
    //             p(vec![
    //                 "incorrect-package",
    //                 "sideeffects-other-in-array",
    //                 "package.json"
    //             ])
    //             .display()
    //         )
    //     ),
    //     _ => unreachable!(),
    // }

    assert!(resolver.load_side_effects(&p(vec![])).unwrap().is_none());
}

#[test]
fn shared_cache_test2() {
    let case_path = p(vec!["browser-module"]);
    let cache = Arc::new(Cache::default());
    let resolver1 = Resolver::new(Options {
        browser_field: true,
        external_cache: Some(cache.clone()),
        ..Default::default()
    });
    should_ignored(&resolver1, &case_path, "./lib/ignore.js");

    let resolver2 = Resolver::new(Options {
        external_cache: Some(cache.clone()),
        ..Default::default()
    });
    should_equal(
        &resolver2,
        &case_path,
        "./lib/ignore.js",
        case_path.join("lib").join("ignore.js").to_path_buf(),
    );

    let resolver3 = Resolver::new(Options {
        external_cache: Some(cache.clone()),
        main_fields: vec!["module".to_string()],
        ..Default::default()
    });
    should_equal(
        &resolver3,
        &p(vec!["main-field-inexist"]),
        ".",
        p(vec!["main-field-inexist", "module.js"]),
    );
}

#[test]
fn empty_test() {
    let case_path = p(vec!["empty"]);
    let resolver = Resolver::new(Options::default());
    should_resolve_failed(&resolver, &case_path, ".");
    should_resolve_failed(&resolver, &p(vec![]), "./empty");
}

#[test]
fn browser_it_self() {
    let case_path = p(vec!["browser-to-self"]);
    let resolver = Resolver::new(Options {
        browser_field: true,
        condition_names: vec_to_set(vec!["browser"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/b.js",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/b.mjs",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.mjs"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/package.json",
        p(vec![
            "browser-to-self",
            "node_modules",
            "b.js",
            "package.json",
        ]),
    );
    should_overflow(&resolver, &case_path, "c.js");
    let resolver = Resolver::new(Options {
        browser_field: true,
        main_fields: vec![
            "browser".to_string(),
            "module".to_string(),
            "main".to_string(),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "c.js",
        p(vec!["browser-to-self", "node_modules", "c.js", "c.js"]),
    );
    let resolver = Resolver::new(Options {
        browser_field: false,
        main_fields: vec![
            "browser".to_string(),
            "module".to_string(),
            "main".to_string(),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "c.js",
        p(vec!["browser-to-self", "node_modules", "c.js", "c.js"]),
    );
    let resolver = Resolver::new(Options {
        browser_field: false,
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_resolve_failed(&resolver, &case_path, "c.js");
}

#[test]
fn self_in_dep_test() {
    let path = p(vec!["self-is-dep", "src", "index.js"]);
    let resolver = Resolver::new(Options::default());
    should_equal(
        &resolver,
        &path,
        "@scope/self-is-dep/src/a",
        p(vec!["self-is-dep", "src", "b.js"]),
    );
}
