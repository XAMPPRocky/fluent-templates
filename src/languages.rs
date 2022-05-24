use unic_langid::LanguageIdentifier;

/// This is taken from fluent_langneg, but changed to return a list of language that match the available languages sorted by specificity
pub fn filter_matches<'a, R: 'a + AsRef<LanguageIdentifier>, A: 'a + AsRef<LanguageIdentifier>>(
    requested: &[R],
    available: &'a [A],
) -> Vec<&'a A> {
    let mut supported_locales = vec![];

    let mut available_locales: Vec<&A> = available.iter().collect();

    for req in requested {
        let req = req.as_ref().to_owned();
        macro_rules! test_strategy {
            ($self_as_range:expr, $other_as_range:expr) => {{
                let mut match_found = false;
                available_locales.retain(|locale| {
                    if locale
                        .as_ref()
                        .matches(&req, $self_as_range, $other_as_range)
                    {
                        match_found = true;
                        supported_locales.push(*locale);
                        return false;
                    }
                    true
                });
            }};
        }

        // 1) Try to find a simple (case-insensitive) string match for the request.
        test_strategy!(false, false);

        // 2) Try to match against the available locales treated as ranges.
        test_strategy!(true, false);

        // Per Unicode TR35, 4.4 Locale Matching, we don't add likely subtags to
        // requested locales, so we'll skip it from the rest of the steps.
        if req.language.is_empty() {
            continue;
        }
    }

    supported_locales.sort_by(|x, y| {
        let x_specificity = into_specificity(x.as_ref());
        let y_specificity = into_specificity(y.as_ref());
        x_specificity.cmp(&y_specificity).reverse()
    });

    supported_locales
}

fn into_specificity(lang: &LanguageIdentifier) -> usize {
    // let parts = lang.into_parts();
    let mut specificity = 0;
    // Script
    if lang.script.is_some() {
        specificity += 1;
    }
    // Region
    if lang.region.is_some() {
        specificity += 1;
    }

    // variant
    specificity += lang.variants().len();

    specificity
}

pub fn negotiate_languages<
    'a,
    R: 'a + AsRef<LanguageIdentifier>,
    A: 'a + AsRef<LanguageIdentifier> + PartialEq,
>(
    requested: &[R],
    available: &'a [A],
    default: Option<&'a A>,
) -> Vec<&'a A> {
    let mut supported = filter_matches(requested, available);

    if let Some(default) = default {
        if !supported.contains(&default) {
            supported.push(default);
        }
    }
    supported
}

#[cfg(test)]
mod test {
    use super::*;
    use fluent_langneg::convert_vec_str_to_langids;

    macro_rules! test_negotiation {
        ($input:expr , $valid:expr => $output:expr) => {
            assert_eq!(
                filter_matches(
                    &convert_vec_str_to_langids($input).expect("Input"),
                    &convert_vec_str_to_langids($valid).expect("Valid")
                ),
                convert_vec_str_to_langids($output)
                    .expect("result")
                    .iter()
                    .collect::<Vec<_>>()
            );
        };
    }
    #[test]
    fn test_hirarchy() {
        test_negotiation!(["de"], ["de", "de-DE-1996", "en-US", "de-DE", "de-CH"] =>  ["de"]);

        test_negotiation!(["de-DE"], ["de", "de-DE-1996", "en-US", "de-DE", "de-CH"] => ["de-DE", "de"]);

        test_negotiation!(["de-CH"], ["de", "de-DE-1996", "en-US", "de-DE", "de-CH"] => ["de-CH", "de"]);

        test_negotiation!(["de-DE-1996"], ["de", "de-DE-1996", "en-US", "de-DE", "de-CH"] => ["de-DE-1996", "de-DE", "de"]);
    }

    #[test]
    fn test_negotiate_languages() {
        assert_eq!(
            negotiate_languages(
                &convert_vec_str_to_langids(["de-DE"]).unwrap(),
                &convert_vec_str_to_langids(["de-DE", "de", "en-US", "de-CH"],).unwrap(),
                None,
            ),
            convert_vec_str_to_langids(["de-DE", "de"])
                .expect("result")
                .iter()
                .collect::<Vec<_>>()
        );
    }
}
