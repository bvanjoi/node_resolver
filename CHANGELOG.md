# Changelog

## Unreleased

- The property type of `Request` changed from `String` to `SmolStr`.

## 0.0.10

- optimized constants in code.

## 0.0.9

- add `enable_unsafe_cache` in `ResolverOptions`, because user sometimes change the `DescriptionFile`, which can lead to some potential problems in `self.cache`.

## 0.0.8

- support `prefer_relative` feature.
- remove `with_xxx` methods, instead of manual assignment.

## 0.0.7

- public `Options`, and change it `description_file` type from `String` to `Option<String>`.

## 0.0.5 && 0.0.6

yanked
 
## 0.0.4

- support `Debug` trait. According to [Debuggability](https://rust-lang.github.io/api-guidelines/debuggability.html#debuggability), all public API types should be implements `Debug`.

## 0.0.3

- (fixture): use `dashmap` to implement cache.
- (fixture): change `resolver.with_base_dir(xxxx).resolve(target)` to `resolver.resolve(xxxx, target)`.
- (chore): add `Windows` and `MacOS` ci environment.
- (refactor): Add coverage test.

## 0.0.2

- support [`exports`](https://nodejs.org/api/packages.html#exports) and [`imports`](https://nodejs.org/api/packages.html#imports) in package.json.

## 0.0.1

init