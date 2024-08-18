// From AOSC-Dev/atm

use anyhow::Result;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester, LanguageLoader,
};
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::I18N_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::I18N_LOADER, $message_id, $($args), *)
    }};
}

lazy_static! {
    pub static ref I18N_LOADER: FluentLanguageLoader =
        load_i18n().expect("Unable to load i18n strings.");
}

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

fn load_i18n() -> Result<FluentLanguageLoader> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let fallback_language: Vec<LanguageIdentifier> = vec!["en-US".parse().unwrap()];
    let languages: Vec<LanguageIdentifier> = requested_languages
        .into_iter()
        .chain(fallback_language)
        .collect();
    language_loader.load_languages(&Localizations, &languages)?;

    Ok(language_loader)
}
