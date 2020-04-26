use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::{langid, langids, LanguageIdentifier};

/// Loads Fluent data at runtime via `lazy_static` to produce a loader.
///
/// Usage:
///
/// ```rust
/// use fluent_templates::*;
///
/// static_loader!(create_loader, "./tests/locales/", "en-US");
///
/// fn init() {
///     let loader = create_loader();
///     let helper = FluentHelper::new(loader);
/// }
/// ```
///
/// `$constructor` is the name of the constructor function for the loader, `$location` is
/// the location of a folder containing individual locale folders, `$fallback` is the language to use
/// for fallback strings.
///
/// Some Fluent users have a share "core.ftl" file that contains strings used by all locales,
/// for example branding information. They also may want to define custom functions on the bundle.
///
/// This can be done with an extended invocation:
///
/// ```rust
/// use fluent_templates::*;
///
/// static_loader!(create_loader, "./tests/locales/", "en-US", core: "./tests/core.ftl",
///                customizer: |bundle| {bundle.add_function("FOOBAR", |_values, _named| {unimplemented!()}); });
///
/// fn init() {
///     let loader = create_loader();
///     let helper = FluentHelper::new(loader);
/// }
/// ```
///
/// The constructor function is cheap to call multiple times since all the heavy duty stuff is stored in shared statics.
///
#[macro_export]
macro_rules! static_loader {
    ($constructor:ident, $location:expr, $fallback:expr) => {
        $crate::lazy_static::lazy_static! {
            static ref RESOURCES: std::collections::HashMap<$crate::loader::LanguageIdentifier, Vec<$crate::fluent_bundle::FluentResource>> = $crate::loader::build_resources($location);
            static ref BUNDLES: std::collections::HashMap<$crate::loader::LanguageIdentifier, $crate::fluent_bundle::concurrent::FluentBundle<&'static $crate::fluent_bundle::FluentResource>> = $crate::loader::build_bundles(&&RESOURCES, None, |_bundle| {});
            static ref LOCALES: Vec<$crate::loader::LanguageIdentifier> = RESOURCES.keys().cloned().collect();
            static ref FALLBACKS: std::collections::HashMap<$crate::loader::LanguageIdentifier, Vec<$crate::loader::LanguageIdentifier>> = $crate::loader::build_fallbacks(&*LOCALES);
        }

        pub fn $constructor() -> $crate::loader::StaticLoader {
            $crate::loader::StaticLoader::new(&*BUNDLES, &*FALLBACKS, $fallback.parse().expect("fallback language not valid"))
        }
    };
    ($constructor:ident, $location:expr, $fallback:expr, core: $core:expr, customizer: $custom:expr) => {
        $crate::lazy_static::lazy_static! {
            static ref CORE_RESOURCE: $crate::fluent_bundle::FluentResource = $crate::loader::load_core_resource($core);
            static ref RESOURCES: std::collections::HashMap<$crate::loader::LanguageIdentifier, Vec<$crate::fluent_bundle::FluentResource>> = $crate::loader::build_resources($location);
            static ref BUNDLES: std::collections::HashMap<$crate::loader::LanguageIdentifier, $crate::fluent_bundle::concurrent::FluentBundle<&'static $crate::fluent_bundle::FluentResource>> = $crate::loader::build_bundles(&*RESOURCES, Some(&CORE_RESOURCE), $custom);
            static ref LOCALES: Vec<$crate::loader::LanguageIdentifier> = RESOURCES.keys().cloned().collect();
            static ref FALLBACKS: std::collections::HashMap<$crate::loader::LanguageIdentifier, Vec<$crate::loader::LanguageIdentifier>> = $crate::loader::build_fallbacks(&*LOCALES);
        }

        pub fn $constructor() -> $crate::loader::StaticLoader {
            $crate::loader::StaticLoader::new(&*BUNDLES, &*FALLBACKS, $fallback.parse().expect("fallback language not valid"))
        }
    };
}

/// A simple Loader implementation, with statically-loaded fluent data.
/// Typically created with the [`static_loader!()`] macro
pub struct StaticLoader {
    bundles: &'static HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>>,
    fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    fallback: LanguageIdentifier,
}

impl StaticLoader {
    /// Construct a StaticLoader
    ///
    /// You should probably be using the constructor from `static_loader!()`
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

    /// Convenience function to look up a string without falling back to the default fallback language
    pub fn lookup_no_default_fallback(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> Option<String> {
        for l in self.fallbacks.get(lang).expect("language not found") {
            if let Some(val) = self.lookup_single_language(l, text_id, args) {
                return Some(val);
            }
        }

        None
    }
}

impl super::Loader for StaticLoader {
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
