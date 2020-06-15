use std::collections::HashMap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

use crate::error::LoaderError;

pub use unic_langid::{langid, langids, LanguageIdentifier};

/// A builder pattern struct for constructing `ArcLoader`s.
pub struct ArcLoaderBuilder<'a, 'b> {
    location: &'a Path,
    fallback: LanguageIdentifier,
    shared: Option<&'b [PathBuf]>,
    customize: Option<fn(&mut FluentBundle<Arc<FluentResource>>)>,
}

impl<'a, 'b> ArcLoaderBuilder<'a, 'b> {
    /// Adds Fluent resources that are shared across all localizations.
    pub fn shared_resources(mut self, shared: Option<&'b [PathBuf]>) -> Self {
        self.shared = shared;
        self
    }

    /// Allows you to customise each `FluentBundle`.
    pub fn customize(mut self, customize: fn(&mut FluentBundle<Arc<FluentResource>>)) -> Self {
        self.customize = Some(customize);
        self
    }

    /// Constructs an `ArcLoader` from the settings provided.
    pub fn build(self) -> Result<ArcLoader, Box<dyn std::error::Error>> {
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
            let mut bundle = FluentBundle::new(&[lang.clone()][..]);

            for shared_resource in self.shared.as_deref().unwrap_or(&[]) {
                bundle
                    .add_resource(Arc::new(crate::fs::read_from_file(shared_resource)?))
                    .map_err(|errors| LoaderError::FluentBundle{errors})?;
            }

            for res in v {
                bundle
                    .add_resource(res.clone())
                    .map_err(|errors| LoaderError::FluentBundle{errors})?;
            }

            if let Some(customize) = self.customize {
                (customize)(&mut bundle);
            }

            bundles.insert(lang.clone(), bundle);
        }

        let fallbacks = super::build_fallbacks(&*resources.keys().cloned().collect::<Vec<_>>());

        Ok(ArcLoader {
            resources,
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
    resources: HashMap<LanguageIdentifier, Vec<Arc<FluentResource>>>,
}

impl super::Loader for ArcLoader {
    // Traverse the fallback chain,
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<String, FluentValue>>,
    ) -> String {
        for l in self.fallbacks.get(lang).expect("language not found") {
            if let Some(val) = self.lookup_single_language(l, text_id, args) {
                return val;
            }
        }
        if *lang != self.fallback {
            if let Some(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return val;
            }
        }
        format!("Unknown localization {}", text_id)
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

    /// Returns a Vec over the locales that were detected.
    pub fn locales(&self) -> Vec<LanguageIdentifier> {
        self.resources.keys().cloned().collect()
    }

    /// Convenience function to look up a string for a single language
    pub fn lookup_single_language(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<String, FluentValue>>,
    ) -> Option<String> {
        if let Some(bundle) = self.bundles.get(lang) {
            if let Some(message) = bundle.get_message(text_id).and_then(|m| m.value) {
                let mut errors = Vec::new();

                let args = super::map_to_str_map(args);
                let value = bundle.format_pattern(&message, args.as_ref(), &mut errors);

                if errors.is_empty() {
                    Some(value.into())
                } else {
                    panic!(
                        "Failed to format a message for locale {} and id {}.\nErrors\n{:?}",
                        lang, text_id, errors
                    )
                }
            } else {
                None
            }
        } else {
            panic!("Unknown language {}", lang)
        }
    }
}
