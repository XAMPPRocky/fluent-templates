//! # Fluent Templates: Templating for Fluent
//!
//! `fluent-templates` lets you to easily integrate Fluent localisation into
//! your Rust application or library. It does this by providing a high level
//! "loader" API that loads fluent strings based on simple language negotiation,
//! and the `FluentLoader` struct which is a `Loader` agnostic container type
//! that comes with optional trait implementations for popular templating
//! engines such as handlebars or tera that allow you to be able to use your
//! localisations in your templates with no boilerplate.
//!
//! ## Loaders
//! Currently this crate provides two different kinds of loaders that cover two
//! main use cases.
//!
//! - [`static_loader!`] — A procedural macro that loads your fluent resources
//!   at *compile-time* into your binary and creates a new [`StaticLoader`]
//!   static variable that allows you to access the localisations.
//!   `static_loader!` is most useful when you want to localise your
//!   application and want to ship your fluent resources with your binary.
//!
//! - [`ArcLoader`] — A struct that loads your fluent resources at *run-time*
//!   using `Arc` as its backing storage. `ArcLoader` is most useful for when
//!   you want to be able to change and/or update localisations at run-time, or
//!   if you're writing a developer tool that wants to provide fluent
//!   localisation in your own application such as a static site generator.
//!
//!
//! ## `static_loader!`
//! The easiest way to use `fluent-templates` is to use the [`static_loader!`]
//! procedural macro that will create a new [`StaticLoader`] static variable
//! behind a `Arc` pointer.
//!
//! ```
//! fluent_templates::static_loader! {
//!     // Declare our `StaticLoader` named `LOCALES`.
//!     static LOCALES = {
//!         // The directory of localisations and fluent resources.
//!         locales: "./tests/locales",
//!         // The language to falback on if something is not present.
//!         fallback_language: "en-US",
//!         // Optional: A shared fluent resource
//!         core_locales: "./tests/locales/core.ftl",
//!         // Optional: A function that is run over each fluent bundle.
//!         customise: |bundle| bundle.set_use_isolating(false),
//!     };
//! }
//! # fn main() {}
//! ```
//!
//! ### Looking up fluent resources
//!
//! #### Hello World
//! ```fluent
//!  # In `locales/en-US/main.ftl`
//!  hello = Hello World!
//!  # In `locales/fr/main.ftl`
//!  hello = Bonjour le monde!
//!  # In `locales/de/main.ftl`
//!  hello = Hallo Welt!
//! ```
//!
//! ```no_run
//! use unic_langid::langid;
//! use fluent_templates::{Loader, static_loader};
//!
//! static_loader! {
//!     static LOCALES = {
//!         locales: "./tests/locales",
//!         fallback_language: "en-US",
//!     };
//! }
//!
//! fn main() {
//!     println!("{}", LOCALES.lookup(&langid!("en-US"), "hello", None));
//! }
//! ```
//!
//! ### Tera
//! ```rust
//! fluent_templates::static_loader! {
//!     static LOCALES = {
//!         locales: "./tests/locales",
//!         fallback_language: "en-US",
//!     };
//! }
//!
//!
//! # #[cfg(feature = "tera")] const _: () = {
//! use tera::Tera;
//!
//! fn init(tera: &mut Tera) {
//!     let helper = fluent_templates::FluentLoader::new(LOCALES.clone());
//!     tera.register_function("fluent", helper);
//! }
//!
//! fn render_page(tera: &mut Tera, ctx: &tera::Context) -> String {
//!     tera.render_str(r#"{{ fluent(key="foo-bar", lang="en") }} baz"#, ctx).unwrap()
//! }
//! # };
//! # fn main() { }
//! ```
//!
//! ### Handlebars
//! ```rust
//! fluent_templates::static_loader! {
//!     static LOCALES = {
//!         locales: "./tests/locales",
//!         fallback_language: "en-US",
//!     };
//! }
//!
//! # #[cfg(feature = "handlebars")] const _: () = {
//! use handlebars::Handlebars;
//!
//! fn init(handlebars: &mut Handlebars) {
//!     let helper = fluent_templates::FluentLoader::new(LOCALES.clone());
//!     handlebars.register_helper("fluent", Box::new(helper));
//! }
//!
//! fn render_page(handlebars: &Handlebars) -> String {
//!     let data = serde_json::json!({"lang": "zh-CN"});
//!     handlebars.render_template("{{fluent \"foo-bar\"}} baz", &data).unwrap()
//! }
//! # };
//! # fn main() { }
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
//! [`StaticLoader`]: ./struct.StaticLoader.html
//! [`ArcLoader`]: ./struct.ArcLoader.html
#![warn(missing_docs)]

#[doc(hidden)]
pub extern crate lazy_static;

#[doc(hidden)]
pub extern crate fluent_bundle;

pub use error::LoaderError;
pub use loader::{ArcLoader, ArcLoaderBuilder, FluentLoader, Loader, StaticLoader};

mod error;
#[doc(hidden)]
pub mod fs;
#[doc(hidden)]
pub mod loader;

#[cfg(feature = "macros")]
pub use fluent_template_macros::static_loader;

#[doc(hidden)]
pub use arc_swap;
#[doc(hidden)]
pub use once_cell;

/// A convenience `Result` type that defaults to `error::Loader`.
pub type Result<T, E = error::LoaderError> = std::result::Result<T, E>;
