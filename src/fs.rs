use std::fs;
use std::path::Path;

use fluent_bundle::FluentResource;
pub use unic_langid::{langid, langids, LanguageIdentifier};

use crate::error;

pub fn read_from_file<P: AsRef<Path>>(path: P) -> crate::Result<FluentResource> {
    let path = path.as_ref();
    resource_from_str(
        &fs::read_to_string(path).map_err(|source| error::LoaderError::Fs {
            path: path.into(),
            source,
        })?,
    )
}

pub fn resource_from_str(src: &str) -> crate::Result<FluentResource> {
    FluentResource::try_new(src.to_owned())
        .map_err(|(_, errs)| error::FluentError::from(errs).into())
}

pub fn resources_from_vec(srcs: &[String]) -> crate::Result<Vec<FluentResource>> {
    let mut vec = Vec::with_capacity(srcs.len());

    for src in srcs {
        vec.push(resource_from_str(src)?);
    }

    Ok(vec)
}

pub(crate) fn read_from_dir<P: AsRef<Path>>(path: P) -> crate::Result<Vec<FluentResource>> {
    let (tx, rx) = flume::unbounded();

    #[cfg(not(any(feature = "use-ignore", feature = "walkdir",)))]
    compile_error!("one of the features `use-ignore` or `walkdir` must be enabled.");

    #[cfg(feature = "use-ignore")]
    ignore::WalkBuilder::new(path).build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry
                    .file_type()
                    .as_ref()
                    .map_or(false, fs::FileType::is_file)
                    && entry.path().extension().map_or(false, |e| e == "ftl")
                {
                    if let Ok(string) = std::fs::read_to_string(entry.path()) {
                        let _ = tx.send(string);
                    } else {
                        log::warn!("Couldn't read {}", entry.path().display());
                    }
                }
            }

            ignore::WalkState::Continue
        })
    });

    #[cfg(all(not(feature = "ignore"), feature = "walkdir"))]
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |e| e == "ftl"))
        .for_each(|e| {
            if let Ok(string) = std::fs::read_to_string(e.path()) {
                let _ = tx.send(string);
            } else {
                log::warn!("Couldn't read {}", e.path().display());
            }
        });

    resources_from_vec(&rx.drain().collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FluentBundle;
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

        let mut bundle = FluentBundle::new_concurrent(vec![unic_langid::langid!("en-US")]);
        for resource in &result {
            bundle.add_resource(resource).unwrap();
        }

        let mut errors = Vec::new();

        // Ensure the correct files were loaded
        assert_eq!(
            "bar",
            bundle.format_pattern(
                bundle.get_message("foo").and_then(|m| m.value()).unwrap(),
                None,
                &mut errors
            )
        );

        assert_eq!(
            "baz",
            bundle.format_pattern(
                bundle.get_message("bar").and_then(|m| m.value()).unwrap(),
                None,
                &mut errors
            )
        );
        assert_eq!(None, bundle.get_message("baz")); // The extension was txt

        Ok(())
    }
}
