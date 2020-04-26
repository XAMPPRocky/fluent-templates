//! [Fluent](https://projectfluent.org/) helper for [Handlebars](https://docs.rs/handlebars).
//!
//! This crate provides a Handlebars helper that can load Fluent strings.
//!
//!
//! # Setting up the fluent helper with handlebars
//!
//! The easiest way to use this is to use the [`simple_loader!()`] macro:
//!
//! ```rust
//! use fluent_template_helper::*;
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
//! ```
//!
//! You should have a `locales/` folder somewhere with one folder per language code,
//! containing all of your FTL files. See the [`simple_loader!()`] macro for more options.
//!
//! Make sure the [`handlebars::Context`] has a toplevel "lang" field when rendering.
//!
//!
//! # Using the fluent helper in your templates
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
//! [`simple_loader!()`]: ./macro.simple_loader.html

#[doc(hidden)]
pub extern crate lazy_static;

#[doc(hidden)]
pub extern crate fluent_bundle;

pub use helper::FluentHelper;
pub use loader::{ArcLoader, ArcLoaderBuilder, Loader, StaticLoader};

mod fs;
mod error;
mod helper;
pub mod loader;

pub type Result<T, E = error::LoaderError> = std::result::Result<T, E>;

