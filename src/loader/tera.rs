use fluent_bundle::FluentValue;
use std::borrow::Cow;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

use crate::Loader;

const LANG_KEY: &str = "lang";
const FLUENT_KEY: &str = "key";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("No `lang` argument provided.")]
    NoLangArgument,
    #[error("`lang` must be a valid unicode language identifier.")]
    LangArgumentInvalid,
    #[error("No `id` argument provided.")]
    NoFluentArgument,
    #[error("Couldn't convert JSON to Fluent value.")]
    JsonToFluentFail,
}

impl From<Error> for tera::Error {
    fn from(error: Error) -> Self {
        tera::Error::message(error)
    }
}

fn tera_to_fluent(json: tera::Value) -> crate::Result<FluentValue<'static>, Error> {
    if let Some(n) = json.as_f64() {
        return Ok(FluentValue::from(n));
    }
    if let Some(n) = json.as_u64() {
        return Ok(FluentValue::from(n));
    }
    if let Some(s) = json.as_str() {
        return Ok(FluentValue::from(s.to_string()));
    }
    return Err(Error::JsonToFluentFail);
}

fn parse_language(arg: &str) -> crate::Result<LanguageIdentifier, Error> {
    arg.parse::<LanguageIdentifier>()
        .ok()
        .ok_or(Error::LangArgumentInvalid)
}

impl<L: Loader + Send + Sync + 'static> tera::Function<Result<tera::Value, tera::Error>>
    for crate::FluentLoader<L>
{
    fn call(&self, kwargs: tera::Kwargs, _: &tera::State) -> Result<tera::Value, tera::Error> {
        let lang_arg: Option<LanguageIdentifier> = kwargs
            .get::<&str>(LANG_KEY)
            .map_err(|_| Error::LangArgumentInvalid)?
            .map(parse_language)
            .transpose()?;
        let lang = lang_arg
            .as_ref()
            .or(self.default_lang.as_ref())
            .ok_or(Error::NoLangArgument)?;

        let id = kwargs
            .must_get::<&str>(FLUENT_KEY)
            .map_err(|_| Error::NoFluentArgument)?;

        /// Filters kwargs to exclude ones used by this function and tera.
        fn is_not_tera_key((k, _): &(&tera::value::Key<'_>, &tera::Value)) -> bool {
            match k.as_str() {
                Some(LANG_KEY) | Some(FLUENT_KEY) | Some("__tera_one_off") => false,
                _ => true,
            }
        }

        let mut fluent_args = HashMap::new();

        for (key, value) in kwargs.iter().filter(is_not_tera_key) {
            if let Some(key) = key.as_str() {
                fluent_args.insert(
                    Cow::from(heck::ToKebabCase::to_kebab_case(key)),
                    tera_to_fluent(value.clone())?,
                );
            }
        }

        let response = self.loader.lookup_with_args(lang, id, &fluent_args);
        Ok(tera::Value::from(response))
    }
}
