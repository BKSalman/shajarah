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
