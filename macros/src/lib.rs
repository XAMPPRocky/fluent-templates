use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Ident, Result,
};

struct StaticLoader {
    vis: Option<syn::Visibility>,
    name: Ident,
    locales_directory: PathBuf,
    fallback_language: syn::LitStr,
    core_locales: Option<PathBuf>,
    customise: Option<syn::ExprClosure>,
}

impl Parse for StaticLoader {
    fn parse(input: ParseStream) -> Result<Self> {
        let workspace_path = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| String::from("./")),
        );
        let vis = input.parse::<syn::Visibility>().ok();
        input.parse::<token::Static>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<token::Eq>()?;
        let fields;
        braced!(fields in input);
        let mut core_locales: Option<syn::LitStr> = None;
        let mut customise = None;
        let mut fallback_language = None;
        let mut locales_directory: Option<syn::LitStr> = None;

        while !fields.is_empty() {
            let k = fields.parse::<Ident>()?;
            fields.parse::<syn::Token![:]>()?;

            if k == "customise" {
                customise = Some(fields.parse()?);
            } else if k == "core_locales" {
                core_locales = Some(fields.parse()?);
            } else if k == "fallback_language" {
                fallback_language = Some(fields.parse()?);
            } else if k == "locales" {
                locales_directory = Some(fields.parse()?);
            } else {
                return Err(syn::Error::new(k.span(), "Not a valid parameter"));
            }

            if fields.is_empty() {
                break;
            }
            fields.parse::<token::Comma>()?;
        }
        input.parse::<token::Semi>()?;

        let locales_directory = locales_directory
            .ok_or_else(|| syn::Error::new(name.span(), "Missing `locales` field"))?;

        let locales_directory_path = workspace_path.join(locales_directory.value());

        if std::fs::metadata(&locales_directory_path).is_err() {
            return Err(syn::Error::new(locales_directory.span(), format!("Couldn't read locales directory, this path should be relative to your crate's `Cargo.toml`. Looking for: {:?}", locales_directory_path)));
        }

        let core_locales = if let Some(core_locales) = &core_locales {
            let core_locales_path = workspace_path.join(core_locales.value());
            if std::fs::metadata(&core_locales_path).is_err() {
                return Err(syn::Error::new(core_locales.span(), format!("Couldn't read core fluent resource, this path should be relative to your crate's `Cargo.toml`. Looking for: {:?}", core_locales_path)));
            }
            Some(core_locales_path)
        } else {
            None
        };

        let fallback_language = fallback_language
            .ok_or_else(|| syn::Error::new(name.span(), "Missing `fallback_language` field"))?;

        Ok(Self {
            vis,
            name,
            locales_directory: locales_directory_path,
            fallback_language,
            core_locales,
            customise,
        })
    }
}

/// Copied from `fluent_templates::loader` to avoid needing a seperate crate to
/// share the function.
fn build_resources(dir: impl AsRef<std::path::Path>) -> HashMap<String, Vec<String>> {
    let mut all_resources = HashMap::new();
    for entry in std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|rs| rs.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
    {
        if let Some(lang) = entry
            .file_name()
            .into_string()
            .ok()
            .filter(|l| l.parse::<unic_langid::LanguageIdentifier>().is_ok())
        {
            let resources = read_from_dir(entry.path());
            all_resources.insert(lang, resources);
        }
    }
    all_resources
}

/// Copied from `fluent_templates::fs` to avoid needing a seperate crate to
/// share the function.
pub(crate) fn read_from_dir<P: AsRef<Path>>(path: P) -> Vec<String> {
    #[cfg(not(any(feature = "ignore", feature = "walkdir",)))]
    compile_error!("one of the features `ignore` or `walkdir` must be enabled.");

    #[cfg(feature = "ignore")]
    {
        let (tx, rx) = flume::unbounded();

        ignore::WalkBuilder::new(path)
            .follow_links(true)
            .build_parallel()
            .run(|| {
                let tx = tx.clone();
                Box::new(move |result| {
                    if let Ok(entry) = result {
                        if entry.file_type().as_ref().map_or(false, |e| e.is_file())
                            && entry.path().extension().map_or(false, |e| e == "ftl")
                        {
                            tx.send(entry.path().display().to_string()).unwrap();
                        }
                    }

                    ignore::WalkState::Continue
                })
            });

        return rx.drain().collect();
    }

    #[cfg(all(not(feature = "ignore"), feature = "walkdir"))]
    walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |e| e == "ftl"))
        .map(|e| e.path().display().to_string())
        .collect()
}

/// Loads all of your fluent resources at compile time as `&'static str`s and
/// and creates a new `StaticLoader` static variable that you can use in your
/// program. This allows you to easily ship your localisations as part of a
/// single binary.
///
/// ### Example
/// ```no_compile
/// fluent_templates::static_loader! {
///     // Declare our `StaticLoader` named `LOCALES`.
///     static LOCALES = {
///         // The directory of localisations and fluent resources.
///         locales: "./tests/locales",
///         // The language to falback on if something is not present.
///         fallback_language: "en-US",
///         // Optional: A shared fluent resource
///         core_locales: "./tests/locales/core.ftl",
///         // Optional: A function that is run over each fluent bundle.
///         customise: |bundle| {},
///     };
/// }
/// ```
#[proc_macro]
#[allow(non_snake_case)]
pub fn static_loader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let StaticLoader {
        core_locales,
        customise,
        fallback_language,
        locales_directory,
        name,
        vis,
        ..
    } = parse_macro_input!(input as StaticLoader);
    let CRATE_NAME: TokenStream = quote!(fluent_templates);
    let LAZY: TokenStream = quote!(#CRATE_NAME::once_cell::sync::Lazy);
    let LANGUAGE_IDENTIFIER: TokenStream = quote!(#CRATE_NAME::loader::LanguageIdentifier);
    let FLUENT_BUNDLE: TokenStream = quote!(#CRATE_NAME::FluentBundle);
    let FLUENT_RESOURCE: TokenStream = quote!(#CRATE_NAME::fluent_bundle::FluentResource);
    let HASHMAP: TokenStream = quote!(std::collections::HashMap);

    let core_resource = if let Some(core_locales) = &core_locales {
        let core_locales = core_locales.display().to_string();
        quote!(
            Some(
                #CRATE_NAME::fs::resource_from_str(include_str!(#core_locales))
                    .expect("Couldn't load core resources")
            )
        )
    } else {
        quote!(None)
    };

    let fallback_language_value = fallback_language.value();
    let maybe_fallback_language_identifier =
        fallback_language_value.parse::<unic_langid::LanguageIdentifier>();
    if !maybe_fallback_language_identifier.is_ok() {
        return syn::Error::new(
            fallback_language.span(),
            format!(
                "Invalid language identifier \"{}\" for fallback language",
                &fallback_language_value
            ),
        )
        .to_compile_error()
        .into();
    }
    let fallback_language_identifier = maybe_fallback_language_identifier.unwrap();

    let mut insert_resources: Vec<_> = build_resources(locales_directory).into_iter().collect();

    if !insert_resources
        .iter()
        .any(|(lang, _)| *lang == fallback_language_value)
    {
        return syn::Error::new(
            fallback_language.span(),
            "Fallback language not found in locales directory",
        )
        .to_compile_error()
        .into();
    }

    // Make the output `TokenStream` only depend on the filenames and the file contents,
    // not hashmap/filesystem iteration order.
    insert_resources.sort();

    let insert_resources = insert_resources
        .into_iter()
        .map(|(locale, resources)| {
            quote!(
                resources.insert(
                    #locale.parse().unwrap(),
                    vec![#(#CRATE_NAME::fs::resource_from_str(include_str!(#resources)).unwrap(),)*]
                );
            )
        })
        .collect::<TokenStream>();

    let customise = customise.map_or(quote!(|_| ()), |c| quote!(#c));

    let resource_map = quote! {
        let mut resources = #HASHMAP::new();
        #insert_resources
        resources
    };

    let FALLBACK: TokenStream = {
        // Initialize the language identifier from the fallback language string
        // using unsafe code from bytes built at compile time.

        let str_to_u32 = |part_str: &str| -> u32 {
            let script_as_bytes = part_str.as_bytes();
            let mut script_buf = [0u8; 4];
            script_buf[..script_as_bytes.len()].copy_from_slice(script_as_bytes);
            u32::from_le_bytes(script_buf)
        };

        let str_to_u64 = |part_str: &str| -> u64 {
            let script_as_bytes = part_str.as_bytes();
            let mut script_buf = [0u8; 8];
            script_buf[..script_as_bytes.len()].copy_from_slice(script_as_bytes);
            u64::from_le_bytes(script_buf)
        };

        let language_quote = {
            let language_as_u64 = str_to_u64(fallback_language_identifier.language.as_str());
            quote! {
                unsafe { unic_langid::subtags::Language::from_raw_unchecked(#language_as_u64) }
            }
        };
        let script_quote = match fallback_language_identifier.script {
            Some(script) => {
                let script_as_u32 = str_to_u32(script.as_str());
                quote! {
                    Some(unsafe { unic_langid::subtags::Script::from_raw_unchecked(#script_as_u32) })
                }
            }
            None => quote!(None),
        };
        let region_quote = match fallback_language_identifier.region {
            Some(region) => {
                let region_as_u32 = str_to_u32(region.as_str());
                quote! {
                    Some(unsafe { unic_langid::subtags::Region::from_raw_unchecked(#region_as_u32) })
                }
            }
            None => quote!(None),
        };
        let variants_quote = {
            let variants_as_str = fallback_language_identifier
                .variants()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            match variants_as_str.is_empty() {
                false => {
                    let mut variants_quote = "Some(Box::new([".to_string();
                    for variant in variants_as_str {
                        let variant_as_u64 = str_to_u64(variant);
                        variants_quote.push_str(&format!(
                            "unsafe {{ unic_langid::subtags::Variant::from_raw_unchecked({}) }},",
                            variant_as_u64
                        ));
                    }
                    variants_quote.pop();
                    variants_quote.push_str("]))");
                    variants_quote.parse::<TokenStream>().unwrap()
                }
                true => quote!(None),
            }
        };

        quote! {
            #CRATE_NAME::LanguageIdentifier::from_raw_parts_unchecked(
                #language_quote,
                #script_quote,
                #region_quote,
                #variants_quote,
            )
        }
    };

    let quote = quote! {
        #vis static #name : #LAZY<#CRATE_NAME::StaticLoader> = #LAZY::new(|| {
            static CORE_RESOURCE:
                #LAZY<Option<#FLUENT_RESOURCE>> =
                #LAZY::new(|| { #core_resource });

            static RESOURCES:
                #LAZY<#HASHMAP<#LANGUAGE_IDENTIFIER, Vec<#FLUENT_RESOURCE>>> =
                #LAZY::new(|| { #resource_map });

            static BUNDLES:
                #LAZY<
                    #HASHMAP<
                        #LANGUAGE_IDENTIFIER,
                        #FLUENT_BUNDLE<&'static #FLUENT_RESOURCE>
                    >
                > =
                #LAZY::new(||  {
                    #CRATE_NAME::loader::build_bundles(
                        &*RESOURCES,
                        CORE_RESOURCE.as_ref(),
                        #customise
                    )
                });

            static FALLBACKS:
                #LAZY<#HASHMAP<#LANGUAGE_IDENTIFIER, Vec<#LANGUAGE_IDENTIFIER>>> =
                #LAZY::new(|| #CRATE_NAME::loader::build_fallbacks(
                    &RESOURCES.keys().cloned().collect::<Vec<#LANGUAGE_IDENTIFIER>>()
                ));

            #CRATE_NAME::StaticLoader::new(
                &BUNDLES,
                &FALLBACKS,
                #FALLBACK
            )
        });
    };

    // println!("{}", quote);

    proc_macro::TokenStream::from(quote)
}
