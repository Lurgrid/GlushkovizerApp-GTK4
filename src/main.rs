#![windows_subsystem = "windows"]
mod glushkovizerapp;
mod utils;
use glushkovizerapp::GlushkovizerApp;
use gtk::{glib, prelude::*};

const APP_ID: &str = "com.sagbot.GlushkovizerApp";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let glush = GlushkovizerApp::new(app);
    glush.present();
}
