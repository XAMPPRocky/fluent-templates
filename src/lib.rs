//! # Fluent Templates: Templating for Fluent
//!
//! This crate provides "loaders" that are able to load fluent strings based on
//! simple language negotiation, and the `FluentHelper` which is an opague
//! type that provides the integration between a given templating engine such as
//! handlebars or tera.
//!
//! ## Loaders
//! Currently this crate provides two different kinds of loaders that cover two
//! main use cases.
//!
//! - [`static_loader!`] — A macro that generates a loader that loads the
//!   localisations into static memory. Useful for when your localisations are
//!   known at **compile-time**.
//! - [`ArcLoader`] — A struct that atomically stores and reference counts the
//!   localisations. Useful for when your localisations are only known at
//!   **run-time**.
//!
//! ## Handlebars Example
//! The easiest way to use `fluent-templates` is to use the [`static_loader!`]
//! macro:
//!
//! ```rust
//! # #[cfg(feature = "handlebars")] {
//! use fluent_templates::*;
//! use handlebars::*;
//! use serde_json::*;
//!
//! static_loader!(create_loader, "./locales/", "en-US");
//!
//! fn init(handlebars: &mut Handlebars) {
//!     let loader = create_loader();
//!     let helper = FluentHelper::new(loader);
//!     handlebars.register_helper("fluent", Box::new(helper));
//! }
//!
//! fn render_page(handlebars: &Handlebars) -> String {
//!     let data = json!({"lang": "zh-CN"});
//!     handlebars.render_template("{{fluent \"foo-bar\"}} baz", &data).unwrap()
//! }
//! # }
//! ```
//!
//! You should have a `locales/` folder somewhere with one folder per language
//! code, containing all of your FTL files. See the [`static_loader!`] macro
//! for more options.
//!
//! Make sure the [`handlebars::Context`] has a top-level `"lang"` field when rendering.
//!
//! ### Handlebars helper syntax.
//!
//! The main helper provided is the `{{fluent}}` helper. If you have the following Fluent
//! file:
//!
//! ```fluent
//! foo-bar = "foo bar"
//! placeholder = this has a placeholder { $variable }
//! ```
//!
//! You can include the strings in your template with
//!
//! ```hbs
//! {{fluent "foo-bar"}} <!-- will render "foo bar" -->
//! {{fluent "placeholder" variable="baz"}} <!-- will render "this has a placeholder baz" -->
//!```
//!
//! You may also use the `{{fluentparam}}` helper to specify [variables], especially if you need
//! them to be multiline, like so:
//!
//! ```hbs
//! {{#fluent "placeholder"}}
//!     {{#fluentparam "variable"}}
//!         first line
//!         second line
//!     {{/fluentparam}}
//! {{/fluent}}
//! ```
//!
//! Multiple `{{fluentparam}}`s may be specified
//!
//! [variables]: https://projectfluent.org/fluent/guide/variables.html
//! [`static_loader!`]: ./macro.static_loader.html
#![warn(missing_docs)]

#[doc(hidden)]
pub extern crate lazy_static;

#[doc(hidden)]
pub extern crate fluent_bundle;

pub use error::LoaderError;
pub use helper::FluentHelper;
pub use loader::{ArcLoader, ArcLoaderBuilder, Loader, StaticLoader};

mod error;
mod fs;
mod helper;
pub mod loader;

/// A convenience `Result` type that defaults to `error::Loader`.
pub type Result<T, E = error::LoaderError> = std::result::Result<T, E>;
