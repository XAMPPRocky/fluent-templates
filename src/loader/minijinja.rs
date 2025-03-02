use fluent_bundle::FluentValue;
use minijinja::value::Kwargs;
use minijinja::Value;
//use serde_json::Value as Json;
use std::borrow::Cow;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

use crate::Loader;

const LANG_KEY: &str = "lang";
//const FLUENT_KEY: &str = "key";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("No `lang` argument provided.")]
    NoLangArgument,
    #[error("`lang` must be a valid unicode language identifier.")]
    LangArgumentInvalid,
    #[error("Couldn't convert minijinja::Value to Fluent value.")]
    ValueToFluentFail,
}

impl From<Error> for minijinja::Error {
    fn from(error: Error) -> Self {
        minijinja::Error::new(minijinja::ErrorKind::UndefinedError, error.to_string())
    }
}

fn value_to_fluent(value: &Value) -> crate::Result<FluentValue<'static>, minijinja::Error> {
    match value {
        v if v.is_integer() => Ok(FluentValue::from(i64::try_from(v.clone())?)),
        v if v.is_number() => Ok(FluentValue::from(f64::try_from(v.clone())?)),
        v => {
            if let Some(s) = v.as_str() {
                Ok(FluentValue::from(s.to_string()))
            } else {
                Err(Error::ValueToFluentFail)?
            }
        }
    }
}

fn parse_language(arg: &str) -> crate::Result<LanguageIdentifier, Error> {
    arg.parse::<LanguageIdentifier>()
        .ok()
        .ok_or(Error::LangArgumentInvalid)
}

impl<L: Loader + Send + Sync> crate::FluentLoader<L> {
    fn minijinja_call(&self, id: String, kwargs: Kwargs) -> Result<String, minijinja::Error> {
        let lang_arg = kwargs.get(LANG_KEY).ok().map(parse_language).transpose()?;
        let lang = lang_arg
            .as_ref()
            .or(self.default_lang.as_ref())
            .ok_or(Error::NoLangArgument)?;

        /// Filters kwargs to exclude ones used by this function and tera.
        fn is_not_tera_key(k: &&str) -> bool {
            *k != LANG_KEY
        }

        let mut fluent_args = HashMap::new();

        for key in kwargs.args().filter(is_not_tera_key) {
            let value = &kwargs.get(key)?;
            fluent_args.insert(
                Cow::from(heck::ToKebabCase::to_kebab_case(key)),
                value_to_fluent(value)?,
            );
        }

        let response = self.loader.lookup_with_args(lang, &id, &fluent_args);
        Ok(response)
    }
    pub fn into_minijinja_fn(self) -> impl Fn(String, Kwargs) -> Result<String, minijinja::Error> {
        move |a, b| self.minijinja_call(a, b)
    }
}
