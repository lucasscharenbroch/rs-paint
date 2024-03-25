mod image;
mod ui;
mod undo;

fn main() -> gtk::glib::ExitCode {
    gtk::init();
    ui::UiState::run_main_ui()
}
