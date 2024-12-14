# Fluent Templates: A High level Fluent API.

![Build & Test](https://github.com/XAMPPRocky/fluent-templates/workflows/Build%20&%20Test/badge.svg)
[![crates.io](https://img.shields.io/crates/d/fluent-templates.svg)](https://crates.io/crates/fluent-templates)
[![Help Wanted](https://img.shields.io/github/issues/XAMPPRocky/fluent-templates/help%20wanted?color=green)](https://github.com/XAMPPRocky/fluent-templates/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
[![Lines Of Code](https://tokei.rs/b1/github/XAMPPRocky/fluent-templates?category=code)](https://github.com/XAMPPRocky/tokei)
[![Documentation](https://docs.rs/fluent-templates/badge.svg)](https://docs.rs/fluent-templates/)

`fluent-templates` lets you to easily integrate [Fluent] localisation into
your Rust application or library. It does this by providing a high level
"loader" API that loads fluent strings based on simple language negotiation,
and the `FluentLoader` struct which is a `Loader` agnostic container type
that comes with optional trait implementations for popular templating
engines such as handlebars or tera that allow you to be able to use your
localisations in your templates with no boilerplate.

## Loaders
Currently this crate provides two different kinds of loaders that cover two
main use cases.

- [`static_loader!`] — A procedural macro that loads your fluent resources
  at *compile-time* into your binary and creates a new [`StaticLoader`]
  static variable that allows you to access the localisations.
  `static_loader!` is most useful when you want to localise your
  application and want to ship your fluent resources with your binary.

- [`ArcLoader`] — A struct that loads your fluent resources at *run-time*
  using `Arc` as its backing storage. `ArcLoader` is most useful for when
  you want to be able to change and/or update localisations at run-time, or
  if you're writing a developer tool that wants to provide fluent
  localisation in your own application such as a static site generator.


## `static_loader!`
The easiest way to use `fluent-templates` is to use the [`static_loader!`]
procedural macro that will create a new [`StaticLoader`] static variable.

### Basic Example
```rust
fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "./tests/locales",
        // The language to falback on if something is not present.
        fallback_language: "en-US",
        // Optional: A fluent resource that is shared with every locale.
        core_locales: "./tests/locales/core.ftl",
    };
}
```

### Customise Example
You can also modify each `FluentBundle` on initialisation to be able to
change configuration or add resources from Rust.
```rust
use std::sync::LazyLock;
use fluent_bundle::FluentResource;
use fluent_templates::static_loader;

static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "./tests/locales",
        // The language to falback on if something is not present.
        fallback_language: "en-US",
        // Optional: A fluent resource that is shared with every locale.
        core_locales: "./tests/locales/core.ftl",
        // Optional: A function that is run over each fluent bundle.
        customise: |bundle| {
            // Since this will be called for each locale bundle and
            // `FluentResource`s need to be either `&'static` or behind an
            // `Arc` it's recommended you use lazily initialised
            // static variables.
            static CRATE_VERSION_FTL: LazyLock<FluentResource> = LazyLock::new(|| {
                let ftl_string = String::from(
                    concat!("-crate-version = {}", env!("CARGO_PKG_VERSION"))
                );

                FluentResource::try_new(ftl_string).unwrap()
            });

            bundle.add_resource(&CRATE_VERSION_FTL);
        }
    };
}
```

## Locales Directory
`fluent-templates` will collect all subdirectories that match a valid
[Unicode Language Identifier][uli] and bundle all fluent files found in
those directories and map those resources to the respective identifier.
`fluent-templates` will recurse through each language directory as needed
and will respect any `.gitignore` or `.ignore` files present.

[uli]: https://docs.rs/unic-langid/0.9.0/unic_langid/

### Example Layout
```text
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

### Looking up fluent resources
You can use the [`Loader`] trait to `lookup` a given fluent resource, and
provide any additional arguments as needed with `lookup_with_args`.

#### Example
```fluent
 # In `locales/en-US/main.ftl`
 hello-world = Hello World!
 greeting = Hello { $name }!

 # In `locales/fr/main.ftl`
 hello-world = Bonjour le monde!
 greeting = Bonjour { $name }!

 # In `locales/de/main.ftl`
 hello-world = Hallo Welt!
 greeting = Hallo { $name }!
```

```rust
use std::collections::HashMap;

use unic_langid::{LanguageIdentifier, langid};
use fluent_templates::{Loader, static_loader};

const US_ENGLISH: LanguageIdentifier = langid!("en-US");
const FRENCH: LanguageIdentifier = langid!("fr");
const GERMAN: LanguageIdentifier = langid!("de");

static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn main() {
    assert_eq!("Hello World!", LOCALES.lookup(&US_ENGLISH, "hello-world"));
    assert_eq!("Bonjour le monde!", LOCALES.lookup(&FRENCH, "hello-world"));
    assert_eq!("Hallo Welt!", LOCALES.lookup(&GERMAN, "hello-world"));

    let args = {
        let mut map = HashMap::new();
        map.insert(String::from("name"), "Alice".into());
        map
    };

    assert_eq!("Hello Alice!", LOCALES.lookup_with_args(&US_ENGLISH, "greeting", &args));
    assert_eq!("Bonjour Alice!", LOCALES.lookup_with_args(&FRENCH, "greeting", &args));
    assert_eq!("Hallo Alice!", LOCALES.lookup_with_args(&GERMAN, "greeting", &args));
}
```

### Tera
With the `tera` feature you can use `FluentLoader` as a Tera function.
It accepts a `key` parameter pointing to a fluent resource and `lang` for
what language to get that key for. Optionally you can pass extra arguments
to the function as arguments to the resource. `fluent-templates` will
automatically convert argument keys from Tera's `snake_case` to the fluent's
preferred `kebab-case` arguments.

```toml
fluent-templates = { version = "*", features = ["tera"] }
```

```rust
use fluent_templates::{FluentLoader, static_loader};

static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn main() {
    let mut tera = tera::Tera::default();
    let ctx = tera::Context::default();
    tera.register_function("fluent", FluentLoader::new(&*LOCALES));
    assert_eq!(
        "Hello World!",
        tera.render_str(r#"{{ fluent(key="hello-world", lang="en-US") }}"#, &ctx).unwrap()
    );
    assert_eq!(
        "Hello Alice!",
        tera.render_str(r#"{{ fluent(key="greeting", lang="en-US", name="Alice") }}"#, &ctx).unwrap()
    );
}
```

### Handlebars
In handlebars, `fluent-templates` will read the `lang` field in your
[`handlebars::Context`] while rendering.

```toml
fluent-templates = { version = "*", features = ["handlebars"] }
```

```rust
use fluent_templates::{FluentLoader, static_loader};

static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn main() {
    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentLoader::new(&*LOCALES)));
    let data = serde_json::json!({"lang": "zh-CN"});
    assert_eq!("Hello World!", handlebars.render_template(r#"{{fluent "hello-world"}}"#, &data).unwrap());
    assert_eq!("Hello Alice!", handlebars.render_template(r#"{{fluent "greeting" name="Alice"}}"#, &data).unwrap());
}
```

### Handlebars helper syntax.
The main helper provided is the `{{fluent}}` helper. If you have the
following Fluent file:

```fluent
foo-bar = "foo bar"
placeholder = this has a placeholder { $variable }
placeholder2 = this has { $variable1 } { $variable2 }
```

You can include the strings in your template with

```hbs
<!-- will render "foo bar" -->
{{fluent "foo-bar"}}
<!-- will render "this has a placeholder baz" -->
{{fluent "placeholder" variable="baz"}}
```

You may also use the `{{fluentparam}}` helper to specify [variables],
especially if you need them to be multiline.

```hbs
{{#fluent "placeholder2"}}
    {{#fluentparam "variable1"}}
        first line
        second line
    {{/fluentparam}}
    {{#fluentparam "variable2"}}
        first line
        second line
    {{/fluentparam}}
{{/fluent}}
```

### FAQ

#### Why is there extra characters around the values of arguments?
These are called "Unicode Isolating Marks" that used to allow the text to be bidirectional.
You can disable this with `FluentBundle::set_isolating_marks` being set to `false`.

```rust
static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}
```

[variables]: https://projectfluent.org/fluent/guide/variables.html
[`static_loader!`]: https://docs.rs/fluent-templates/0.5.4/fluent_templates/macro.static_loader.html
[`StaticLoader`]: https://docs.rs/fluent-templates/0.5.4/fluent_templates/struct.StaticLoader.html
[`ArcLoader`]: https://docs.rs/fluent-templates/0.5.4/fluent_templates/struct.ArcLoader.html
[`Loader`]: https://docs.rs/fluent-templates/0.5.4/fluent_templates/trait.Loader.html
[`handlebars::Context`]: https://docs.rs/handlebars/3.1.0/handlebars/struct.Context.html
[fluent]: https://projectfluent.org
