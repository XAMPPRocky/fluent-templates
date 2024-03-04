//! This modules contains both the `static_loader` and `ArcLoader`
//! implementations, as well as the `Loader` trait. Which provides a loader
//! agnostic interface.

#[cfg(feature = "handlebars")]
mod handlebars;

#[cfg(feature = "tera")]
mod tera;

mod shared;

use std::collections::HashMap;

use crate::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use fluent_langneg::negotiate_languages;

pub use unic_langid::{langid, langids, LanguageIdentifier};

mod arc_loader;
mod static_loader;

pub use arc_loader::{ArcLoader, ArcLoaderBuilder};
pub use static_loader::StaticLoader;

/// A loader capable of looking up Fluent keys given a language.
pub trait Loader {
    /// Look up `text_id` for `lang` in Fluent.
    fn lookup(&self, lang: &LanguageIdentifier, text_id: &str) -> String {
        self.lookup_complete::<&str>(lang, text_id, None)
    }

    /// Look up `text_id` for `lang` with `args` in Fluent.
    fn lookup_with_args<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: &HashMap<T, FluentValue>,
    ) -> String {
        self.lookup_complete(lang, text_id, Some(args))
    }

    /// Look up `text_id` for `lang` in Fluent, using any `args` if provided.
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String;

    /// Look up `text_id` for `lang` in Fluent.
    fn try_lookup(&self, lang: &LanguageIdentifier, text_id: &str) -> Option<String> {
        self.try_lookup_complete::<&str>(lang, text_id, None)
    }

    /// Look up `text_id` for `lang` with `args` in Fluent.
    fn try_lookup_with_args<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: &HashMap<T, FluentValue>,
    ) -> Option<String> {
        self.try_lookup_complete(lang, text_id, Some(args))
    }

    /// Look up `text_id` for `lang` in Fluent, using any `args` if provided.
    fn try_lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Option<String>;

    /// Returns an Iterator over the locales that are present.
    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_>;
}

impl<L> Loader for std::sync::Arc<L>
where
    L: Loader,
{
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String {
        L::lookup_complete(self, lang, text_id, args)
    }

    fn try_lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Option<String> {
        L::try_lookup_complete(self, lang, text_id, args)
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        L::locales(self)
    }
}

impl<'a, L> Loader for &'a L
where
    L: Loader,
{
    fn lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> String {
        L::lookup_complete(self, lang, text_id, args)
    }

    fn try_lookup_complete<T: AsRef<str>>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<T, FluentValue>>,
    ) -> Option<String> {
        L::try_lookup_complete(self, lang, text_id, args)
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        L::locales(self)
    }
}

/// A `Loader` agnostic container type with optional trait implementations
/// for integrating with different libraries.
pub struct FluentLoader<L> {
    loader: L,
    #[allow(unused)]
    default_lang: Option<LanguageIdentifier>,
}

impl<L> FluentLoader<L> {
    /// Create a new `FluentLoader`.
    pub fn new(loader: L) -> Self {
        Self {
            loader,
            default_lang: None,
        }
    }

    /// Set default language for this `FluentLoader`.
    /// Template engines can use this value when rendering translations.
    /// So far this feature is only implemented for Tera.
    pub fn with_default_lang(self, lang: LanguageIdentifier) -> Self {
        Self {
            loader: self.loader,
            default_lang: Some(lang),
        }
    }
}

/// Constructs a map of languages with a list of potential fallback languages.
pub fn build_fallbacks(
    locales: &[LanguageIdentifier],
) -> HashMap<LanguageIdentifier, Vec<LanguageIdentifier>> {
    let mut map = HashMap::new();

    for locale in locales.iter() {
        map.insert(
            locale.to_owned(),
            negotiate_languages(
                &[locale],
                locales,
                None,
                fluent_langneg::NegotiationStrategy::Filtering,
            )
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        );
    }

    map
}

/// Creates a new static `FluentBundle` for `lang` using `resources`. Optionally
/// shared resources can be specified with `core_resource` and the bundle can
/// be customized with `customizer`.
fn create_bundle(
    lang: LanguageIdentifier,
    resources: &'static [FluentResource],
    core_resource: Option<&'static FluentResource>,
    customizer: &impl Fn(&mut FluentBundle<&'static FluentResource>),
) -> FluentBundle<&'static FluentResource> {
    let mut bundle: FluentBundle<&'static FluentResource> =
        FluentBundle::new_concurrent(vec![lang]);
    if let Some(core) = core_resource {
        bundle
            .add_resource(core)
            .expect("Failed to add core resource to bundle");
    }
    for res in resources {
        bundle
            .add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");
    }

    customizer(&mut bundle);
    bundle
}

/// Maps from map of languages containing a list of resources to a map of
/// languages containing a `FluentBundle` of those resources.
pub fn build_bundles(
    resources: &'static HashMap<LanguageIdentifier, Vec<FluentResource>>,
    core_resource: Option<&'static FluentResource>,
    customizer: impl Fn(&mut FluentBundle<&'static FluentResource>),
) -> HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>> {
    let mut bundles = HashMap::new();
    for (k, v) in resources.iter() {
        bundles.insert(
            k.clone(),
            create_bundle(k.clone(), v, core_resource, &customizer),
        );
    }
    bundles
}

fn map_to_fluent_args<'map, T: AsRef<str>>(
    map: Option<&'map HashMap<T, FluentValue>>,
) -> Option<FluentArgs<'map>> {
    let mut new = FluentArgs::new();

    if let Some(map) = map {
        for (key, value) in map {
            new.set(key.as_ref(), value.clone());
        }
    }

    Some(new)
}
