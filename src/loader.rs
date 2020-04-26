//! This modules contains both the `static_loader` and `ArcLoader`
//! implementations, as well as the `Loader` trait. Which provides a loader
//! agnostic interface.

use std::collections::HashMap;
use std::fs::read_dir;

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
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> String;
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
pub fn create_bundle(
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

/// Builds a map of languages and their available resources based on `dir`.
pub fn build_resources(
    dir: impl AsRef<std::path::Path>,
) -> HashMap<LanguageIdentifier, Vec<FluentResource>> {
    let mut all_resources = HashMap::new();
    let entries = read_dir(dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            if let Ok(lang) = entry.file_name().into_string() {
                let resources = crate::fs::read_from_dir(entry.path()).unwrap();
                all_resources.insert(lang.parse().unwrap(), resources);
            }
        }
    }
    all_resources
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

/// Attempts to load a core resource and panicks if not found.
pub fn load_core_resource(path: &str) -> FluentResource {
    crate::fs::read_from_file(path).expect("cannot find core resource")
}
