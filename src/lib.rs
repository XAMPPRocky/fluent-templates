#![doc = include_str!("../README.md")]

#![warn(missing_docs)]

#[doc(hidden)]
pub extern crate fluent_bundle;

#[doc(hidden)]
pub type FluentBundle<R> =
    fluent_bundle::bundle::FluentBundle<R, intl_memoizer::concurrent::IntlLangMemoizer>;

pub use error::LoaderError;
pub use loader::{ArcLoader, ArcLoaderBuilder, FluentLoader, Loader, MultiLoader, StaticLoader};

mod error;
#[doc(hidden)]
pub mod fs;
mod languages;
#[doc(hidden)]
pub mod loader;

#[cfg(feature = "macros")]
pub use fluent_template_macros::static_loader;
#[cfg(feature = "macros")]
pub use unic_langid::langid;
pub use unic_langid::LanguageIdentifier;

/// A convenience `Result` type that defaults to `error::Loader`.
pub type Result<T, E = error::LoaderError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Loader;
    use unic_langid::{langid, LanguageIdentifier};

    #[test]
    fn check_if_loader_is_object_safe() {
        const US_ENGLISH: LanguageIdentifier = langid!("en-US");

        let loader = ArcLoader::builder("./tests/locales", US_ENGLISH)
            .customize(|bundle| bundle.set_use_isolating(false))
            .build()
            .unwrap();

        let loader: Box<dyn Loader> = Box::new(loader);
        assert_eq!("Hello World!", loader.lookup(&US_ENGLISH, "hello-world"));
    }
}
