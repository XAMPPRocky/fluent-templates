use fluent_bundle::FluentValue;
use serde_json::Value as Json;
use snafu::OptionExt;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

use crate::Loader;

const LANG_KEY: &str = "lang";
const FLUENT_KEY: &str = "key";

#[derive(Debug, snafu::Snafu)]
enum Error {
    #[snafu(display("No `lang` argument provided."))]
    NoLangArgument,
    #[snafu(display("`lang` must be a valid unicode language identifier."))]
    LangArgumentInvalid,
    #[snafu(display("No `id` argument provided."))]
    NoFluentArgument,
    #[snafu(display("Couldn't convert JSON to Fluent value."))]
    JsonToFluentFail,
}

impl From<Error> for tera::Error {
    fn from(error: Error) -> Self {
        tera::Error::msg(error)
    }
}

fn json_to_fluent(json: Json) -> crate::Result<FluentValue<'static>, Error> {
    match json {
        Json::Number(n) if n.is_u64() => Ok(FluentValue::from(n.as_u64().unwrap())),
        Json::Number(n) if n.is_f64() => Ok(FluentValue::from(n.as_f64().unwrap())),
        Json::String(s) => Ok(FluentValue::String(s.into())),
        _ => Err(Error::JsonToFluentFail),
    }
}

fn parse_language(arg: &Json) -> crate::Result<LanguageIdentifier, Error> {
    arg.as_str()
        .context(self::LangArgumentInvalid)?
        .parse::<LanguageIdentifier>()
        .ok()
        .context(self::LangArgumentInvalid)
}

impl<L: Loader + Send + Sync> tera::Function for crate::FluentLoader<L> {
    fn call(&self, args: &HashMap<String, Json>) -> Result<Json, tera::Error> {
        let lang_arg = args.get(LANG_KEY).map(parse_language).transpose()?;
        let lang = lang_arg
            .as_ref()
            .or(self.default_lang.as_ref())
            .context(self::NoLangArgument)?;

        let id = args
            .get(FLUENT_KEY)
            .and_then(Json::as_str)
            .context(self::NoFluentArgument)?;

        /// Filters kwargs to exclude ones used by this function and tera.
        fn is_not_tera_key((k, _): &(&String, &Json)) -> bool {
            let k = &**k;
            !(k == LANG_KEY || k == FLUENT_KEY || k == "__tera_one_off")
        }

        let mut fluent_args = HashMap::new();

        for (key, value) in args.iter().filter(is_not_tera_key) {
            fluent_args.insert(
                heck::KebabCase::to_kebab_case(&**key),
                json_to_fluent(value.clone())?,
            );
        }

        let response = self.loader.lookup_with_args(lang, &id, &fluent_args);

        Ok(Json::String(response))
    }
}
