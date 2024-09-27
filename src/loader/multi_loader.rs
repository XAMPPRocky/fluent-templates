use crate::Loader;
use fluent_bundle::FluentValue;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

pub use unic_langid::LanguageIdentifier;

/// A loader comprised of other loaders.
///
/// This loader allows for loaders with multiple sources to be used
/// from a single one, instead of a multiple of them.
///
/// The idea behind this loader is to allow for the scenario where you depend
/// on crates that have their own loader (think of protocol crates which
/// are dependencies of a frontend -> the frontend needs to know each of
/// the protocol's messages and be able to display them). Using a multiloader
/// allows you to query multiple localization sources from one single source.
///
/// Note that a [`M̀ultiloader`] is most useful where each of your fluent modules
/// is specially namespaced to avoid name collisions.
///
/// # Usage
/// ```rust
/// use fluent_templates::{ArcLoader, StaticLoader, MultiLoader, Loader};
/// use unic_langid::{LanguageIdentifier, langid};
///
/// const US_ENGLISH: LanguageIdentifier = langid!("en-US");
/// const CHINESE: LanguageIdentifier = langid!("zh-CN");
///
/// fluent_templates::static_loader! {
///     static LOCALES = {
///         locales: "./tests/locales",
///         fallback_language: "en-US",
///         // Removes unicode isolating marks around arguments, you typically
///         // should only set to false when testing.
///         customise: |bundle| bundle.set_use_isolating(false),
///     };
/// }
///
/// fn main() {
///     let cn_loader = ArcLoader::builder("./tests/locales", CHINESE)
///         .customize(|bundle| bundle.set_use_isolating(false))
///         .build()
///         .unwrap();
///
///     let multiloader = MultiLoader::from_iter([
///         Box::new(&*LOCALES) as Box<dyn Loader>,
///         Box::new(cn_loader) as Box<dyn Loader>,
///     ]);
///     assert_eq!("Hello World!", multiloader.lookup(&US_ENGLISH, "hello-world"));
///     assert_eq!("儿", multiloader.lookup(&CHINESE, "exists"));
/// }
/// ```
///
/// # Order of search
/// The one that is inserted first is also the one searched first.
pub struct MultiLoader {
    pub loaders: VecDeque<Box<dyn Loader>>,
}

impl MultiLoader {
    /// Creates a [`MultiLoader`] without any loaders.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`MultiLoader`] from an iterator of loaders.
    pub fn from_iter(iter: impl IntoIterator<Item = Box<dyn Loader>>) -> Self {
        Self {
            loaders: iter.into_iter().collect(),
        }
    }
}

impl Default for MultiLoader {
    fn default() -> Self {
        Self {
            loaders: VecDeque::default(),
        }
    }
}

impl crate::Loader for MultiLoader {
    fn lookup_complete(
        &self,
        lang: &unic_langid::LanguageIdentifier,
        text_id: &str,
        args: Option<&std::collections::HashMap<Cow<'static, str>, fluent_bundle::FluentValue>>,
    ) -> String {
        for loader in self.loaders.iter() {
            if let Some(text) = loader.try_lookup_complete(lang, text_id, args) {
                return text;
            }
        }
        format!("Unknown localization {text_id}")
    }

    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        for loader in self.loaders.iter() {
            if let Some(text) = loader.try_lookup_complete(lang, text_id, args) {
                return Some(text);
            }
        }
        None
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.loaders.iter().map(|loader| loader.locales()).flatten())
    }
}
