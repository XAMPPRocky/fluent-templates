use handlebars::*;
use handlebars_fluent::*;

simple_loader!(load, "./tests/locales", "en-US", core: "./tests/locales/core.ftl");

use serde_json::json;
#[test]
fn test_helper() {
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("fluent", Box::new(FluentHelper::new(load())));
    let eng = json!({"lang": "en-US"});
    assert_eq!(
        handlebars.render_template("{{fluent \"simple\"}}", &eng).unwrap(),
        "simple text"
    );
}
