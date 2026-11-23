//! Kern des Daedyx Launchers: Quelle, Download, Zustand, Installation. Ohne Oberfläche.

pub mod config;
pub mod download;
pub mod error;
pub mod installer;
pub mod jobs;
pub mod launch;
pub mod locate;
pub mod lock;
pub mod paths;
pub mod probe;
pub mod selfupdate;
pub mod service;
pub mod source;
pub mod state;
pub mod sync;
pub mod system;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod download_tests;
#[cfg(test)]
mod jobs_tests;
#[cfg(test)]
mod launch_tests;
#[cfg(test)]
mod lock_tests;
#[cfg(test)]
mod paths_tests;
#[cfg(test)]
mod selfupdate_tests;
#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod test_support;
