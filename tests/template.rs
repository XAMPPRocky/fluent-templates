#![allow(unused)]
#![allow(clippy::incompatible_msrv)]

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        core_locales: "./tests/locales/core.ftl",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fluent_templates::static_loader! {
    pub(crate) static _LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US"
    };
}

macro_rules! make_loaders {
    () => {{
        let arc = ArcLoader::builder("tests/locales", unic_langid::langid!("en-US"))
            .shared_resources(Some(&["./tests/locales/core.ftl".into()]))
            .customize(|bundle| bundle.set_use_isolating(false))
            .build()
            .unwrap();

        (FluentLoader::new(&*crate::LOCALES), FluentLoader::new(arc))
    }};
}

/// Generates tests for each loader in different locales.
macro_rules! generate_tests {
    ($(fn $locale_test_fn:ident ($template_engine:ident, $locale:expr) {
        $($assert_macro:ident ! ( $lhs:expr , $rhs:expr ) );* $(;)?
    })+) => {
        use fluent_templates::*;
        $(
            #[test]
            fn $locale_test_fn() {
                // when only minijinja feature is enable, serde_json isn't included
                #[cfg(any(feature = "handlebars", feature = "tera"))]
                let data = serde_json::json!({"lang": $locale});
                #[cfg(not(any(feature = "handlebars", feature = "tera")))]
                let data: Vec<(_, _)> = vec![("lang", $locale)];

                #[cfg(feature = "handlebars")]
                if stringify!($template_engine) == "handlebars" {
                    let (static_loader, arc_loader) = make_loaders!();
                    let mut static_handlebars = handlebars::Handlebars::new();
                    static_handlebars.register_helper("fluent", Box::new(static_loader));
                    let mut arc_handlebars = handlebars::Handlebars::new();
                    arc_handlebars.register_helper("fluent", Box::new(arc_loader));

                    $(
                        $assert_macro ! (
                            static_handlebars
                            .render_template($lhs, &data)
                            .unwrap(),
                            $rhs
                        );

                        $assert_macro ! (
                            arc_handlebars
                            .render_template($lhs, &data)
                            .unwrap(),
                            $rhs
                        );
                    )*
                }

                #[cfg(feature = "tera")]
                if stringify!($template_engine) == "tera" {
                    let (static_loader, arc_loader) = make_loaders!();
                    let mut static_tera = tera::Tera::default();
                    static_tera.register_function("fluent", static_loader);
                    let mut arc_tera = tera::Tera::default();
                    arc_tera.register_function("fluent", arc_loader);

                    $(
                        let lhs = $lhs.replace("{lang}", $locale);
                        $assert_macro ! (
                            static_tera
                            .render_str(&lhs, &tera::Context::from_value(data.clone()).unwrap())
                            .unwrap(),
                            $rhs
                        );

                        $assert_macro ! (
                            arc_tera
                            .render_str(&lhs, &tera::Context::from_value(data.clone()).unwrap())
                            .unwrap(),
                            $rhs
                        );
                    )*
                }

                #[cfg(feature = "minijinja")]
                if stringify!($template_engine) == "minijinja" {
                    let (static_loader, arc_loader) = make_loaders!();
                    let mut static_minijinja = minijinja::Environment::default();
                    static_minijinja.add_function("fluent", static_loader.into_minijinja_fn());
                    let mut arc_minijinja = minijinja::Environment::default();
                    arc_minijinja.add_function("fluent", arc_loader.into_minijinja_fn());

                    $(
                        let lhs = $lhs.replace("{lang}", $locale);
                        $assert_macro ! (
                            static_minijinja
                            .render_str(&lhs, &data.clone())
                            .unwrap(),
                            $rhs
                        );

                        $assert_macro ! (
                            arc_minijinja
                            .render_str(&lhs, &data.clone())
                            .unwrap(),
                            $rhs
                        );
                    )*
                }

            }
        )+
    };
}

#[cfg(feature = "handlebars")]
mod handlebars {
    generate_tests! {
        fn english(handlebars, "en-US") {
            assert_eq!(r#"{{fluent "simple"}}"#, "simple text");
            assert_eq!(r#"{{fluent "reference"}}"#, "simple text with a reference: foo");
            assert_eq!(r#"{{fluent "parameter" param="PARAM"}}"#, "text with a PARAM");
            assert_eq!(r#"{{fluent "parameter2" param="P1" multi-word-param="P2"}}"#, "text one P1 second P2");
            assert_eq!(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, "text with a blah blah");
            assert_eq!(r#"{{#fluent "parameter2"}}{{#fluentparam "param"}}foo{{/fluentparam}}{{#fluentparam "multi-word-param"}}bar{{/fluentparam}}{{/fluent}}"#, "text one foo second bar");
            assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");
        }

        fn french(handlebars, "fr") {
            assert_eq!(r#"{{fluent "simple"}}"#, "texte simple");
            assert_eq!(r#"{{fluent "reference"}}"#, "texte simple avec une référence: foo");
            assert_eq!(r#"{{fluent "parameter" param="PARAM"}}"#, "texte avec une PARAM");
            assert_eq!(r#"{{fluent "parameter2" param="P1" multi-word-param="P2"}}"#, "texte une P1 seconde P2");
            assert_eq!(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, "texte avec une blah blah");
            assert_eq!(r#"{{#fluent "parameter2"}}{{#fluentparam "param"}}foo{{/fluentparam}}{{#fluentparam "multi-word-param"}}bar{{/fluentparam}}{{/fluent}}"#, "texte une foo seconde bar");
            assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");

        }

        fn chinese(handlebars, "zh-TW") {
            assert_eq!(r#"{{fluent "exists"}}"#, "兒");
            assert_eq!(r#"{{fluent "fallback-zh"}}"#, "气");
            assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");
        }
    }
}

#[cfg(feature = "tera")]
mod tera {
    generate_tests! {
        fn english(tera, "en-US") {
            assert_eq!(r#"{{ fluent(key="simple", lang="{lang}") }}"#, "simple text");
            assert_eq!(r#"{{ fluent(key="reference", lang="{lang}") }}"#, "simple text with a reference: foo");
            assert_eq!(r#"{{ fluent(key="parameter", lang="{lang}", param="PARAM") }}"#, "text with a PARAM");
            assert_eq!(r#"{{ fluent(key="parameter2", lang="{lang}", param="P1", multi_word_param="P2") }}"#, "text one P1 second P2");
            assert_eq!(r#"{{ fluent(key="fallback", lang="{lang}") }}"#, "this should fall back");
        }

        fn french(tera, "fr") {
            assert_eq!(r#"{{ fluent(key="simple", lang="{lang}") }}"#, "texte simple");
            assert_eq!(r#"{{ fluent(key="reference", lang="{lang}") }}"#, "texte simple avec une référence: foo");
            assert_eq!(r#"{{ fluent(key="parameter", param="PARAM", lang="{lang}") }}"#, "texte avec une PARAM");
            assert_eq!(r#"{{ fluent(key="parameter2", param="P1", multi_word_param="P2", lang="{lang}") }}"#, "texte une P1 seconde P2");
            assert_eq!(r#"{{ fluent(key="fallback", lang="{lang}") }}"#, "this should fall back");

        }

        fn chinese(tera, "zh-TW") {
            assert_eq!(r#"{{ fluent(key="exists", lang="{lang}") }}"#, "兒");
            assert_eq!(r#"{{ fluent(key="fallback-zh", lang="{lang}") }}"#, "气");
            assert_eq!(r#"{{ fluent(key="fallback", lang="{lang}") }}"#, "this should fall back");
        }
    }

    /// Default lang argument works
    #[test]
    fn use_default_lang() {
        let loader = FluentLoader::new(&*super::LOCALES).with_default_lang("de".parse().unwrap());
        let mut tera = tera::Tera::default();
        tera.register_function("fluent", loader);
        let context = tera::Context::new();
        assert_eq!(
            tera.render_str(r#"{{ fluent(key="hello-world") }}"#, &context)
                .unwrap(),
            "Hallo Welt!"
        );
        assert_eq!(
            tera.render_str(r#"{{ fluent(key="hello-world", lang="fr") }}"#, &context)
                .unwrap(),
            "Bonjour le monde!"
        );
    }

    /// Rendering fails when no default and no explicit lang argument is provided
    #[test]
    fn no_default_and_no_argument_error() {
        let loader = FluentLoader::new(&*super::LOCALES);
        let mut tera = tera::Tera::default();
        tera.register_function("fluent", loader);
        let context = tera::Context::new();
        assert!(tera
            .render_str(r#"{{ fluent(key="hellow-world") }}"#, &context)
            .is_err());
    }
}

#[cfg(feature = "minijinja")]
mod minijinja {
    generate_tests! {
        fn english(minijinja, "en-US") {
            assert_eq!(r#"{{ fluent("simple", lang="{lang}") }}"#, "simple text");
            assert_eq!(r#"{{ fluent("reference", lang="{lang}") }}"#, "simple text with a reference: foo");
            assert_eq!(r#"{{ fluent("parameter", lang="{lang}", param="PARAM") }}"#, "text with a PARAM");
            assert_eq!(r#"{{ fluent("parameter2", lang="{lang}", param="P1", multi_word_param="P2") }}"#, "text one P1 second P2");
            assert_eq!(r#"{{ fluent("fallback", lang="{lang}") }}"#, "this should fall back");
        }

        fn french(minijinja, "fr") {
            assert_eq!(r#"{{ fluent("simple", lang="{lang}") }}"#, "texte simple");
            assert_eq!(r#"{{ fluent("reference", lang="{lang}") }}"#, "texte simple avec une référence: foo");
            assert_eq!(r#"{{ fluent("parameter", param="PARAM", lang="{lang}") }}"#, "texte avec une PARAM");
            assert_eq!(r#"{{ fluent("parameter2", param="P1", multi_word_param="P2", lang="{lang}") }}"#, "texte une P1 seconde P2");
            assert_eq!(r#"{{ fluent("fallback", lang="{lang}") }}"#, "this should fall back");

        }

        fn chinese(minijinja, "zh-TW") {
            assert_eq!(r#"{{ fluent("exists", lang="{lang}") }}"#, "兒");
            assert_eq!(r#"{{ fluent("fallback-zh", lang="{lang}") }}"#, "气");
            assert_eq!(r#"{{ fluent("fallback", lang="{lang}") }}"#, "this should fall back");
        }
    }

    /// Default lang argument works
    #[test]
    fn use_default_lang() {
        let loader = FluentLoader::new(&*super::LOCALES).with_default_lang("de".parse().unwrap());
        let mut minijinja = minijinja::Environment::default();
        minijinja.add_function("fluent", loader.into_minijinja_fn());
        let context = minijinja::context! {};
        assert_eq!(
            minijinja
                .render_str(r#"{{ fluent("hello-world") }}"#, &context)
                .unwrap(),
            "Hallo Welt!"
        );
        assert_eq!(
            minijinja
                .render_str(r#"{{ fluent("hello-world", lang="fr") }}"#, &context)
                .unwrap(),
            "Bonjour le monde!"
        );
    }

    /// Rendering fails when no default and no explicit lang argument is provided
    #[test]
    fn no_default_and_no_argument_error() {
        let loader = FluentLoader::new(&*super::LOCALES);
        let mut minijinja = minijinja::Environment::default();
        minijinja.add_function("fluent", loader.into_minijinja_fn());
        let context = minijinja::context! {};
        assert!(minijinja
            .render_str(r#"{{ fluent("hellow-world") }}"#, &context)
            .is_err());
    }
}
