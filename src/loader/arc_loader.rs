use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;
use std::sync::Arc;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::{langid, langids, LanguageIdentifier};

pub struct ArcLoaderBuilder<'a, 'b> {
    location: &'a Path,
    fallback: LanguageIdentifier,
    core: Option<&'b Path>,
    customize: Option<fn(&mut FluentBundle<Arc<FluentResource>>)>,
}

impl<'a, 'b> ArcLoaderBuilder<'a, 'b> {

    pub fn core<P: AsRef<Path> + ?Sized>(mut self, core: &'b P) -> Self {
        self.core = Some(core.as_ref());
        self
    }

    pub fn customize(mut self, customize: fn(&mut FluentBundle<Arc<FluentResource>>)) -> Self {
        self.customize = Some(customize);
        self
    }

    pub fn build(self) -> Result<ArcLoader, Box<dyn std::error::Error>> {
        let mut resources = HashMap::new();
        for entry in read_dir(self.location).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                if let Ok(lang) = entry.file_name().into_string() {
                    let lang_resources = super::read_from_dir(entry.path())?
                        .into_iter()
                        .map(Arc::new)
                        .collect::<Vec<_>>();
                    resources.insert(lang.parse::<LanguageIdentifier>().unwrap(), lang_resources);
                }
            }
        }

        let mut bundles = HashMap::new();
        for (lang, v) in resources.iter() {
            let mut bundle = FluentBundle::new(&[lang.clone()][..]);

            if let Some(ref core) = self.core {
                bundle.add_resource(Arc::new(super::read_from_file(core)?))
                    .expect("Failed to add core FTL resources to the bundle.");
            }

            for res in v {
                bundle
                    .add_resource(res.clone())
                    .expect("Failed to add FTL resources to the bundle.");
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
/// use fluent_template_helper::{ArcLoader, FluentHelper};
///
/// let mut handlebars = handlebars::Handlebars::new();
/// let loader = ArcLoader::new("locales/", unic_langid::langid!("en-US"))
///     .core("locales/core.ftl")
///     .customize(|bundle| bundle.set_use_isolating(false))
///     .build()
///     .unwrap();
/// handlebars.register_helper("fluent", Box::new(FluentHelper::new(loader)));
/// ```
pub struct ArcLoader {
    bundles: HashMap<LanguageIdentifier, FluentBundle<Arc<FluentResource>>>,
    fallback: LanguageIdentifier,
    fallbacks: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    resources: HashMap<LanguageIdentifier, Vec<Arc<FluentResource>>>,
}

impl super::Loader for ArcLoader {
    // Traverse the fallback chain,
    fn lookup(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
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
    pub fn new<P: AsRef<Path> + ?Sized>(location: &P, fallback: LanguageIdentifier) -> ArcLoaderBuilder {
        ArcLoaderBuilder { location: location.as_ref(), fallback, core: None, customize: None }
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
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> Option<String> {
        if let Some(bundle) = self.bundles.get(lang) {
            if let Some(message) = bundle.get_message(text_id).and_then(|m| m.value) {
                let mut errors = Vec::new();

                let value = bundle.format_pattern(&message, dbg!(args), &mut errors);

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


