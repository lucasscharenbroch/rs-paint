mod image;
mod ui;
mod transformable;
mod shape;

fn main() -> gtk::glib::ExitCode {
    gtk::init().expect("Failed to initialize gtk");
    ui::UiState::run_main_ui()
}
