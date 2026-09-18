//! Thread-local context for common settings for documentation generation.

use std::any::TypeId;
use std::cell::RefCell;
use std::path::Path;

use indexmap::IndexMap;
use ts_rs::TS;

use crate::error::Error;
use crate::openapi::SchemaObject;

thread_local! {
    static GEN_CTX: RefCell<GenContext> = RefCell::new(GenContext::new());
}

/// Access the current thread-local context for
/// API documentation generation.
pub fn in_context<R, F>(cb: F) -> R
where
    F: FnOnce(&mut GenContext) -> R,
{
    GEN_CTX.with(|ctx| cb(&mut ctx.borrow_mut()))
}

/// Register an error handler in the current thread-local context.
///
/// Only one handler is allowed at a time, this
/// function will overwrite the existing one.
///
/// Due to the design of the library in some cases
/// errors can be false positives that cannot be
/// avoided.
///
/// It is advised **not to panic** in this handler
/// unless you are interested in the backtrace for
/// a specific error.
pub fn on_error(handler: impl Fn(Error) + 'static) {
    in_context(|ctx| ctx.error_handler = Some(Box::new(handler)));
}

/// Set the inferred status code of empty responses (`()`).
///
/// Some frameworks might use `204` for empty responses, whereas
/// others will set `200`.
///
/// The default value depends on the framework feature.
pub fn inferred_empty_response_status(status: u16) {
    in_context(|ctx| {
        ctx.no_content_status = status;
    });
}

/// Infer responses based on request handler
/// return types.
///
/// This is enabled by default.
pub fn infer_responses(infer: bool) {
    in_context(|ctx| {
        ctx.infer_responses = infer;
    });
}

/// Output all theoretically possible error responses
/// including framework-specific ones.
///
/// This is disabled by default.
pub fn all_error_responses(infer: bool) {
    in_context(|ctx| {
        ctx.all_error_responses = infer;
    });
}

/// Export all TypeScript types that were registered so far (by types
/// appearing in documented request/response bodies and parameters) to
/// the given output directory, using [`ts-rs`](https://docs.rs/ts-rs).
///
/// Each type is written to its own `.ts` file, `ts-rs` takes care of
/// generating the necessary `import` statements between them.
pub fn export_types(out_dir: impl AsRef<Path>) -> Result<(), Error> {
    in_context(|ctx| ctx.export_types(out_dir))
}

/// Reset the state of the thread-local context.
///
/// Currently clears:
///
/// - the registry of TypeScript types collected so far
/// - disables inferred responses
///
/// This function is not required in most cases.
pub fn reset_context() {
    in_context(|ctx| *ctx = GenContext::new());
}

/// A single TypeScript type that was registered in a [`GenContext`],
/// along with the means to export it (and its dependencies) to disk.
#[derive(Clone)]
struct RegisteredType {
    name: String,
    export: fn(&ts_rs::Config) -> Result<(), ts_rs::ExportError>,
}

/// A context for documentation generation that provides settings
/// and a registry of TypeScript types generated via `ts-rs`.
pub struct GenContext {
    /// Configuration used when generating and exporting TypeScript types.
    pub ts_config: ts_rs::Config,

    pub(crate) infer_responses: bool,

    pub(crate) all_error_responses: bool,

    /// Status code for no content.
    pub(crate) no_content_status: u16,

    /// TypeScript types registered so far, keyed by their Rust [`TypeId`]
    /// so that each distinct type is only exported once.
    types: IndexMap<TypeId, RegisteredType>,

    /// The following filter is used internally
    /// to reduce the amount of false positives
    /// when possible.
    pub(crate) show_error: fn(&Error) -> bool,
    error_handler: Option<Box<dyn Fn(Error)>>,
}

impl GenContext {
    fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "axum")] {
                let no_content_status = 200;
            } else {
                let no_content_status = 204;
            }
        }

        Self {
            ts_config: ts_rs::Config::new(),
            infer_responses: true,
            all_error_responses: false,
            show_error: default_error_filter,
            error_handler: None,
            no_content_status,
            types: IndexMap::new(),
        }
    }

    pub(crate) fn reset_error_filter(&mut self) {
        self.show_error = default_error_filter;
    }

    /// Add an error in the current context.
    #[tracing::instrument(skip_all)]
    pub fn error(&mut self, error: Error) {
        if let Some(handler) = &self.error_handler {
            if !(self.show_error)(&error) {
                return;
            }

            handler(error);
        }
    }

    /// Register a type for TypeScript generation, returning a
    /// [`SchemaObject`] referencing it by name.
    ///
    /// The type is only exported once even if registered multiple times.
    pub fn register_type<T>(&mut self) -> SchemaObject
    where
        T: TS + 'static + ?Sized,
    {
        let name = T::name(&self.ts_config);

        self.types
            .entry(TypeId::of::<T>())
            .or_insert_with(|| RegisteredType {
                name: name.clone(),
                export: T::export_all,
            });

        SchemaObject {
            ts_type: name,
            description: T::docs(),
            example: None,
        }
    }

    /// Export all types registered so far to the given directory.
    pub fn export_types(&self, out_dir: impl AsRef<Path>) -> Result<(), Error> {
        let cfg = ts_rs::Config::new().with_out_dir(out_dir.as_ref());

        for registered in self.types.values() {
            (registered.export)(&cfg)
                .map_err(|e| Error::Other(Box::new(TsExportError(registered.name.clone(), e))))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct TsExportError(String, ts_rs::ExportError);

impl std::fmt::Display for TsExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to export TypeScript type {:?}: {}", self.0, self.1)
    }
}

impl std::error::Error for TsExportError {}

fn default_error_filter(_: &Error) -> bool {
    true
}
