use std::{collections::HashMap, fs, path::Path};

use ignore::WalkBuilder;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Ident, Result,
};

struct StaticLoader {
    name: Ident,
    locales_directory: syn::LitStr,
    fallback_language: syn::LitStr,
    core_locales: Option<syn::LitStr>,
    customise: Option<syn::ExprClosure>,
}

impl Parse for StaticLoader {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<token::Static>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<token::Eq>()?;
        let fields;
        braced!(fields in input);
        let mut core_locales = None;
        let mut customise = None;
        let mut fallback_language = None;
        let mut locales_directory = None;

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
        let fallback_language = fallback_language
            .ok_or_else(|| syn::Error::new(name.span(), "Missing `fallback_language` field"))?;

        Ok(Self {
            name,
            locales_directory,
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
    let (tx, rx) = flume::unbounded();

    WalkBuilder::new(path).build_parallel().run(|| {
        Box::new(|result| {
            let tx = tx.clone();
            if let Ok(entry) = result {
                if entry
                    .file_type()
                    .as_ref()
                    .map_or(false, fs::FileType::is_file)
                    && entry.path().extension().map_or(false, |e| e == "ftl")
                {
                    tx.send(std::fs::read_to_string(entry.path()).unwrap())
                        .unwrap();
                }
            }

            ignore::WalkState::Continue
        })
    });

    rx.drain().collect::<Vec<_>>()
}

#[proc_macro]
#[allow(non_snake_case)]
pub fn static_loader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let StaticLoader {
        core_locales,
        customise,
        fallback_language,
        locales_directory,
        name,
        ..
    } = parse_macro_input!(input as StaticLoader);
    let CRATE_NAME: TokenStream = quote!(fluent_templates);
    let ARC: TokenStream = quote!(std::sync::Arc);
    let LAZY: TokenStream = quote!(#CRATE_NAME::once_cell::sync::Lazy);
    let LANGUAGE_IDENTIFIER: TokenStream = quote!(#CRATE_NAME::loader::LanguageIdentifier);
    let FLUENT_BUNDLE: TokenStream = quote!(#CRATE_NAME::fluent_bundle::concurrent::FluentBundle);
    let FLUENT_RESOURCE: TokenStream = quote!(#CRATE_NAME::fluent_bundle::FluentResource);
    let HASHMAP: TokenStream = quote!(std::collections::HashMap);

    let core_contents = if let Some(core_locales) = &core_locales {
        match std::fs::read_to_string(core_locales.value()) {
            Ok(string) => string,
            Err(_) => panic!("Couldn't read {}", core_locales.value()),
        }
    } else {
        String::new()
    };

    let insert_resources = build_resources(locales_directory.value())
        .into_iter()
        .map(|(locale, resources)| {
            quote!(
                resources.insert(
                    #locale.parse().unwrap(),
                    vec![#(#CRATE_NAME::fs::resource_from_str(#resources).unwrap(),)*]
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

    let quote = quote! {
        static #name : #LAZY<#ARC<#CRATE_NAME::StaticLoader>> = #LAZY::new(|| {
            static CORE_RESOURCE:
                #LAZY<#FLUENT_RESOURCE> =
                #LAZY::new(|| #CRATE_NAME::fs::resource_from_str(#core_contents).expect("Couldn't load core resources"),);

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
                        Some(&CORE_RESOURCE),
                        #customise
                    )
                });

            static LOCALES:
                #LAZY<Vec<#LANGUAGE_IDENTIFIER>> =
                #LAZY::new(|| RESOURCES.keys().cloned().collect());

            static FALLBACKS:
                #LAZY<#HASHMAP<#LANGUAGE_IDENTIFIER, Vec<#LANGUAGE_IDENTIFIER>>> =
                #LAZY::new(|| #CRATE_NAME::loader::build_fallbacks(&*LOCALES));

            #ARC::new(
                #CRATE_NAME::StaticLoader::new(
                    &BUNDLES,
                    &FALLBACKS,
                    #fallback_language.parse().expect("invalid fallback language")
                )
            )
        });
    };

    // println!("{}", quote);

    proc_macro::TokenStream::from(quote)
}
