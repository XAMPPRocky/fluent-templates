use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable,
};

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use handlebars::template::{Parameter, TemplateElement};
use serde_json::Value as Json;
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use crate::Loader;

pub struct I18NHelper {
    loader: Loader,
}

impl I18NHelper {
    pub fn new() -> Self {
        Self {
            loader: Loader::new(),
        }
    }
}

#[derive(Default)]
struct StringOutput {
    pub s: String,
}

impl Output for StringOutput {
    fn write(&mut self, seg: &str) -> Result<(), io::Error> {
        self.s.push_str(seg);
        Ok(())
    }
}

impl HelperDef for I18NHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        reg: &'reg Handlebars,
        context: &'rc Context,
        rcx: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let id = if let Some(id) = h.param(0) {
            id
        } else {
            return Err(RenderError::new(
                "{{text}} must have at least one parameter",
            ));
        };

        let id = if let Some(id) = id.path() {
            id
        } else {
            return Err(RenderError::new("{{text}} takes an identifier parameter"));
        };

        let mut args = if h.hash().is_empty() {
            None
        } else {
            let map = h
                .hash()
                .iter()
                .filter_map(|(k, v)| {
                    let json = v.value();
                    let val = match *json {
                        Json::Number(ref n) => FluentValue::Number(n.to_string()),
                        Json::String(ref s) => FluentValue::String(s.to_string()),
                        _ => return None,
                    };
                    Some((&**k, val))
                })
                .collect();
            Some(map)
        };

        if let Some(tpl) = h.template() {
            if args.is_none() {
                args = Some(HashMap::new());
            }
            let args = args.as_mut().unwrap();
            for element in &tpl.elements {
                if let TemplateElement::HelperBlock(ref block) = element {
                    if block.name != "textparam" {
                        return Err(RenderError::new(format!(
                            "{{{{text}}}} can only contain {{{{textparam}}}} elements, not {}",
                            block.name
                        )));
                    }
                    let id = if let Some(el) = block.params.get(0) {
                        if let Parameter::Name(ref s) = *el {
                            s
                        } else {
                            return Err(RenderError::new(
                                "{{textparam}} takes an identifier parameter",
                            ));
                        }
                    } else {
                        return Err(RenderError::new("{{textparam}} must have one parameter"));
                    };
                    if let Some(ref tpl) = block.template {
                        let mut s = StringOutput::default();
                        tpl.render(reg, context, rcx, &mut s)?;
                        args.insert(&*id, FluentValue::String(s.s));
                    }
                }
            }
        }
        let lang = context
            .data()
            .get("lang")
            .expect("Language not set in context")
            .as_str()
            .expect("Language must be string");
        let pontoon = context
            .data()
            .get("pontoon_enabled")
            .expect("Pontoon not set in context")
            .as_bool()
            .expect("Pontoon must be boolean");
        let in_context =
            pontoon && !id.ends_with("-title") && !id.ends_with("-alt") && !id.starts_with("meta-");

        let response = self.loader.lookup(lang, &id, args.as_ref());
        if in_context {
            out.write(&format!("<span data-l10n-id='{}'>", id))
                .map_err(RenderError::with)?;
        }
        out.write(&response).map_err(RenderError::with)?;
        if in_context {
            out.write("</span>").map_err(RenderError::with)?;
        }
        Ok(())
    }
}
