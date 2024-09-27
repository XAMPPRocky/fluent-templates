use crate::Loader;
use fluent_bundle::FluentValue;
use std::borrow::Cow;
use std::collections::HashMap;

pub use unic_langid::LanguageIdentifier;

/// A loader comprised of other loaders.
///
/// This loader allows for loaders with multiple sources to be used
/// from a single one, instead of a multiple of them.
pub struct MultiLoader {
    pub loaders: Vec<Box<dyn Loader>>,
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
            loaders: Vec::default(),
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
