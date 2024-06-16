use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::languages::negotiate_languages;
use crate::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

use crate::error::{LoaderError, LookupError};

pub use unic_langid::LanguageIdentifier;

type Customize = Option<Box<dyn FnMut(&mut FluentBundle<Arc<FluentResource>>)>>;

/// A builder pattern struct for constructing `ArcLoader`s.
pub struct ArcLoaderBuilder<'a, 'b> {
    location: &'a Path,
    fallback: LanguageIdentifier,
    shared: Option<&'b [PathBuf]>,
    customize: Customize,
}

impl<'a, 'b> ArcLoaderBuilder<'a, 'b> {
    /// Adds Fluent resources that are shared across all localizations.
    pub fn shared_resources(mut self, shared: Option<&'b [PathBuf]>) -> Self {
        self.shared = shared;
        self
    }

    /// Allows you to customise each `FluentBundle`.
    pub fn customize(
        mut self,
        customize: impl FnMut(&mut FluentBundle<Arc<FluentResource>>) + 'static,
    ) -> Self {
        self.customize = Some(Box::new(customize));
        self
    }

    /// Constructs an `ArcLoader` from the settings provided.
    pub fn build(mut self) -> Result<ArcLoader, Box<dyn std::error::Error>> {
        let mut resources = HashMap::new();

        for entry in read_dir(self.location)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(lang) = entry.file_name().into_string() {
                    let lang_resources = crate::fs::read_from_dir(entry.path())?
                        .into_iter()
                        .map(Arc::new)
                        .collect::<Vec<_>>();
                    resources.insert(lang.parse::<LanguageIdentifier>()?, lang_resources);
                }
            }
        }

        let mut bundles = HashMap::new();
        for (lang, v) in resources.iter() {
            let mut bundle = FluentBundle::new_concurrent(vec![lang.clone()]);

            for shared_resource in self.shared.unwrap_or(&[]) {
                bundle
                    .add_resource(Arc::new(crate::fs::read_from_file(shared_resource)?))
                    .map_err(|errors| LoaderError::FluentBundle { errors })?;
            }

            for res in v {
                bundle
                    .add_resource(res.clone())
                    .map_err(|errors| LoaderError::FluentBundle { errors })?;
            }

            if let Some(customize) = self.customize.as_mut() {
                (customize)(&mut bundle);
            }

            bundles.insert(lang.clone(), bundle);
        }

        let fallbacks = super::build_fallbacks(&resources.keys().cloned().collect::<Vec<_>>());

        Ok(ArcLoader {
            bundles,
            fallbacks,
            fallback: self.fallback,
        })
    }
}

/// A loader that uses `Arc<FluentResource>` as its backing storage. This is
/// mainly useful for when you need to load fluent at run time. You can
/// configure the initialisation with `ArcLoaderBuilder`.
/// ```no_run
/// use fluent_templates::ArcLoader;
///
/// let loader = ArcLoader::builder("locales/", unic_langid::langid!("en-US"))
///     .shared_resources(Some(&["locales/core.ftl".into()]))
///     .customize(|bundle| bundle.set_use_isolating(false))
///     .build()
///     .unwrap();
/// ```
pub struct ArcLoader {
    bundles: HashMap<LanguageIdentifier, FluentBundle<Arc<FluentResource>>>,
    fallback: LanguageIdentifier,
    fallbacks: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
}

impl super::Loader for ArcLoader {
    // Traverse the fallback chain,
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String {
        for lang in negotiate_languages(&[lang], &self.bundles.keys().collect::<Vec<_>>(), None) {
            if let Ok(val) = self.lookup_single_language(lang, text_id, args) {
                return val;
            }
        }
        if *lang != self.fallback {
            if let Ok(val) = self.lookup_single_language(&self.fallback, text_id, args) {
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
            if let Ok(val) = self.lookup_single_language(lang, text_id, args) {
                return Some(val);
            }
        }
        if *lang != self.fallback {
            if let Ok(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return Some(val);
            }
        }
        None
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.fallbacks.keys())
    }
}

impl ArcLoader {
    /// Creates a new `ArcLoaderBuilder`
    pub fn builder<P: AsRef<Path> + ?Sized>(
        location: &P,
        fallback: LanguageIdentifier,
    ) -> ArcLoaderBuilder {
        ArcLoaderBuilder {
            location: location.as_ref(),
            fallback,
            shared: None,
            customize: None,
        }
    }

    /// Convenience function to look up a string for a single language
    pub fn lookup_single_language<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Result<String, LookupError> {
        super::shared::lookup_single_language(&self.bundles, lang, text_id, args)
    }

    /// Convenience function to look up a string without falling back to the
    /// default fallback language
    pub fn lookup_no_default_fallback<S: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<S, FluentValue>>,
    ) -> Option<String> {
        super::shared::lookup_no_default_fallback(
            &self.bundles,
            &self.fallbacks,
            lang,
            text_id,
            args,
        )
    }

    /// Return the fallback language
    pub fn fallback(&self) -> &LanguageIdentifier {
        &self.fallback
    }
}
