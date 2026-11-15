//! Gemeinsame Prüfbausteine für Launcher und Apps: Katalog, Versionen, Prüfsumme, Signatur.

pub mod catalog;
pub mod catalog_edit;
pub mod error;
pub mod verify;
pub mod version;

#[cfg(test)]
mod catalog_edit_tests;
#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod verify_tests;
#[cfg(test)]
mod version_tests;
