use crate::reference::Locale;

pub fn text(locale: Locale, fr: &'static str, en: &'static str) -> &'static str {
    match locale {
        Locale::Fr => fr,
        Locale::En => en,
    }
}
