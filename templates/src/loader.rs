//! This modules contains both the `static_loader` and `ArcLoader`
//! implementations, as well as the `Loader` trait. Which provides a loader
//! agnostic interface.

use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};
use fluent_langneg::negotiate_languages;

pub use unic_langid::{langid, langids, LanguageIdentifier};

mod arc_loader;
mod static_loader;

pub use arc_loader::{ArcLoader, ArcLoaderBuilder};
pub use static_loader::StaticLoader;

/// A loader capable of looking up Fluent keys given a language.
pub trait Loader {
    /// Look up `text_id` for `lang` in Fluent and, provides any `args` if present.
    fn lookup(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<String, FluentValue>>,
    ) -> String;
}

impl<L> Loader for std::sync::Arc<L>
where
    L: Loader,
{
    fn lookup(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<String, FluentValue>>,
    ) -> String {
        L::lookup(self, lang, text_id, args)
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
    let mut bundle: FluentBundle<&'static FluentResource> = FluentBundle::new(&[lang]);
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
    for (k, ref v) in resources.iter() {
        bundles.insert(
            k.clone(),
            create_bundle(k.clone(), &v, core_resource, &customizer),
        );
    }
    bundles
}

fn map_to_str_map<'a>(
    map: Option<&'a HashMap<String, FluentValue>>,
) -> Option<HashMap<&'a str, FluentValue<'a>>> {
    let mut new = HashMap::with_capacity(map.map(HashMap::len).unwrap_or(0));

    if let Some(map) = map {
        for (key, value) in map.iter() {
            new.insert(&**key, value.clone());
        }
    }

    Some(new)
}
