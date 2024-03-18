mod image;
mod ui;

fn main() -> gtk::glib::ExitCode {
    let mut ui_state = ui::UiState::new();
    ui_state.run()
}
