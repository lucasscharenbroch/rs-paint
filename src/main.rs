mod image;
mod ui;

fn main() -> gtk::glib::ExitCode {
    gtk::init();
    ui::UiState::run_main_ui()
}
