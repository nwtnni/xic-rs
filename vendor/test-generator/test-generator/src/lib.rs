//! # Overview
//! This crate provides `#[test_resources]` and `#[bench_resources]` procedural macro attributes
//! that generates multiple parametrized tests using one body with different resource input parameters.
//! A test is generated for each resource matching the specific resource location pattern.
//!
//! [![Crates.io](https://img.shields.io/crates/v/test-generator.svg)](https://crates.io/crates/test-generator)
//! [![MIT License](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/frehberg/test-generator/blob/master/LICENSE-MIT)
//! [![Apache License](http://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/frehberg/test-generator/blob/master/LICENSE-APACHE)
//! [![Example](http://img.shields.io/badge/crate-Example-red.svg)](https://github.com/frehberg/test-generator/tree/master/example)
//!
//! [Documentation](https://docs.rs/test-generator/)
//!
//! [Repository](https://github.com/frehberg/test-generator/)
//!
//! # Getting Started
//!
//! First of all you have to add this dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! test-generator = "^0.3"
//! ```
//! The test-functionality is supports stable Rust since version 1.30,
//! whereas the bench-functionality requires an API from unstable nightly release.
//!
//! ```ignore
//! #![cfg(test)]
//! extern crate test_generator;
//!
//! // Don't forget that procedural macros are imported with `use` statement,
//! // for example importing the macro 'test_resources'
//! #![cfg(test)]
//! use test_generator::test_resources;
//! ```
//!
//! # Example usage `test`:
//!
//! The `test` functionality supports the stable release of Rust-compiler since version 1.30.
//!
//! ```ignore
//! #![cfg(test)]
//! extern crate test_generator;
//!
//! use test_generator::test_resources;
//!
//! #[test_resources("res/*/input.txt")]
//! fn verify_resource(resource: &str) {
//!    assert!(std::path::Path::new(resource).exists());
//! }
//! ```
//!
//! Output from `cargo test` for 3 test-input-files matching the pattern, for this example:
//!
//! ```console
//! $ cargo test
//!
//! running 3 tests
//! test tests::verify_resource_res_set1_input_txt ... ok
//! test tests::verify_resource_res_set2_input_txt ... ok
//! test tests::verify_resource_res_set3_input_txt ... ok
//!
//! test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
//! ```
//! # Example usage `bench`:
//!
//! The `bench` functionality requires the nightly release of the Rust-compiler.
//!
//! ```ignore
//! #![feature(test)] // nightly feature required for API test::Bencher
//!
//! #[macro_use]
//! extern crate test_generator;
//!
//! extern crate test; /* required for test::Bencher */
//!
//! mod bench {
//!     #[bench_resources("res/*/input.txt")]
//!     fn measure_resource(b: &mut test::Bencher, resource: &str) {
//!         let path = std::path::Path::new(resource);
//!         b.iter(|| path.exists());
//!     }
//! }
//! ```
//! Output from `cargo +nightly bench` for 3 bench-input-files matching the pattern, for this example:
//!
//! ```console
//! running 3 tests
//! test bench::measure_resource_res_set1_input_txt ... bench:       2,492 ns/iter (+/- 4,027)
//! test bench::measure_resource_res_set2_input_txt ... bench:       2,345 ns/iter (+/- 2,167)
//! test bench::measure_resource_res_set3_input_txt ... bench:       2,269 ns/iter (+/- 1,527)
//!
//! test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 0 filtered out
//! ```
//!
//! # Example
//! The [example](https://github.com/frehberg/test-generator/tree/master/example) demonstrates usage
//! and configuration of these macros, in combination with the crate
//! `build-deps` monitoring for any change of these resource files and conditional rebuild.
//!
//! # Internals
//! Let's assume the following code and 3 files matching the pattern "res/*/input.txt"
//! ```ignore
//! #[test_resources("res/*/input.txt")]
//! fn verify_resource(resource: &str) { assert!(std::path::Path::new(resource).exists()); }
//! ```
//! the generated code for this input resource will look like
//! ```
//! #[test]
//! #[allow(non_snake_case)]
//! fn verify_resource_res_set1_input_txt() { verify_resource("res/set1/input.txt".into()); }
//! #[test]
//! #[allow(non_snake_case)]
//! fn verify_resource_res_set2_input_txt() { verify_resource("res/set2/input.txt".into()); }
//! #[test]
//! #[allow(non_snake_case)]
//! fn verify_resource_res_set3_input_txt() { verify_resource("res/set3/input.txt".into()); }
//! ```
//! Note: The trailing `into()` method-call permits users to implement the `Into`-Trait for auto-conversations.
//!
extern crate glob;
extern crate proc_macro;

use proc_macro::TokenStream;

use self::glob::{glob, Paths};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, ItemFn, Lit};

// Form canonical name without any punctuation/delimiter or special character
fn canonical_fn_name(s: &str) -> String {
    // remove delimiters and special characters
    s.replace(
        &['"', ' ', '.', ':', '-', '*', '/', '\\', '\n', '\t', '\r'][..],
        "_",
    )
}

/// Return the concatenation of two token-streams
fn concat_ts_cnt(
    accu: (u64, proc_macro2::TokenStream),
    other: proc_macro2::TokenStream,
) -> (u64, proc_macro2::TokenStream) {
    let (accu_cnt, accu_ts) = accu;
    (accu_cnt + 1, quote! { #accu_ts #other })
}

/// MacroAttributes elements
struct MacroAttributes {
    glob_pattern: Lit,
}

/// MacroAttributes parser
impl Parse for MacroAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let glob_pattern: Lit = input.parse()?;
        if !input.is_empty() {
            panic!("found multiple parameters, expected one");
        }

        Ok(MacroAttributes { glob_pattern })
    }
}

/// Macro generating test-functions, invoking the fn for each item matching the resource-pattern.
///
/// The resource-pattern must not expand to empty list, otherwise an error is raised.
/// The generated test-functions is aregular tests, being compiled by the rust-compiler; and being
/// executed in parallel by the test-framework.
/// ```
/// #[cfg(test)]
/// extern crate test_generator;
///
/// #[cfg(test)]
/// mod tests {
///   use test_generator::test_resources;
///
///   #[test_resources("res/*/input.txt")]
///   fn verify_resource(resource: &str) {
///      assert!(std::path::Path::new(resource).exists());
///   }
/// }
/// ```
/// Assuming the following package layout with test file `mytests.rs` and resource folder `res/`,
/// the output below will be printed on console. The functionality of `build.rs` is explained at crate
/// [build-deps](https://crates.io/crates/build-deps) and demonstrated with
/// [example](https://github.com/frehberg/test-generator/tree/master/example)
///
/// ```ignore
/// ├── build.rs
/// ├── Cargo.toml
/// ├── res
/// │   ├── set1
/// │   │   ├── expect.txt
/// │   │   └── input.txt
/// │   ├── set2
/// │   │   ├── expect.txt
/// │   │   └── input.txt
/// │   └── set3
/// │       ├── expect.txt
/// │       └── input.txt
/// ├── src
/// │   └── main.rs
/// ├── benches
/// │   └── mybenches.rs
/// └── tests
///     └── mytests.rs
/// ```
/// Producing the following test output
///
/// ```ignore
/// $ cargo test
///
/// running 3 tests
/// test tests::verify_resource_res_set1_input_txt ... ok
/// test tests::verify_resource_res_set2_input_txt ... ok
/// test tests::verify_resource_res_set3_input_txt ... ok
///
/// test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
/// ```
#[proc_macro_attribute]
pub fn test_resources(attrs: TokenStream, func: TokenStream) -> TokenStream {
    let MacroAttributes { glob_pattern } = parse_macro_input!(attrs as MacroAttributes);

    let pattern = match glob_pattern {
        Lit::Str(l) => l.value(),
        Lit::Bool(l) => panic!("expected string parameter, got '{}'", &l.value),
        Lit::Byte(l) => panic!("expected string parameter, got '{}'", &l.value()),
        Lit::ByteStr(_) => panic!("expected string parameter, got byte-string"),
        Lit::Char(l) => panic!("expected string parameter, got '{}'", l.value()),
        Lit::Int(l) => panic!("expected string parameter, got '{}'", l),
        Lit::Float(l) => panic!("expected string parameter, got '{}'", l),
        _ => panic!("expected string parameter"),
    };

    let func_copy: proc_macro2::TokenStream = func.clone().into();

    let func_ast: ItemFn = syn::parse(func).expect("failed to parse tokens as a function");

    let func_ident = func_ast.sig.ident;

    let paths: Paths =
        glob(&pattern).unwrap_or_else(|_| panic!("No such file or directory {}", &pattern));

    // for each path generate a test-function and fold them to single tokenstream
    let result = paths
        .map(|path| {
            let path_as_str = path
                .expect("No such file or directory")
                .into_os_string()
                .into_string()
                .expect("bad encoding");
            let test_name = format!("{}_{}", func_ident, &path_as_str);

            // create function name without any delimiter or special character
            let test_name = canonical_fn_name(&test_name);

            // quote! requires proc_macro2 elements
            let test_ident = proc_macro2::Ident::new(&test_name, proc_macro2::Span::call_site());

            let item = quote! {
                #[test]
                #[allow(non_snake_case)]
                fn # test_ident () -> anyhow::Result<()> {
                    # func_ident ( #path_as_str .into() )
                }
            };

            item
        })
        .fold((0, func_copy), concat_ts_cnt);

    // panic, the pattern did not match any file or folder
    if result.0 == 0 {
        panic!("no resource matching the pattern {}", &pattern);
    }
    // transforming proc_macro2::TokenStream into proc_macro::TokenStream
    result.1.into()
}

/// Macro generating bench-functions, invoking the fn for each item matching the resource-pattern.
///
/// The resource-pattern must not expand to empty list, otherwise an error is raised.
/// The generated test-functions is a regular bench, being compiled by the rust-compiler; and being
/// executed in sequentially by the bench-framework.
/// ```ignore
/// #![feature(test)] // nightly feature required for API test::Bencher
///
/// #[cfg(test)]
/// extern crate test; /* required for test::Bencher */
/// #[cfg(test)]
/// extern crate test_generator;
///
/// #[cfg(test)]
/// mod tests {
///   use test_generator::bench_resources;
///
///   #[bench_resources("res/*/input.txt")]
///   fn measure_resource(b: &mut test::Bencher, resource: &str) {
///      let path = std::path::Path::new(resource);
///      b.iter(|| path.exists());
///   }
/// }
/// ```
/// Assuming the following package layout with the bench file `mybenches.rs` and resource folder `res/`,
/// the output below will be printed on console. The functionality of `build.rs` is explained at crate
/// [build-deps](https://crates.io/crates/build-deps) and demonstrated with
/// [example](https://github.com/frehberg/test-generator/tree/master/example)
///
/// ```ignore
/// ├── build.rs
/// ├── Cargo.toml
/// ├── res
/// │   ├── set1
/// │   │   ├── expect.txt
/// │   │   └── input.txt
/// │   ├── set2
/// │   │   ├── expect.txt
/// │   │   └── input.txt
/// │   └── set3
/// │       ├── expect.txt
/// │       └── input.txt
/// ├── src
/// │   └── main.rs
/// ├── benches
/// │   └── mybenches.rs
/// └── tests
///     └── mytests.rs
/// ```
/// Output from `cargo +nightly bench` for 3 bench-input-files matching the pattern, for this example:
///
/// ```ignore
/// running 3 tests
/// test bench::measure_resource_res_set1_input_txt ... bench:       2,492 ns/iter (+/- 4,027)
/// test bench::measure_resource_res_set2_input_txt ... bench:       2,345 ns/iter (+/- 2,167)
/// test bench::measure_resource_res_set3_input_txt ... bench:       2,269 ns/iter (+/- 1,527)
///
/// test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 0 filtered out
/// ```
#[proc_macro_attribute]
pub fn bench_resources(attrs: TokenStream, func: TokenStream) -> TokenStream {
    let MacroAttributes { glob_pattern } = parse_macro_input!(attrs as MacroAttributes);

    let pattern = match glob_pattern {
        Lit::Str(l) => l.value(),
        Lit::Bool(l) => panic!("expected string parameter, got '{}'", l.value),
        Lit::Byte(l) => panic!("expected string parameter, got '{}'", l.value()),
        Lit::ByteStr(_) => panic!("expected string parameter, got byte-string"),
        Lit::Char(l) => panic!("expected string parameter, got '{}'", l.value()),
        Lit::Int(l) => panic!("expected string parameter, got '{}'", l),
        Lit::Float(l) => panic!("expected string parameter, got '{}'", l),
        _ => panic!("expected string parameter"),
    };

    let func_copy: proc_macro2::TokenStream = func.clone().into();

    let func_ast: ItemFn = syn::parse(func).expect("failed to parse tokens as a function");

    let func_ident = func_ast.sig.ident;

    let paths: Paths =
        glob(&pattern).unwrap_or_else(|_| panic!("No such file or directory {}", &pattern));

    // for each path generate a test-function and fold them to single tokenstream
    let result = paths
        .map(|path| {
            let path_as_str = path
                .expect("No such file or directory")
                .into_os_string()
                .into_string()
                .expect("bad encoding");
            let test_name = format!("{}_{}", func_ident, &path_as_str);

            // create function name without any delimiter or special character
            let test_name = canonical_fn_name(&test_name);

            // quote! requires proc_macro2 elements
            let test_ident = proc_macro2::Ident::new(&test_name, proc_macro2::Span::call_site());

            let item = quote! {
                #[bench]
                #[allow(non_snake_case)]
                fn # test_ident (b: &mut test::Bencher) {
                    # func_ident ( b, #path_as_str .into() );
                }
            };

            item
        })
        .fold((0, func_copy), concat_ts_cnt);

    // panic, the pattern did not match any file or folder
    if result.0 == 0 {
        panic!("no resource matching the pattern {}", &pattern);
    }

    // transforming proc_macro2::TokenStream into proc_macro::TokenStream
    result.1.into()
}
