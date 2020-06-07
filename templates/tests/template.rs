#![allow(unused)]

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./templates/tests/locales",
        fallback_language: "en-US",
        core_locales: "./templates/tests/locales/core.ftl",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fluent_templates::static_loader! {
    static _LOCALES = {
        locales: "./templates/tests/locales",
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

        (
            FluentLoader::new(crate::LOCALES.clone()),
            FluentLoader::new(arc),
        )
    }};
}

/// Generates tests for each loader in different locales.
macro_rules! generate_tests {
    ($(fn $locale_test_fn:ident ($template_engine:ident, $locale:expr) {
        $($assert_macro:ident ! ( $lhs:expr , $rhs:expr ) );* $(;)?
    })+) => {
        use serde_json::json;
        use fluent_templates::*;
        $(
            #[test]
            fn $locale_test_fn() {
                let data = json!({"lang": $locale});

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

            }
        )+
    };
}

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
}
