use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};
use fluent_langneg::negotiate_languages;

pub use unic_langid::{langid, langids, LanguageIdentifier};

mod arc_loader;
mod static_loader;

pub use arc_loader::{ArcLoader, ArcLoaderBuilder};
pub use static_loader::StaticLoader;

/// Something capable of looking up Fluent keys given a language.
///
/// Use [SimpleLoader] if you just need the basics
pub trait Loader {
    fn lookup(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> String;
}

pub fn build_fallbacks(
    locales: &[LanguageIdentifier],
) -> HashMap<LanguageIdentifier, Vec<LanguageIdentifier>> {
    let mut map = HashMap::new();

    for locale in locales.iter() {
        map.insert(
            locale.to_owned(),
            negotiate_languages(
                &[locale],
                locales,
                None,
                fluent_langneg::NegotiationStrategy::Filtering,
            )
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        );
    }

    map
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

        // Prevent loading non-FTL files as translations, such as VIM temporary files.
        if entry.path().extension().and_then(|e| e.to_str()) != Some("ftl") {
            continue;
        }

        let resource = read_from_file(entry.path())?;
        result.push(resource);
    }
    Ok(result)
}

pub fn create_bundle(
    lang: LanguageIdentifier,
    resources: &'static [FluentResource],
    core_resource: Option<&'static FluentResource>,
    customizer: &impl Fn(&mut FluentBundle<&'static FluentResource>),
) -> FluentBundle<&'static FluentResource> {
    let mut bundle: FluentBundle<&'static FluentResource> = FluentBundle::new(&[lang]);
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

pub fn build_resources(dir: &str) -> HashMap<LanguageIdentifier, Vec<FluentResource>> {
    let mut all_resources = HashMap::new();
    let entries = read_dir(dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            if let Ok(lang) = entry.file_name().into_string() {
                let resources = read_from_dir(entry.path()).unwrap();
                all_resources.insert(lang.parse().unwrap(), resources);
            }
        }
    }
    all_resources
}

pub fn build_bundles(
    resources: &'static HashMap<LanguageIdentifier, Vec<FluentResource>>,
    core_resource: Option<&'static FluentResource>,
    customizer: impl Fn(&mut FluentBundle<&'static FluentResource>),
) -> HashMap<LanguageIdentifier, FluentBundle<&'static FluentResource>> {
    let mut bundles = HashMap::new();
    for (k, ref v) in resources.iter() {
        bundles.insert(
            k.clone(),
            create_bundle(k.clone(), &v, core_resource, &customizer),
        );
    }
    bundles
}

pub fn load_core_resource(path: &str) -> FluentResource {
    read_from_file(path).expect("cannot find core resource")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_bundle::concurrent::FluentBundle;
    use std::error::Error;

    #[test]
    fn test_load_from_dir() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        std::fs::write(dir.path().join("core.ftl"), "foo = bar\n".as_bytes())?;
        std::fs::write(dir.path().join("other.ftl"), "bar = baz\n".as_bytes())?;
        std::fs::write(dir.path().join("invalid.txt"), "baz = foo\n".as_bytes())?;
        std::fs::write(dir.path().join(".binary_file.swp"), &[0, 1, 2, 3, 4, 5])?;

        let result = read_from_dir(dir.path())?;
        assert_eq!(2, result.len()); // Doesn't include the binary file or the txt file

        let mut bundle = FluentBundle::new(&[unic_langid::langid!("en-US")]);
        for resource in &result {
            bundle.add_resource(resource).unwrap();
        }

        let mut errors = Vec::new();

        // Ensure the correct files were loaded
        assert_eq!(
            "bar",
            bundle.format_pattern(
                bundle.get_message("foo").and_then(|m| m.value).unwrap(),
                None,
                &mut errors
            )
        );

        assert_eq!(
            "baz",
            bundle.format_pattern(
                bundle.get_message("bar").and_then(|m| m.value).unwrap(),
                None,
                &mut errors
            )
        );
        assert_eq!(None, bundle.get_message("baz")); // The extension was txt

        Ok(())
    }
}
