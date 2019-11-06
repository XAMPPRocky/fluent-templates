use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use fluent_locale::negotiate_languages;

pub trait Loader {
    fn lookup(
        &self,
        lang: &str,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> String;
}

lazy_static! {
    static ref CORE_RESOURCE: FluentResource =
        read_from_file("./locales/core.ftl").expect("cannot find core.ftl");
    static ref RESOURCES: HashMap<String, Vec<FluentResource>> = build_resources();
    static ref BUNDLES: HashMap<String, FluentBundle<'static>> = build_bundles();
    static ref LOCALES: Vec<&'static str> = RESOURCES.iter().map(|(l, _)| &**l).collect();
    static ref FALLBACKS: HashMap<String, Vec<String>> = build_fallbacks();
}

pub fn build_fallbacks() -> HashMap<String, Vec<String>> {
    LOCALES
        .iter()
        .map(|locale| {
            (
                locale.to_string(),
                negotiate_languages(
                    &[locale],
                    &LOCALES,
                    None,
                    &fluent_locale::NegotiationStrategy::Filtering,
                )
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            )
        })
        .collect()
}

pub struct SimpleLoader {
    bundles: &'static HashMap<String, FluentBundle<'static>>,
    fallbacks: &'static HashMap<String, Vec<String>>,
}

impl SimpleLoader {
    pub fn new() -> Self {
        Self {
            bundles: &*BUNDLES,
            fallbacks: &*FALLBACKS,
        }
    }

    pub fn lookup_single_language(
        &self,
        lang: &str,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> Option<String> {
        if let Some(bundle) = self.bundles.get(lang) {
            if bundle.has_message(text_id) {
                let (value, _errors) = bundle.format(text_id, args).unwrap_or_else(|| {
                    panic!(
                        "Failed to format a message for locale {} and id {}",
                        lang, text_id
                    )
                });
                Some(value)
            } else {
                None
            }
        } else {
            panic!("Unknown language {}", lang)
        }
    }

    // Don't fall back to English
    pub fn lookup_no_english(
        &self,
        lang: &str,
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

impl Loader for SimpleLoader {
    // Traverse the fallback chain,
    fn lookup(
        &self,
        lang: &str,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> String {
        for l in self.fallbacks.get(lang).expect("language not found") {
            if let Some(val) = self.lookup_single_language(l, text_id, args) {
                return val;
            }
        }
        if lang != "en-US" {
            if let Some(val) = self.lookup_single_language("en-US", text_id, args) {
                return val;
            }
        }
        format!("Unknown localization {}", text_id)
    }
}

pub fn read_from_file<P: AsRef<Path>>(filename: P) -> io::Result<FluentResource> {
    let mut file = File::open(filename)?;
    let mut string = String::new();

    file.read_to_string(&mut string)?;

    Ok(FluentResource::try_new(string).expect("File did not parse!"))
}

pub fn read_from_dir<P: AsRef<Path>>(dirname: P) -> io::Result<Vec<FluentResource>> {
    let mut result = Vec::new();
    for dir_entry in read_dir(dirname)? {
        let entry = dir_entry?;
        let resource = read_from_file(entry.path())?;
        result.push(resource);
    }
    Ok(result)
}

pub fn create_bundle(lang: &str, resources: &'static Vec<FluentResource>) -> FluentBundle<'static> {
    let mut bundle = FluentBundle::new(&[lang]);
    bundle
        .add_resource(&CORE_RESOURCE)
        .expect("Failed to add core resource to bundle");
    for res in resources {
        bundle
            .add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");
    }

    bundle
        .add_function("EMAIL", |values, _named| {
            let email = match *values.get(0)?.as_ref()? {
                FluentValue::String(ref s) => s,
                _ => return None,
            };
            Some(FluentValue::String(format!(
                "<a href='mailto:{0}' lang='en-US'>{0}</a>",
                email
            )))
        })
        .expect("could not add function");

    bundle
        .add_function("ENGLISH", |values, _named| {
            let text = match *values.get(0)?.as_ref()? {
                FluentValue::String(ref s) => s,
                _ => return None,
            };
            Some(FluentValue::String(format!(
                "<span lang='en-US'>{0}</span>",
                text
            )))
        })
        .expect("could not add function");

    bundle
}

fn build_resources() -> HashMap<String, Vec<FluentResource>> {
    let mut all_resources = HashMap::new();
    let entries = read_dir("./locales").unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            if let Ok(lang) = entry.file_name().into_string() {
                let resources = read_from_dir(entry.path()).unwrap();
                all_resources.insert(lang, resources);
            }
        }
    }
    all_resources
}

fn build_bundles() -> HashMap<String, FluentBundle<'static>> {
    let mut bundles = HashMap::new();
    for (ref k, ref v) in &*RESOURCES {
        bundles.insert(k.to_string(), create_bundle(&k, &v));
    }
    bundles
}
