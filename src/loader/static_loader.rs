use std::collections::HashMap;

use crate::{languages::negotiate_languages, FluentBundle};
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::LanguageIdentifier;

/// A simple Loader implementation, with statically-loaded fluent data.
/// Typically created with the [`static_loader!`] macro
///
/// [`static_loader!`]: ./macro.static_loader.html
pub struct StaticLoader {
    bundles: &'static HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>>,
    fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    fallback: LanguageIdentifier,
}

impl StaticLoader {
    /// Construct a new `StaticLoader`.
    ///
    /// This is exposed as publicly so that it can be used inside the
    /// `static_loader!` macro. it's not meant to be called directly.
    #[doc(hidden)]
    pub fn new(
        bundles: &'static HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>>,
        fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
        fallback: LanguageIdentifier,
    ) -> Self {
        Self {
            bundles,
            fallbacks,
            fallback,
        }
    }

    /// Convenience function to look up a string for a single language
    pub fn lookup_single_language<S: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<S, FluentValue>>,
    ) -> Option<String> {
        super::shared::lookup_single_language(self.bundles, lang, text_id, args)
    }

    /// Convenience function to look up a string without falling back to the
    /// default fallback language
    pub fn lookup_no_default_fallback<S: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<S, FluentValue>>,
    ) -> Option<String> {
        super::shared::lookup_no_default_fallback(self.bundles, self.fallbacks, lang, text_id, args)
    }
}

impl super::Loader for StaticLoader {
    // Traverse the fallback chain,
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String {
        for lang in negotiate_languages(&[lang], &self.bundles.keys().collect::<Vec<_>>(), None) {
            if let Some(val) = self.lookup_single_language(lang, text_id, args) {
                return val;
            }
        }

        if *lang != self.fallback {
            if let Some(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return val;
            }
        }
        format!("Unknown localization {text_id}")
    }

    // Traverse the fallback chain,
    fn try_lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Option<String> {
        for lang in negotiate_languages(&[lang], &self.bundles.keys().collect::<Vec<_>>(), None) {
            if let Some(val) = self.lookup_single_language(lang, text_id, args) {
                return Some(val);
            }
        }

        if *lang != self.fallback {
            if let Some(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return Some(val);
            }
        }
        None
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.fallbacks.keys())
    }
}
