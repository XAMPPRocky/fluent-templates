use std::borrow::Borrow;
use std::collections::HashMap;

use crate::{error::LookupError, FluentBundle};
use fluent_bundle::{FluentResource, FluentValue};

pub use unic_langid::LanguageIdentifier;

pub fn lookup_single_language<T: AsRef<str>, R: Borrow<FluentResource>>(
    bundles: &HashMap<LanguageIdentifier, FluentBundle<R>>,
    lang: &LanguageIdentifier,
    text_id: &str,
    args: Option<&HashMap<T, FluentValue>>,
) -> Result<String, LookupError> {
    let bundle = bundles
        .get(lang)
        .ok_or_else(|| LookupError::LangNotLoaded(lang.clone()))?;

    let mut errors = Vec::new();
    let message_retrieve_error = || LookupError::MessageRetrieval(text_id.to_owned());

    let pattern = if let Some((msg, attr)) = text_id.split_once('.') {
        bundle
            .get_message(msg)
            .ok_or_else(message_retrieve_error)?
            .attributes()
            .find(|attribute| attribute.id() == attr)
            .ok_or_else(|| LookupError::AttributeNotFound {
                message_id: msg.to_owned(),
                attribute: attr.to_owned(),
            })?
            .value()
    } else {
        bundle
            .get_message(text_id)
            .ok_or_else(message_retrieve_error)?
            .value()
            .ok_or_else(message_retrieve_error)?
    };

    let args = args.map(super::map_to_fluent_args);
    let value = bundle.format_pattern(pattern, args.as_ref(), &mut errors);

    if errors.is_empty() {
        Ok(value.into())
    } else {
        Err(LookupError::FluentError(errors))
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
        if let Ok(val) = lookup_single_language(bundles, l, text_id, args) {
            return Some(val);
        }
    }

    None
}
