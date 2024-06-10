use super::automata_page::AutomataPage;
use crate::utils::get_automata_texture;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::AlertDialog;
use glib::subclass::InitializingObject;
use glushkovizer::prelude::*;
use glushkovizer::{automata::Automata, regexp::RegExp};
use gtk::{
    glib, template_callbacks, Button, CompositeTemplate, Entry, FileDialog, FileFilter, Image,
    ScrolledWindow, Stack, StackSwitcher,
};
use std::{
    cell::Cell,
    fmt::Display,
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/sagbot/GlushkovApp/glushkovizer.ui")]
pub struct GlushkovizerApp {
    automata: Cell<Automata<'static, char, usize>>,
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub image: TemplateChild<Image>,
    #[template_child]
    pub scroll_switcher: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub stack: TemplateChild<Stack>,
    #[template_child]
    pub switcher: TemplateChild<StackSwitcher>,
    #[template_child]
    pub prev: TemplateChild<Button>,
    #[template_child]
    pub next: TemplateChild<Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for GlushkovizerApp {
    const NAME: &'static str = "GlushkovizerApp";
    type Type = super::GlushkovizerApp;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
        klass.install_action("win.save", None, |window, _, _| {
            let window = window.clone();
            glib::MainContext::default().spawn_local(async move { window.imp().save().await });
        });

        klass.install_action("win.open", None, |window, _, _| {
            let window = window.clone();
            glib::MainContext::default().spawn_local(async move { window.imp().import().await });
        });
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for GlushkovizerApp {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

#[template_callbacks]
impl GlushkovizerApp {
    #[template_callback]
    fn handle_parse_clicked(&self, _: &Button) {
        let sr = self.entry.text().to_string();
        let r = match RegExp::try_from(sr) {
            Err(s) => {
                self.error_handler(&s);
                return;
            }
            Ok(r) => r,
        };
        self.automata.set(Automata::from(r));
        self.update();
    }

    #[template_callback]
    fn handle_entry_activate(&self, _: &Entry) {
        let sr = self.entry.text().to_string();
        let r = match RegExp::try_from(sr) {
            Err(s) => {
                self.error_handler(&s);
                return;
            }
            Ok(r) => r,
        };
        self.automata.set(Automata::from(r));
        self.update();
    }

    #[template_callback]
    async fn handle_import_clicked(&self, _: &Button) {
        self.import().await;
    }

    #[template_callback]
    async fn handle_save_clicked(&self, _: &Button) {
        self.save().await;
    }

    #[template_callback]
    async fn prev_handle(&self, _: &Button) {
        let adj = self.scroll_switcher.hadjustment();
        adj.set_value(adj.value() - self.scroll_switcher.width() as f64);
    }

    #[template_callback]
    async fn next_handle(&self, _: &Button) {
        let adj = self.scroll_switcher.hadjustment();
        adj.set_value(adj.value() + self.scroll_switcher.width() as f64);
    }
}

impl GlushkovizerApp {
    fn update(&self) {
        let pauto = unsafe { self.stack.first_child().unwrap_unchecked() };
        while let Some(child) = pauto.next_sibling() {
            self.stack.remove(&child)
        }
        self.stack.set_visible_child(&pauto);

        let a = unsafe { &*self.automata.as_ptr() };
        let width = self.stack.width();
        let height = self.stack.height();
        let texture = match get_automata_texture(a, width, height) {
            Err(e) => {
                self.error_handler(&e);
                return;
            }
            Ok(t) => t,
        };
        self.image.set_from_paintable(Some(&texture));
        self.image.set_size_request(width, height);

        let scc = a
            .extract_scc()
            .into_iter()
            .filter(|a| a.is_orbit())
            .collect::<Vec<_>>();

        for (ind, automata) in scc.into_iter().enumerate() {
            let auto = match AutomataPage::new(automata, width, height) {
                Err(e) => {
                    self.error_handler(&e);
                    return;
                }
                Ok(t) => t,
            };
            self.stack.add_titled(
                &ScrolledWindow::builder().child(&auto).build(),
                Some(&format!("orbit{}", ind + 1)),
                &format!("Orbit N°{}", ind + 1),
            );
        }
    }

    async fn save(&self) {
        let d = FileDialog::builder()
            .title("Choose a file to save the automata")
            .modal(true)
            .build();
        let file = d.save_future(Some(self.obj().as_ref())).await;
        match file {
            Err(e) => {
                self.error_handler(&e);
                return;
            }
            Ok(file) => {
                let mut path: PathBuf = file.path().unwrap();
                path.set_extension("json");
                let mut file = match File::create_new(path.clone()) {
                    Err(e) => {
                        self.error_handler(&e);
                        return;
                    }
                    Ok(f) => f,
                };
                match serde_json::to_string(unsafe { &*self.automata.as_ptr() }) {
                    Err(e) => {
                        self.error_handler(&e);
                        return;
                    }
                    Ok(json) => match file.write_all(json.as_bytes()) {
                        Err(e) => {
                            self.error_handler(&e);
                            return;
                        }
                        Ok(_) => (),
                    },
                }
            }
        };
    }

    async fn import(&self) {
        let f = FileFilter::new();
        f.add_suffix("json");
        let d = FileDialog::builder()
            .title("Choose a automata file")
            .default_filter(&f)
            .modal(true)
            .build();
        let file = d.open_future(Some(self.obj().as_ref())).await;
        match file {
            Err(e) => {
                self.error_handler(&e);
                return;
            }
            Ok(file) => {
                let path: PathBuf = file.path().unwrap();
                let file = match File::open(path) {
                    Err(e) => {
                        self.error_handler(&e);
                        return;
                    }
                    Ok(f) => f,
                };
                self.automata
                    .set(match serde_json::from_reader(BufReader::new(file)) {
                        Err(e) => {
                            self.error_handler(&e);
                            return;
                        }
                        Ok(a) => a,
                    })
            }
        };
        self.update();
    }

    fn error_handler(&self, error: &impl Display) {
        AlertDialog::builder()
            .title("You have an error in your regular expression syntax !")
            .body(error.to_string())
            .can_close(true)
            .build()
            .present(self.obj().upcast_ref::<adw::ApplicationWindow>());
    }
}

impl WidgetImpl for GlushkovizerApp {}
impl WindowImpl for GlushkovizerApp {}
impl ApplicationWindowImpl for GlushkovizerApp {}
impl AdwApplicationWindowImpl for GlushkovizerApp {}
