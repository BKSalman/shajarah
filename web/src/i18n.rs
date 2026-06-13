use std::{borrow::Cow, fmt::Display};

use garde::i18n::{InvalidCreditCard, InvalidEmail, InvalidPhoneNumber, InvalidUrl, IpKind};

pub struct Arabic;

impl garde::I18n for Arabic {
    fn length_lower_than(&self, min: usize) -> Cow<'static, str> {
        Cow::Owned(format!("يجب أن يكون ُ{min} على الأقل"))
    }

    fn length_greater_than(&self, max: usize) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.length_greater_than(max)
    }

    fn range_lower_than(&self, min: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.range_lower_than(min)
    }

    fn range_greater_than(&self, max: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.range_greater_than(max)
    }

    fn credit_card_invalid(&self, reason: InvalidCreditCard) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.credit_card_invalid(reason)
    }

    fn pattern_no_match(&self, pattern: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.pattern_no_match(pattern)
    }

    fn contains_missing(&self, pattern: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.contains_missing(pattern)
    }

    fn url_invalid(&self, reason: InvalidUrl) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.url_invalid(reason)
    }

    fn prefix_missing(&self, pattern: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.prefix_missing(pattern)
    }

    fn suffix_missing(&self, pattern: &dyn Display) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.suffix_missing(pattern)
    }

    fn phone_number_invalid(&self, reason: InvalidPhoneNumber) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.phone_number_invalid(reason)
    }

    fn ip_invalid(&self, kind: IpKind) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.ip_invalid(kind)
    }

    fn matches_field_mismatch(&self, field: &dyn Display) -> Cow<'static, str> {
        let field = field.to_string();
        if field == "confirm_password" || field == "password" {
            Cow::Borrowed("كلمات المرور لا تتطابق")
        } else {
            garde::i18n::DefaultI18n.matches_field_mismatch(&field)
        }
    }

    fn email_invalid(&self, reason: InvalidEmail) -> Cow<'static, str> {
        Cow::Borrowed("البريد غير صالح")
    }

    fn ascii_invalid(&self) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.ascii_invalid()
    }

    fn alphanumeric_invalid(&self) -> Cow<'static, str> {
        garde::i18n::DefaultI18n.alphanumeric_invalid()
    }

    fn required_not_set(&self) -> Cow<'static, str> {
        Cow::Borrowed("هذا الحقل مطلوب")
    }
}
