use std::fs;
use std::path::Path;

use fluent_bundle::FluentResource;

use crate::error;
use snafu::*;
pub use unic_langid::{langid, langids, LanguageIdentifier};

pub fn read_from_file<P: AsRef<Path>>(path: P) -> crate::Result<FluentResource> {
    let path = path.as_ref();
    resource_from_string(fs::read_to_string(path).context(error::Fs { path })?)
}

pub fn resource_from_string(src: String) -> crate::Result<FluentResource> {
    FluentResource::try_new(src)
        .map_err(|(_, errs)| errs)
        .context(error::Fluent)

}

pub(crate) fn read_from_dir<P: AsRef<Path>>(path: P) -> crate::Result<Vec<FluentResource>> {
    let path = path.as_ref();
    let mut result = Vec::new();
    for dir_entry in fs::read_dir(path).context(error::Fs { path })? {
        let entry = dir_entry.context(error::Fs { path })?;

        // Prevent loading non-FTL files as translations, such as VIM temporary files.
        if entry.path().extension().and_then(|e| e.to_str()) != Some("ftl") {
            continue;
        }

        result.push(read_from_file(entry.path())?);
    }

    Ok(result)
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
