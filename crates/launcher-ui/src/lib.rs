//! Oberfläche des Daedyx Launchers (egui). Zustand und Darstellung sind getrennt, damit beides testbar ist.

pub mod app;
mod app_dialogs;
pub mod app_settings;
pub mod autostart;
pub mod brand;
pub mod card_view;
pub mod chips;
pub mod chrome;
pub mod errors;
pub mod filter;
pub mod fonts;
pub mod i18n;
pub mod model;
pub mod notice_view;
pub mod pages;
pub mod palette;
pub mod row_view;
pub mod section_view;
pub mod settings;
pub mod settings_view;
pub mod shell;
pub mod theme;
pub mod update_banner;
pub mod widgets;

#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod autostart_tests;
#[cfg(test)]
mod brand_tests;
#[cfg(test)]
mod card_view_tests;
#[cfg(test)]
mod errors_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod i18n_tests;
#[cfg(test)]
mod model_tests;
#[cfg(test)]
mod palette_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod widgets_tests;
