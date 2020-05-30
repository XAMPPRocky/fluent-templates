use fluent_bundle::FluentValue;
use serde_json::Value as Json;
use snafu::OptionExt;
use std::collections::HashMap;

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

impl<L: Loader + Send + Sync> tera::Function for crate::FluentHelper<L> {
    fn call(&self, args: &HashMap<String, Json>) -> Result<Json, tera::Error> {
        let lang = args
            .get(LANG_KEY)
            .and_then(Json::as_str)
            .context(self::NoLangArgument)?
            .parse::<unic_langid::LanguageIdentifier>()
            .ok()
            .context(self::LangArgumentInvalid)?;
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
            fluent_args.insert(heck::KebabCase::to_kebab_case(&**key), json_to_fluent(value.clone())?);
        }

        let response = self.loader.lookup(&lang, &id, Some(&fluent_args));

        Ok(Json::String(response))
    }
}
