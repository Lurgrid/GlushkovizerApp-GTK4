use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use glushkovizer::prelude::*;
use glushkovizer::{automata::Automata, regexp::RegExp};
use gtk::ScrolledWindow;
use gtk::{
    gdk::Texture, gdk_pixbuf::PixbufLoader, glib, template_callbacks, Align, Button,
    CompositeTemplate, Entry, FileDialog, FileFilter, Image, Stack, StackSwitcher, TextView,
};
use std::{
    cell::Cell,
    fmt::Display,
    fs::File,
    hash::Hash,
    io::{BufReader, Error, Result, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

macro_rules! error {
    ($error:expr, $x:expr) => {{
        $error.buffer().set_text($x.to_string().as_str());
        $error.set_visible(true);
        return;
    }};
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/sagbot/GlushkovApp/glushkovizer.ui")]
pub struct GlushkovizerApp {
    automata: Cell<Automata<'static, char, usize>>,
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub image: TemplateChild<Image>,
    #[template_child]
    pub error: TemplateChild<TextView>,
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

        self.error
            .bind_property("visible", &self.stack.get(), "visible")
            .invert_boolean()
            .bidirectional()
            .sync_create()
            .build();

        self.stack
            .bind_property("visible", &self.switcher.get(), "visible")
            .bidirectional()
            .sync_create()
            .build();

        self.next
            .bind_property("visible", &self.prev.get(), "visible")
            .bidirectional()
            .sync_create()
            .build();
    }
}

#[template_callbacks]
impl GlushkovizerApp {
    #[template_callback]
    fn handle_parse_clicked(&self, _: &Button) {
        let sr = self.entry.text().to_string();
        let r = match RegExp::try_from(sr) {
            Err(s) => {
                self.error.buffer().set_text(s.as_str());
                self.error.set_visible(true);
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
            Err(s) => error!(self.error, s),
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
        self.scroll_switcher
            .emit_scroll_child(gtk::ScrollType::PageLeft, true);
    }

    #[template_callback]
    async fn next_handle(&self, _: &Button) {
        self.scroll_switcher
            .emit_scroll_child(gtk::ScrollType::PageRight, false);
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
        let height = self.obj().height() - 110;
        let texture = match get_automata_texture(a, width, height) {
            Err(e) => error!(self.error, e),
            Ok(t) => t,
        };
        self.image.set_from_paintable(Some(&texture));
        self.image.set_size_request(width, height);
        self.stack.set_visible(true);

        let scc = a
            .extract_scc()
            .into_iter()
            .filter(|a| a.is_orbit())
            .collect::<Vec<_>>();

        for (ind, automata) in scc.into_iter().enumerate() {
            let texture = match get_automata_texture(&automata, width, height) {
                Err(e) => error!(self.error, e),
                Ok(t) => t,
            };
            let image = Image::from_paintable(Some(&texture));
            image.set_halign(Align::Fill);
            image.set_valign(Align::Fill);
            image.set_hexpand(true);
            image.set_vexpand(true);
            image.set_size_request(width, height);
            self.stack.add_titled(
                &image,
                Some(&format!("orbit{ind}")),
                &format!("Orbit N°{ind}"),
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
            Err(e) => error!(self.error, e),
            Ok(file) => {
                let mut path: PathBuf = file.path().unwrap();
                path.set_extension("json");
                let mut file = match File::create_new(path.clone()) {
                    Err(e) => error!(self.error, e),
                    Ok(f) => f,
                };
                match serde_json::to_string(unsafe { &*self.automata.as_ptr() }) {
                    Err(e) => error!(self.error, e),
                    Ok(json) => match file.write_all(json.as_bytes()) {
                        Err(e) => error!(self.error, e),
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
            Err(e) => error!(self.error, e),
            Ok(file) => {
                let path: PathBuf = file.path().unwrap();
                let file = match File::open(path) {
                    Err(e) => error!(self.error, e),
                    Ok(f) => f,
                };
                self.automata
                    .set(match serde_json::from_reader(BufReader::new(file)) {
                        Err(e) => error!(self.error, e),
                        Ok(a) => a,
                    })
            }
        };
        self.update();
    }
}

/// Renvoie une Texture représentant le graph, en cas d'erreur renvoie cette
/// erreur
fn get_automata_texture<'a, T, V>(
    a: &impl ToDot<'a, T, V>,
    width: i32,
    height: i32,
) -> Result<Texture>
where
    T: Eq + Hash + Display + Clone,
    V: Eq + Hash + Display + Clone,
{
    let svg = get_svg(
        a,
        gtk::Settings::default()
            .map(|s| s.is_gtk_application_prefer_dark_theme())
            .unwrap_or(false),
    )?;
    let loader = PixbufLoader::new();

    loader.set_size(width, height);
    loader
        .write(svg.as_bytes())
        .expect("Cannot write on the PixbufLoader");
    loader.close().expect("Cannot close the PixbufLoader");
    let pixbuf = loader
        .pixbuf()
        .expect("Cannot convert the PixbufLoader to Pixbuf");
    Ok(Texture::for_pixbuf(&pixbuf))
}

/// Renvoie la représentation de "g" en SVG en cas de succès, sinon en cas
/// d'erreur renvoie cette erreur.
fn get_svg<'a, T, V>(g: &impl ToDot<'a, T, V>, inverse: bool) -> Result<String>
where
    T: Eq + Hash + Display + Clone,
    V: Eq + Hash + Display + Clone,
{
    use std::io::ErrorKind;
    let mut c = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut inp) = c.stdin {
        inp.write_all(g.to_dot(inverse).unwrap().as_bytes())?;
    } else {
        return Err(Error::new(ErrorKind::Other, "No input"));
    }
    let output = c.wait_with_output()?;
    String::from_utf8(output.stdout)
        .map_err(|_| Error::new(ErrorKind::Other, "Not a valid utf-8 output"))
        .map(|s| {
            if inverse {
                s.replace("black", "white")
            } else {
                s
            }
        })
}

impl WidgetImpl for GlushkovizerApp {}

impl WindowImpl for GlushkovizerApp {}

impl ApplicationWindowImpl for GlushkovizerApp {}

impl AdwApplicationWindowImpl for GlushkovizerApp {}
