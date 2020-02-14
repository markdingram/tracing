//! Utilities for enriching error handling with [`tracing`] diagnostic
//! information.
//!
//! # Overview
//!
//! [`tracing`] is a framework for instrumenting Rust programs to collect
//! scoped, structured, and async-aware diagnostics. This crate provides
//! integrations between [`tracing`] instrumentation and Rust error handling. It
//! enables enriching error types with diagnostic information from `tracing`
//! [span] contexts, formatting those contexts when errors are displayed, and
//! automatically generate `tracing` [events] when errors occur.
//!
//! The crate provides the following:
//!
//! * [`SpanTrace`], a captured trace of the current `tracing` [span] context
//!
//! * [`ErrorLayer`], a [subscriber layer] which enables capturing `SpanTrace`s
//!
//! * [`InstrumentResult`]/[`InstrumentError`], traits for bundling errors with a [`SpanTrace`]
//! with their [`in_current_span()`] methods.
//!
//! * [`ExtractSpanTrace`]extension trait, for extracting `SpanTrace`s from behind `dyn Error` trait
//! objects.
//!
//! **Note**: This crate is currently experimental.
//!
//! *Compiler support: requires `rustc` 1.39+*
//!
//! ## Feature Flags
//!
//! - `stack-error` - Enables an experimental version of [`TracedError`] that statically wraps an error
//! type without using heap allocations which can still extract its SpanTrace from behind a
//! [`std::error::Error`] trait object via type erasure.
//!
//! ## Usage
//!
//! `tracing-error` provides the [`SpanTrace`] type, which captures the current `tracing` span
//! context when it is constructed and allows it to be displayed at a later time.
//!
//! For example:
//!
//! ```rust
//! use std::{fmt, error::Error};
//! use tracing_error::SpanTrace;
//!
//! #[derive(Debug)]
//! pub struct MyError {
//!     context: SpanTrace,
//!     // ...
//! }
//!
//! impl fmt::Display for MyError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         // ... format other parts of the error ...
//!
//!         self.context.fmt(f)?;
//!
//!         // ... format other error context information, cause chain, etc ...
//!         # Ok(())
//!     }
//! }
//!
//! impl Error for MyError {}
//!
//! impl MyError {
//!     pub fn new() -> Self {
//!         Self {
//!             context: SpanTrace::capture(),
//!             // ... other error information ...
//!         }
//!     }
//! }
//! ```
//!
//! This crate also provides the [`InstrumentResult`]/[`InstrumentError`] traits which can be used
//! to wrap errors with a [`TracedError`] which bundles the inner error with a [`SpanTrace`].
//!
//! ```rust
//! # use std::error::Error;
//! use tracing_error::InstrumentResult;
//!
//! # fn fake_main() -> Result<(), Box<dyn Error>> {
//! std::fs::read_to_string("myfile.txt").in_current_span()?;
//! # Ok(())
//! # }
//! ```
//!
//! Once an error has been wrapped with with a [`TracedError`] the [`SpanTrace`] can be extracted
//! one of 3 ways, either via [`TracedError`]'s `Display`/`Debug` implementations, or via the
//! [`ExtractSpanTrace`] trait.
//!
//! ```rust
//! use std::error::Error;
//! use tracing_error::ExtractSpanTrace as _;
//!
//! fn print_extracted_spantraces(error: &(dyn Error + 'static)) {
//!     let mut error = Some(error);
//!     let mut ind = 0;
//!
//!     eprintln!("Error:");
//!
//!     while let Some(err) = error {
//!         if let Some(spantrace) = err.span_trace() {
//!             eprintln!("found a spantrace!:\n{}", spantrace);
//!         } else {
//!             eprintln!("{:>4}: {}", ind, err);
//!         }
//!
//!         error = err.source();
//!         ind += 1;
//!     }
//! }
//!
//! fn print_naive_spantraces(error: &(dyn Error + 'static)) {
//!     let mut error = Some(error);
//!     let mut ind = 0;
//!
//!     eprintln!("Error:");
//!
//!     while let Some(err) = error {
//!         eprintln!("{:>4}: {}", ind, err);
//!         error = err.source();
//!         ind += 1;
//!     }
//! }
//! ```
//!
//! Applications that wish to use `tracing-error`-enabled errors should
//! construct an [`ErrorLayer`] and add it to their [`Subscriber`] in order to
//! enable capturing [`SpanTrace`]s. For example:
//!
//! ```rust
//! use tracing_error::ErrorLayer;
//! use tracing_subscriber::prelude::*;
//!
//! fn main() {
//!     let subscriber = tracing_subscriber::Registry::default()
//!         // any number of other subscriber layers may be added before or
//!         // after the `ErrorLayer`...
//!         .with(ErrorLayer::default());
//!
//!     // set the subscriber as the default for the application
//!     tracing::subscriber::set_global_default(subscriber);
//! }
//! ```
//!
//! [`SpanTrace`]: struct.SpanTrace.html
//! [`ErrorLayer`]: struct.ErrorLayer.html
//! [`TracedError`]: struct.ErrorLayer.html
//! [`InstrumentResult`]: trait.SpanTrace.html
//! [`InstrumentError`]: trait.SpanTrace.html
//! [`ExtractSpanTrace`]: trait.SpanTrace.html
//! [`in_current_span()`]: trait.InstrumentResult.html#tymethod.in_current_span
//! [span]: https://docs.rs/tracing/latest/tracing/span/index.html
//! [event]: https://docs.rs/tracing/latest/tracing/struct.Event.html
//! [subscriber layer]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html
//! [`tracing`]: https://docs.rs/tracing
//! [`std::error::Error`]: https://doc.rust-lang.org/nightly/std/error/trait.Error.html
#![doc(html_root_url = "https://docs.rs/tracing-error/0.1.1")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
mod backtrace;
#[cfg(not(feature = "stack-error"))]
mod heap_error;
mod layer;
#[cfg(feature = "stack-error")]
mod stack_error;

pub use self::backtrace::SpanTrace;
#[cfg(not(feature = "stack-error"))]
pub use self::heap_error::TracedError;
pub use self::layer::ErrorLayer;
#[cfg(feature = "stack-error")]
pub use self::stack_error::TracedError;

/// Extension trait for instrumenting errors with `SpanTrace`s
pub trait InstrumentError {
    /// The type of the wrapped error after instrumentation
    type Instrumented;

    /// Instrument an Error by bundling it with a SpanTrace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_error::{TracedError, InstrumentError};
    ///
    /// fn wrap_error(e: impl std::error::Error + Send + Sync + 'static) -> TracedError {
    ///     e.in_current_span()
    /// }
    /// ```
    fn in_current_span(self) -> Self::Instrumented;
}

/// Extension trait for instrumenting errors in `Result`s with `SpanTrace`s
pub trait InstrumentResult<T> {
    /// The type of the wrapped error after instrumentation
    type Instrumented;

    /// Instrument an Error by bundling it with a SpanTrace
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{io, fs};
    /// use tracing_error::{TracedError, InstrumentResult};
    ///
    /// # fn fallible_fn() -> io::Result<()> { fs::read_dir("......").map(drop) };
    ///
    /// fn do_thing() -> Result<(), TracedError> {
    ///     fallible_fn().in_current_span()
    /// }
    /// ```
    fn in_current_span(self) -> Result<T, Self::Instrumented>;
}

/// A trait for extracting SpanTraces created by `in_current_span()` from `dyn Error` trait objects
pub trait ExtractSpanTrace {
    /// Attempts to downcast to a `TracedError` and return a reference to its SpanTrace
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_error::ExtractSpanTrace;
    /// use std::error::Error;
    ///
    /// fn print_span_trace(e: &(dyn Error + 'static)) {
    ///     let span_trace = e.span_trace();
    ///     if let Some(span_trace) = span_trace {
    ///         println!("{}", span_trace);
    ///     }
    /// }
    /// ```
    fn span_trace(&self) -> Option<&SpanTrace>;
}

/// The `tracing-error` prelude.
///
/// This brings into scope the `InstrumentError` and `ExtractSpanTrace` extension traits that are used to
/// attach Spantraces to errors and subsequently retrieve them from dyn Errors.
pub mod prelude {
    pub use crate::{ExtractSpanTrace as _, InstrumentError as _, InstrumentResult as _};
}
