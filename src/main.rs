mod app;
mod clipboard;
mod error;
mod ocr;
mod screenshot;
mod text_extractor;

fn main() -> gtk::glib::ExitCode {
    app::run()
}
