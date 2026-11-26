#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use clap::Parser;
use eframe::egui;
use launcher_core::config::build_launcher;
use launcher_core::jobs::{Wake, Worker};
use launcher_core::paths::default_data_dir;
use launcher_ui::app::LauncherApp;
use launcher_ui::i18n::{system_locale, Texts};
use launcher_ui::settings::UiSettings;
use launcher_ui::settings_view::AppInfo;
use launcher_ui::theme;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "daedyx-launcher", version, about = "Daedyx Launcher")]
struct Args {
    /// Datenordner des Launchers
    #[arg(long, env = "DAEDYX_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// Start beim Anmelden: Fenster minimiert
    #[arg(long)]
    autostart: bool,
}

fn data_dir(args: &Args) -> PathBuf {
    args.data_dir.clone().unwrap_or_else(default_data_dir)
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let settings = UiSettings::load(&data_dir(&args));
    let t = Texts::for_language(settings.language.resolve(system_locale().as_deref()));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(t.t("app.title"))
            .with_inner_size([940.0, 640.0])
            .with_min_inner_size([520.0, 420.0])
            .with_icon(egui::IconData {
                rgba: launcher_ui::brand::render_rgba(64),
                width: 64,
                height: 64,
            }),
        ..Default::default()
    };
    eframe::run_native(
        &t.t("app.title"),
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx, settings.theme);
            if args.autostart {
                // Start beim Anmelden: still im Hintergrund, nicht mitten ins Bild
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
            let dir = data_dir(&args);
            let ctx = cc.egui_ctx.clone();
            let wake: Wake = Arc::new(move || ctx.request_repaint());
            // Feste Update-Quellen mit GitHub-Nachweis (U3): nichts einzustellen, also kein Fehler beim Start.
            let launcher = build_launcher(dir.clone(), env!("CARGO_PKG_VERSION"));
            let worker = Worker::start(launcher, wake);
            let info = AppInfo { data_dir: dir };
            Ok(Box::new(
                LauncherApp::new(t, Ok(worker)).with_settings(info, settings),
            ))
        }),
    )
}
