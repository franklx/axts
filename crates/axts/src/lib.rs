//! # Axts
//!
//! `axts` is a code-first TypeScript API client/type generator library. It
//! aims for tight integrations with frameworks and following their
//! conventions, while tries to be out of the way when it is not needed.
//!
//! Axts walks the routes registered on a web framework's router and, for each
//! operation, emits:
//!
//! - TypeScript type declarations for every Rust type involved, generated via
//!   [`ts-rs`](https://docs.rs/ts-rs).
//! - A small, typed `fetch`-based API client with one function per route.
//!
//! This used to generate an [Open API](https://www.openapis.org/) document
//! instead, see [`typescript`] for the new output format.
//!
//! The goal is to minimize the learning curve, mental context switches
//! and make documentation somewhat slightly less of a chore.
//!
//! See the [examples](https://github.com/franklx/axts/tree/master/examples)
//! to see how Axts is used with various frameworks.
//!
//! Currently only Open API version `3.1.x` is supported.
//!
//! Previous releases of axts relied heavily on macros, and the
//! [`linkme`](https://docs.rs/linkme/latest/linkme/) crate for automagic global state.
//! While it all worked, macros were hard to reason about,
//! rustfmt did not work with them, code completion was hit-and-miss.
//!
//! With `0.5.0`, axts was rewritten and instead it is based on on good old functions,
//! type inference and declarative APIs based on the builder pattern.
//!
//! Now all documentation can be traced in the source code[^1],
//! no more macro and global magic all over the place.[^2]
//!
//! [^1]: and with [tracing] spans
//!
//! [^2]: A thread-local context is still used for some settings and
//! shared state.
//!
//! ## Type-based Generation
//!
//! The library uses [`ts-rs`](https://docs.rs/ts-rs) for TypeScript type
//! generation. It should be enough to slap `#[derive(ts_rs::TS)]`
//! alongside [serde]'s `Serialize/Deserialize` for JSON-based APIs.
//!
//! Additionally the [`OperationInput`] and [`OperationOutput`] traits
//! are used for extractor and response types in frameworks to automatically generate
//! expected HTTP parameter and response documentation.
//!
//! For example a `Json<T>` extractor will generate an `application/json`
//! request body referencing the TypeScript type of `T` if it implements
//! [`ts_rs::TS`].
//!
//! ## Declarative Documentation
//!
//! All manual documentation is based on composable [`transform`]
//! functions and builder-pattern-like API.
//!
//! ## Supported Frameworks
//!
//! - [axum](https://docs.rs/axum/latest/axum/): [`axts::axum`](axum).
//! - [actix-web](https://docs.rs/actix-web/latest/actix_web/) is **not
//!   supported** since `0.5.0` only due to lack of developer capacity,
//!   but it's likely to be supported again in the future. If you use
//!   `actix-web` you can still use the macro-based `0.4.*` version of the
//!   library for the time being.
//!
//! ## Errors
//!
//! Some errors occur during code generation, these
//! are usually safe to ignore but might indicate bugs.
//!
//! By default no action is taken on errors, in order to handle them
//! it is possible to register an error handler in the thread-local context
//! with [`axts::generate::on_error`](crate::generate::on_error).
//!
//! False positives are chosen over silently swallowing potential
//! errors, these might happen when there is not enough contextual
//! information to determine whether an error is in fact an error.
//! It is important to keep this in mind, without any filters
//! **simply panicking on all errors is not advised**, especially
//! not in production.
//!
//! ## Feature Flags
//!
//! No features are enabled by default.
//!
//! - `macros`: additional helper macros
//!
//! ### Third-party trait implementations
//!
//! - `bytes`
//! - `http`
//!
//! ### axum integration
//!
//! `axum` and its features gates:
//!
//! - `axum`
//! - `axum-form`
//! - `axum-json`
//! - `axum-matched-path`
//! - `axum-multipart`
//! - `axum-original-uri`
//! - `axum-query`
//! - `axum-tokio` (for `ConnectInfo`)
//! - `axum-ws` (WebSockets)
//!
//! `axum-extra` and its features gates:
//!
//! - `axum-extra`
//! - `axum-extra-cookie`
//! - `axum-extra-cookie-private`
//! - `axum-extra-form`
//! - `axum-extra-headers`
//! - `axum-extra-query`
//! - `axum-extra-json-deserializer`
//!
//! ## MSRV
//!
//! The library will always support the latest stable Rust version,
//! it might support older versions but without guarantees.
//!
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::pedantic, missing_docs, unreachable_pub, rust_2018_idioms)]
#![allow(
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::single_match_else,
    clippy::manual_let_else
)]

// Required for using macros such as the `OperationIo` derive macro in tests.
// These macros use paths starting with `axts::` which would otherwise be invalid within this crate.
#[cfg(test)]
extern crate self as axts;

#[macro_use]
mod macros;
mod impls;

pub mod error;
pub mod generate;
pub mod operation;

pub mod openapi;
pub mod transform;
pub mod typescript;
pub mod util;

#[cfg(feature = "axum")]
pub mod axum;

mod helpers;

pub use helpers::{
    no_api::NoApi, use_api::IntoApi, use_api::UseApi, with_api::ApiOverride, with_api::WithApi,
};

pub use error::Error;
pub use operation::{OperationInput, OperationOutput};

#[cfg(feature = "macros")]
pub use axts_macros::OperationIo;
