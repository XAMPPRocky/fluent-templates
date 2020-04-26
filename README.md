# `fluent-templates`: Templating for Fluent.

![CI Status](https://github.com/XAMPPRocky/fluent-templates/workflows/Rust/badge.svg?branch=master&event=push)
![Current Version](https://img.shields.io/crates/v/fluent-templates.svg)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/fluent-templates.svg)](#license)

This crate provides you with the ability to create [Fluent] loaders for use
in templating engines such as handlebars and tera.

## Basic handlebars example
```rust
//! Requires `--features handlebars`.
use fluent_templates::*;
use handlebars::*;
use serde_json::*;

static_loader!(create_loader, "./locales/", "en-US");

fn init(handlebars: &mut Handlebars) {
    let loader = create_loader();
    let helper = FluentHelper::new(loader);
    handlebars.register_helper("fluent", Box::new(helper));
}

fn render_page(handlebars: &Handlebars) -> String {
    let data = json!({"lang": "zh-CN"});
    handlebars.render_template("{{fluent \"foo-bar\"}} baz", &data).unwrap()
}
```

[fluent]: https://projectfluent.org
