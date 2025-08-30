pub struct Arabic;

impl garde::I18n for Arabic {
    fn length_lower_than(&self, min: usize) -> String {
        format!("يجب أن يكون ُ{min} على الأقل")
    }

    fn length_greater_than(&self, max: usize) -> String {
        garde::i18n::DefaultI18n.length_greater_than(max)
    }

    fn range_lower_than(&self, min: &str) -> String {
        garde::i18n::DefaultI18n.range_lower_than(min)
    }

    fn range_greater_than(&self, max: &str) -> String {
        garde::i18n::DefaultI18n.range_greater_than(max)
    }

    fn credit_card_invalid(&self, error: &str) -> String {
        garde::i18n::DefaultI18n.credit_card_invalid(error)
    }

    fn pattern_no_match(&self, pattern: &str) -> String {
        garde::i18n::DefaultI18n.pattern_no_match(pattern)
    }

    fn contains_missing(&self, pattern: &str) -> String {
        garde::i18n::DefaultI18n.contains_missing(pattern)
    }

    fn url_invalid(&self, error: &str) -> String {
        garde::i18n::DefaultI18n.url_invalid(error)
    }

    fn prefix_missing(&self, pattern: &str) -> String {
        garde::i18n::DefaultI18n.prefix_missing(pattern)
    }

    fn suffix_missing(&self, pattern: &str) -> String {
        garde::i18n::DefaultI18n.suffix_missing(pattern)
    }

    fn phone_number_invalid(&self) -> String {
        garde::i18n::DefaultI18n.phone_number_invalid()
    }

    fn phone_number_invalid_with_error(&self, error: &str) -> String {
        garde::i18n::DefaultI18n.phone_number_invalid_with_error(error)
    }

    fn ip_invalid(&self, kind: &str) -> String {
        garde::i18n::DefaultI18n.ip_invalid(kind)
    }

    fn matches_field_mismatch(&self, field: &str) -> String {
        if field == "confirm_password" || field == "password" {
            String::from("كلمات المرور لا تتطابق")
        } else {
            garde::i18n::DefaultI18n.matches_field_mismatch(field)
        }
    }

    fn email_invalid(&self, _error: &str) -> String {
        String::from("البريد غير صالح")
    }

    fn ascii_invalid(&self) -> String {
        garde::i18n::DefaultI18n.ascii_invalid()
    }

    fn alphanumeric_invalid(&self) -> String {
        garde::i18n::DefaultI18n.alphanumeric_invalid()
    }

    fn required_not_set(&self) -> String {
        String::from("هذا الحقل مطلوب")
    }
}
