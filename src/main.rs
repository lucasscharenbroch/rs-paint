mod image;
mod ui;

fn main() -> gtk::glib::ExitCode {
    gtk::init().expect("Failed to initialize gtk");
    ui::UiState::run_main_ui()
}
