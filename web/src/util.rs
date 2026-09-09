/// Whole years elapsed between `birthday` and now (UTC), saturating at 0.
pub fn age_years(birthday: &jiff::Zoned) -> i16 {
    let today = jiff::Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::UTC)
        .date();
    today
        .since((jiff::Unit::Year, birthday.date()))
        .map(|span| span.get_years())
        .unwrap_or(0)
}

/// Folds the spelling variants Arabic writers use interchangeably, so that
/// searching for "احمد" also finds "أحمد".
pub fn normalize_search(text: &str) -> String {
    text.chars()
        .filter_map(|c| match c {
            'أ' | 'إ' | 'آ' | 'ٱ' => Some('ا'),
            'ى' => Some('ي'),
            'ة' => Some('ه'),
            'ؤ' => Some('و'),
            'ئ' => Some('ي'),
            // tatweel and the diacritics carry no meaning for matching
            'ـ' | '\u{064B}'..='\u{0652}' | '\u{0670}' => None,
            c => Some(c.to_ascii_lowercase()),
        })
        .collect()
}

/// True when every term in `query` shows up somewhere in `text`, so "احمد سالم"
/// matches a name that has both, in any order.
pub fn matches_search(query: &str, text: &str) -> bool {
    let text = normalize_search(text);

    normalize_search(query)
        .split_whitespace()
        .all(|term| text.contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_regardless_of_hamza_and_diacritics() {
        assert!(matches_search("احمد", "أَحْمَد العتيبي"));
        assert!(matches_search("علي", "عَلي"));
        assert!(matches_search("فاطمه", "فاطمة"));
    }

    #[test]
    fn matches_terms_in_any_order() {
        assert!(matches_search("العتيبي احمد", "أحمد بن سالم العتيبي"));
        assert!(!matches_search("احمد خالد", "أحمد بن سالم العتيبي"));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(matches_search("   ", "أحمد"));
    }
}
