use std::borrow::Borrow;
use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::{langid, langids, LanguageIdentifier};

pub fn lookup_single_language<T: AsRef<str>, R: Borrow<FluentResource>>(
    bundles: &HashMap<LanguageIdentifier, FluentBundle<R>>,
    lang: &LanguageIdentifier,
    text_id: &str,
    args: Option<&HashMap<T, FluentValue>>,
) -> Option<String> {
    let bundle = bundles.get(lang)?;
    let mut errors = Vec::new();
    let pattern = if text_id.contains('.') {
        // TODO: #![feature(str_split_once)]
        let ids: Vec<_> = text_id.splitn(2, '.').collect();
        bundle
            .get_message(ids[0])?
            .attributes
            .iter()
            .find(|attribute| attribute.id == ids[1])?
            .value
    } else {
        bundle.get_message(text_id)?.value?
    };

    let args = super::map_to_fluent_args(args);
    let value = bundle.format_pattern(&pattern, args.as_ref(), &mut errors);

    if errors.is_empty() {
        Some(value.into())
    } else {
        panic!(
            "Failed to format a message for locale {} and id {}.\nErrors\n{:?}",
            lang, text_id, errors
        )
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
