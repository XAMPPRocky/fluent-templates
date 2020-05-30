# `fluent-templates`: A high level Fluent & Fluent Template API.

![CI Status](https://github.com/XAMPPRocky/fluent-templates/workflows/Rust/badge.svg?branch=master&event=push)
![Current Version](https://img.shields.io/crates/v/fluent-templates.svg)
[![License: MIT/Apache-2.0](https://img.shields.io/crates/l/fluent-templates.svg)](#license)

`fluent-templates` provides a high level API for adding fluent to your Rust
project, and provides optional integrations with templating languages.

## Loaders
The "loader" APIs read a directory and will load all fluent resources in each
subdirectory that is a valid [Unicode Language Identifier]. You can also
specify shared fluent resources that are used across all localisations.

- [`StaticLoader`] — Created with the `static_loader!` macro, `StaticLoader`
  will load the localisations into static memory. This most useful when the
  location of your fluent localisations is known at compile-time.
- [`ArcLoader`] — A struct that uses atomic references to store the
  localisations. Useful for when the source of your localisations are only known
  at run-time.

### Example Layout
```
locales
├── core.ftl
├── en-US
│   └── main.ftl
├── fr
│   └── main.ftl
├── zh-CN
│   └── main.ftl
└── zh-TW
    └── main.ftl
```

## Example
The easiest way to use `fluent-templates` is to use the [`static_loader!`]
macro:

```rust
fluent_templates::static_loader!(create_loader, "./locales/", "en-US");

fn main() {
    let loader = create_loader();

    assert_eq!("Hello World!", loader.lookup_single_language(&unic_langid::langid!("en-US"), "welcome-text", None).unwrap());
}
```

### Tera
```rust
# #[cfg(feature = "handlebars")] {
use tera::Tera;

fluent_templates::static_loader!(create_loader, "./locales/", "en-US");

fn init(tera: &mut Tera) {
    let loader = create_loader();
    let helper = fluent_templates::FluentHelper::new(loader);
    tera.register_function("fluent", helper);
}

fn render_page(tera: &mut Tera, ctx: &tera::Context) -> String {
    tera.render_str(r#"{{ fluent(key="foo-bar", lang="en") }} baz"#, ctx).unwrap()
}
# }
```

### Handlebars
```rust
# #[cfg(feature = "handlebars")] {
use handlebars::Handlebars;

fluent_templates::static_loader!(create_loader, "./locales/", "en-US");

fn init(handlebars: &mut Handlebars) {
    let loader = create_loader();
    let helper = fluent_templates::FluentHelper::new(loader);
    handlebars.register_helper("fluent", Box::new(helper));
}

fn render_page(handlebars: &Handlebars) -> String {
    let data = serde_json::json!({"lang": "zh-CN"});
    handlebars.render_template("{{fluent \"foo-bar\"}} baz", &data).unwrap()
}
# }
```

You should have a `locales/` folder somewhere with one folder per language
code, containing all of your FTL files. See the [`static_loader!`] macro
for more options.

Make sure the [`handlebars::Context`] has a top-level `"lang"` field when rendering.

### Handlebars helper syntax.

The main helper provided is the `{{fluent}}` helper. If you have the following Fluent
file:

```fluent
foo-bar = "foo bar"
placeholder = this has a placeholder { $variable }
```

You can include the strings in your template with

```hbs
{{fluent "foo-bar"}} <!-- will render "foo bar" -->
{{fluent "placeholder" variable="baz"}} <!-- will render "this has a placeholder baz" -->
``

You may also use the `{{fluentparam}}` helper to specify [variables], especially if you need
them to be multiline, like so:

```hbs
{{#fluent "placeholder"}}
    {{#fluentparam "variable"}}
        first line
        second line
    {{/fluentparam}}
{{/fluent}}
```

Multiple `{{fluentparam}}`s may be specified

[variables]: https://projectfluent.org/fluent/guide/variables.html
[`static_loader!`]: ./macro.static_loader.html

## Features/Template Engines

- [`handlebars`](https://docs.rs/handlebars)
- [`tera`](https://docs.rs/tera)

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
