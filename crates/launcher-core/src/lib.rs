//! Kern des Daedyx Launchers: feste App-Liste, Update-Suche und Download über die gemeinsame Update-Bibliothek
//! (`daedyx-update`, GitHub-Nachweis), Zustand, Installation. Ohne Oberfläche.

pub mod apps;
pub mod config;
pub mod error;
pub mod held;
pub mod install;
pub mod installer;
pub mod jobs;
pub mod launch;
pub mod lineup;
pub mod locate;
pub mod paths;
pub mod probe;
/// Installationsprobe ohne Nachweis, nur mit Merkmal `probe` (Review U3 F-U3-2).
#[cfg(feature = "probe")]
pub mod probe_local;
pub mod remote;
pub mod selfupdate;
pub mod service;
pub mod state;
pub mod sync;
pub mod system;

/// Die Kern-Bibliothek in genau der Fassung, die dieser Launcher benutzt (Oberfläche und Kommandozeile tragen keine
/// zweite Kern-Quelle ein).
pub use daedyx_update;

#[cfg(test)]
mod apps_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod held_tests;
#[cfg(test)]
mod jobs_tests;
#[cfg(test)]
mod launch_tests;
#[cfg(test)]
mod lineup_tests;
#[cfg(test)]
mod paths_tests;
#[cfg(test)]
mod probe_guard_tests;
#[cfg(all(test, feature = "probe"))]
mod probe_local_tests;
#[cfg(test)]
mod selfupdate_tests;
#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod test_server;
#[cfg(test)]
mod test_support;
