use std::borrow::Borrow;
use std::collections::HashMap;

use crate::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::LanguageIdentifier;

pub fn lookup_single_language<T: AsRef<str>, R: Borrow<FluentResource>>(
    bundles: &HashMap<LanguageIdentifier, FluentBundle<R>>,
    lang: &LanguageIdentifier,
    text_id: &str,
    args: Option<&HashMap<T, FluentValue>>,
) -> Option<String> {
    let bundle = bundles.get(lang)?;
    let mut errors = Vec::new();
    let pattern = if let Some((msg, attr)) = text_id.split_once('.') {
        bundle
            .get_message(msg)?
            .attributes()
            .find(|attribute| attribute.id() == attr)?
            .value()
    } else {
        bundle.get_message(text_id)?.value()?
    };

    let args = args.map(super::map_to_fluent_args);
    let value = bundle.format_pattern(pattern, args.as_ref(), &mut errors);

    if errors.is_empty() {
        Some(value.into())
    } else {
        panic!("Failed to format a message for locale {lang} and id {text_id}.\nErrors\n{errors:?}")
    }
}

pub fn lookup_no_default_fallback<S: AsRef<str>, R: Borrow<FluentResource>>(
    bundles: &HashMap<LanguageIdentifier, FluentBundle<R>>,
    fallbacks: &HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    lang: &LanguageIdentifier,
    text_id: &str,
    args: Option<&HashMap<S, FluentValue>>,
) -> Option<String> {
    let fallbacks = fallbacks.get(lang)?;
    for l in fallbacks {
        if let Some(val) = lookup_single_language(bundles, l, text_id, args) {
            return Some(val);
        }
    }

    None
}
