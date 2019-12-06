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

#[macro_export]
macro_rules! simple_loader {
    ($constructor:ident, $location:expr, $fallback:expr) => {
        use $crate::loader::{build_resources, build_bundles, build_fallbacks, SimpleLoader, load_core_resource};
        use std::collections::HashMap;
        use $crate::fluent_bundle::{FluentBundle, FluentResource, FluentValue};
        $crate::lazy_static::lazy_static! {
            static ref RESOURCES: HashMap<String, Vec<FluentResource>> = build_resources($location);
            static ref BUNDLES: HashMap<String, FluentBundle<'static>> = build_bundles(&&RESOURCES, None, |_bundle| {});
            static ref LOCALES: Vec<&'static str> = RESOURCES.iter().map(|(l, _)| &**l).collect();
            static ref FALLBACKS: HashMap<String, Vec<String>> = build_fallbacks(&LOCALES);
        }

        pub fn $constructor() -> SimpleLoader {
            SimpleLoader::new(&*BUNDLES, &*FALLBACKS, $fallback.into())
        }
    };
    ($constructor:ident, $location:expr, $fallback:expr, core: $core:expr, customizer: $custom:expr) => {
        use $crate::loader::{build_resources, build_bundles, build_fallbacks, SimpleLoader, load_core_resource};
        use std::collections::HashMap;
        use $crate::fluent_bundle::{FluentBundle, FluentResource, FluentValue};
        $crate::lazy_static::lazy_static! {
            static ref CORE_RESOURCE: FluentResource = load_core_resource($core);
            static ref RESOURCES: HashMap<String, Vec<FluentResource>> = build_resources($location);
            static ref BUNDLES: HashMap<String, FluentBundle<'static>> = build_bundles(&*RESOURCES, Some(&CORE_RESOURCE), $custom);
            static ref LOCALES: Vec<&'static str> = RESOURCES.iter().map(|(l, _)| &**l).collect();
            static ref FALLBACKS: HashMap<String, Vec<String>> = build_fallbacks(&LOCALES);
        }

        pub fn $constructor() -> SimpleLoader {
            SimpleLoader::new(&*BUNDLES, &*FALLBACKS, $fallback.into())
        }
    };
}

pub fn build_fallbacks(locales: &[&str]) -> HashMap<String, Vec<String>> {
    locales
        .iter()
        .map(|locale| {
            (
                locale.to_string(),
                negotiate_languages(
                    &[locale],
                    &locales,
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
    fallback: String,
}

impl SimpleLoader {
    pub fn new(
        bundles: &'static HashMap<String, FluentBundle<'static>>,
        fallbacks: &'static HashMap<String, Vec<String>>,
        fallback: String,
    ) -> Self {
        Self {
            bundles,
            fallbacks,
            fallback,
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
        if lang != self.fallback {
            if let Some(val) = self.lookup_single_language(&self.fallback, text_id, args) {
                return val;
            }
        }
        format!("Unknown localization {}", text_id)
    }
}

fn read_from_file<P: AsRef<Path>>(filename: P) -> io::Result<FluentResource> {
    let mut file = File::open(filename)?;
    let mut string = String::new();

    file.read_to_string(&mut string)?;

    Ok(FluentResource::try_new(string).expect("File did not parse!"))
}

fn read_from_dir<P: AsRef<Path>>(dirname: P) -> io::Result<Vec<FluentResource>> {
    let mut result = Vec::new();
    for dir_entry in read_dir(dirname)? {
        let entry = dir_entry?;
        let resource = read_from_file(entry.path())?;
        result.push(resource);
    }
    Ok(result)
}

pub fn create_bundle(
    lang: &str,
    resources: &'static Vec<FluentResource>,
    core_resource: Option<&'static FluentResource>,
    customizer: &impl Fn(&mut FluentBundle<'static>)
) -> FluentBundle<'static> {
    let mut bundle = FluentBundle::new(&[lang]);
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

pub fn build_resources(dir: &str) -> HashMap<String, Vec<FluentResource>> {
    let mut all_resources = HashMap::new();
    let entries = read_dir(dir).unwrap();
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

pub fn build_bundles(
    resources: &'static HashMap<String, Vec<FluentResource>>,
    core_resource: Option<&'static FluentResource>,
    customizer: impl Fn(&mut FluentBundle<'static>)
) -> HashMap<String, FluentBundle<'static>> {
    let mut bundles = HashMap::new();
    for (ref k, ref v) in &*resources {
        bundles.insert(k.to_string(), create_bundle(&k, &v, core_resource, &customizer));
    }
    bundles
}

pub fn load_core_resource(path: &str) -> FluentResource {
    read_from_file(path).expect("cannot find core resource")
}
