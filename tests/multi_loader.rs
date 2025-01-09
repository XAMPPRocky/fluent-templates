use std::ops::Deref;

use fluent_templates::{ArcLoader, Loader, MultiLoader};
use unic_langid::{langid, LanguageIdentifier};

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

#[test]
fn check_if_multiloader_works() {
    const US_ENGLISH: LanguageIdentifier = langid!("en-US");
    const CHINESE: LanguageIdentifier = langid!("zh-CN");

    let en_loader = ArcLoader::builder("./tests/locales", US_ENGLISH)
        .customize(|bundle| bundle.set_use_isolating(false))
        .build()
        .unwrap();
    let cn_loader = ArcLoader::builder("./tests/locales", CHINESE)
        .customize(|bundle| bundle.set_use_isolating(false))
        .build()
        .unwrap();

    let multiloader = MultiLoader::from_iter([
        Box::new(LOCALES.deref()) as Box<dyn Loader>,
        Box::new(en_loader) as Box<dyn Loader>,
        Box::new(cn_loader) as Box<dyn Loader>,
    ]);

    assert_eq!(
        "Hello World!",
        multiloader.lookup(&US_ENGLISH, "hello-world")
    );
    assert_eq!("儿", multiloader.lookup(&CHINESE, "exists"));
}
