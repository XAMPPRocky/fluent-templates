use handlebars::*;
use handlebars_fluent::*;

simple_loader!(load, "./tests/locales", "en-US", core: "./tests/locales/core.ftl", customizer: |_bundle| {});

use serde_json::json;

#[test]
fn test_english() {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentHelper::new(load())));
    let data = json!({"lang": "en-US"});
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "simple"}}"#, &data)
            .unwrap(),
        "simple text"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "reference"}}"#, &data)
            .unwrap(),
        "simple text with a reference: foo"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "parameter" param="PARAM"}}"#, &data)
            .unwrap(),
        "text with a PARAM"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "parameter2" param1="P1" param2="P2"}}"#, &data)
            .unwrap(),
        "text one P1 second P2"
    );
    assert_eq!(
        handlebars.render_template(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, &data).unwrap(),
        "text with a blah blah"
    );
    assert_eq!(
        handlebars.render_template(r#"{{#fluent "parameter2"}}{{#fluentparam "param1"}}foo{{/fluentparam}}{{#fluentparam "param2"}}bar{{/fluentparam}}{{/fluent}}"#, &data).unwrap(),
        "text one foo second bar"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "fallback"}}"#, &data)
            .unwrap(),
        "this should fall back"
    );
}

#[test]
fn test_french() {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentHelper::new(load())));
    let data = json!({"lang": "fr"});
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "simple"}}"#, &data)
            .unwrap(),
        "texte simple"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "reference"}}"#, &data)
            .unwrap(),
        "texte simple avec une référence: foo"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "parameter" param="PARAM"}}"#, &data)
            .unwrap(),
        "texte avec une PARAM"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "parameter2" param1="P1" param2="P2"}}"#, &data)
            .unwrap(),
        "texte une P1 seconde P2"
    );
    assert_eq!(
        handlebars.render_template(r#"{{#fluent "parameter"}}{{#fluentparam "param"}}blah blah{{/fluentparam}}{{/fluent}}"#, &data).unwrap(),
        "texte avec une blah blah"
    );
    assert_eq!(
        handlebars.render_template(r#"{{#fluent "parameter2"}}{{#fluentparam "param1"}}foo{{/fluentparam}}{{#fluentparam "param2"}}bar{{/fluentparam}}{{/fluent}}"#, &data).unwrap(),
        "texte une foo seconde bar"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "fallback"}}"#, &data)
            .unwrap(),
        "this should fall back"
    );
}

#[test]
fn test_chinese() {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentHelper::new(load())));
    let data = json!({"lang": "zh-TW"});
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "exists"}}"#, &data)
            .unwrap(),
        "兒"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "fallback-zh"}}"#, &data)
            .unwrap(),
        "气"
    );
    assert_eq!(
        handlebars
            .render_template(r#"{{fluent "fallback"}}"#, &data)
            .unwrap(),
        "this should fall back"
    );
}
