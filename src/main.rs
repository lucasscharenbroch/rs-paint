mod image;
mod ui;
mod transformable;
mod shape;
mod geometry;
mod clipboard;
mod cli;

use clap::Parser;

const SEMANTIC_VERSION: &'static str = "1.0";
const PROGRAM_NAME: &'static str = "RS-Paint";
const PROGRAM_DESCRIPTION: &'static str = "A lightweight image editor, written in Rust using GTK4.";

fn main() -> gtk::glib::ExitCode {
    let cli_settings = cli::CliSettings::parse();

    gtk::init().expect("Failed to initialize gtk");
    ui::UiState::run_main_ui(cli_settings)
}
