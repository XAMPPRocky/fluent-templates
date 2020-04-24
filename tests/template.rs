use handlebars::*;
use fluent_template_helper::*;

static_loader!(load, "./tests/locales", "en-US", core: "./tests/locales/core.ftl", customizer: |bundle| {
    bundle.set_use_isolating(false)
});

use serde_json::json;

/// Generates tests for each loader in different locales.
macro_rules! generate_tests {
    ($(fn $locale_test_fn:ident ($locale:expr) {
        $($assert_macro:ident ! ( $lhs:expr , $rhs:expr ) );* $(;)?
    })+) => {
        $(
            #[test]
            fn $locale_test_fn() {
                let data = json!({"lang": $locale});
                // Test the static loader.
                {
                    let mut handlebars = Handlebars::new();
                    handlebars.register_helper("fluent", Box::new(FluentHelper::new(load())));
                    $(
                        $assert_macro ! (
                            handlebars
                            .render_template($lhs, &data)
                            .unwrap(),
                            $rhs
                        );
                    )*
                }
                // Test the arc loader.
                {
                    let mut handlebars = Handlebars::new();
                    let loader = ArcLoader::new("./tests/locales", unic_langid::langid!("en-US"))
                        .shared_resources(Some(&["./tests/locales/core.ftl".into()]))
                        .customize(|bundle| bundle.set_use_isolating(false))
                        .build()
                        .unwrap();
                    handlebars.register_helper("fluent", Box::new(FluentHelper::new(loader)));
                    $(
                        $assert_macro ! (
                            handlebars
                            .render_template($lhs, &data)
                            .unwrap(),
                            $rhs
                        );
                    )*
                }
            }
        )+
    };
}

generate_tests! {
    fn english("en-US") {
        assert_eq!(r#"{{fluent "simple"}}"#, "simple text");
        assert_eq!(r#"{{fluent "reference"}}"#, "simple text with a reference: foo");
        assert_eq!(r#"{{fluent "parameter" param="PARAM"}}"#, "text with a PARAM");
        assert_eq!(r#"{{fluent "parameter2" param1="P1" param2="P2"}}"#, "text one P1 second P2");
        assert_eq!(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, "text with a blah blah");
        assert_eq!(r#"{{#fluent "parameter2"}}{{#fluentparam "param1"}}foo{{/fluentparam}}{{#fluentparam "param2"}}bar{{/fluentparam}}{{/fluent}}"#, "text one foo second bar");
        assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");
    }

    fn french("fr") {
        assert_eq!(r#"{{fluent "simple"}}"#, "texte simple");
        assert_eq!(r#"{{fluent "reference"}}"#, "texte simple avec une référence: foo");
        assert_eq!(r#"{{fluent "parameter" param="PARAM"}}"#, "texte avec une PARAM");
        assert_eq!(r#"{{fluent "parameter2" param1="P1" param2="P2"}}"#, "texte une P1 seconde P2");
        assert_eq!(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, "texte avec une blah blah");
        assert_eq!(r#"{{#fluent "parameter2"}}{{#fluentparam "param1"}}foo{{/fluentparam}}{{#fluentparam "param2"}}bar{{/fluentparam}}{{/fluent}}"#, "texte une foo seconde bar");
        assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");

    }

    fn chinese("zh-TW") {
        assert_eq!(r#"{{fluent "exists"}}"#, "兒");
        assert_eq!(r#"{{fluent "fallback-zh"}}"#, "气");
        assert_eq!(r#"{{fluent "fallback"}}"#, "this should fall back");
    }
}
