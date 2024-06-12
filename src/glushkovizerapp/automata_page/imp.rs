use crate::utils::get_automata_texture;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::AlertDialog;
use glib::subclass::InitializingObject;
use glushkovizer::automata::{DoorType, SubAutomata};
use glushkovizer::prelude::*;
use gtk::{glib, template_callbacks, Button, CompositeTemplate, Image, Label};
use std::{
    cell::{Cell, UnsafeCell},
    collections::HashSet,
    io::Result,
    mem::{transmute, MaybeUninit},
};

#[derive(CompositeTemplate)]
#[template(resource = "/com/sagbot/GlushkovApp/automata_page.ui")]
pub struct AutomataPage {
    automata: UnsafeCell<SubAutomata<'static, char, usize>>,
    width: Cell<i32>,
    height: Cell<i32>,
    #[template_child]
    pub image: TemplateChild<Image>,
    #[template_child]
    pub next: TemplateChild<Button>,
    #[template_child]
    pub transverse: TemplateChild<Label>,
    #[template_child]
    pub stable: TemplateChild<Label>,
}

impl Default for AutomataPage {
    fn default() -> Self {
        Self {
            automata: UnsafeCell::new(unsafe {
                transmute(MaybeUninit::<SubAutomata<char, usize>>::uninit())
            }),
            width: Default::default(),
            height: Default::default(),
            image: Default::default(),
            next: Default::default(),
            transverse: Default::default(),
            stable: Default::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AutomataPage {
    const NAME: &'static str = "AutomataPage";
    type Type = super::AutomataPage;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[template_callbacks]
impl AutomataPage {
    #[template_callback]
    pub fn next_handler_clicked(&self, _: &Button) {
        let automata = unsafe { &*self.automata.get() };

        automata.kosaraju_type().into_iter().for_each(|doort| {
            let mut inp = HashSet::new();
            let mut out = HashSet::new();
            doort.into_iter().for_each(|(rs, dtype)| match dtype {
                DoorType::None => (),
                DoorType::In => {
                    inp.insert(rs);
                }
                DoorType::Out => {
                    out.insert(rs);
                }
                DoorType::Both => {
                    out.insert(rs.clone());
                    inp.insert(rs);
                }
            });

            out.into_iter().for_each(|output| {
                automata
                    .get_follows(&output)
                    .unwrap()
                    .into_iter()
                    .for_each(|(symbol, set)| {
                        set.into_iter().for_each(|to| {
                            let _ = automata.remove_transition(&output, &to, &symbol);
                        });
                    });
            });
        });

        if let Err(e) = self.update() {
            let window = self
                .obj()
                .ancestor(crate::glushkovizerapp::GlushkovizerApp::static_type())
                .expect("Failed to retrieve the GlushkovizerApp window");

            AlertDialog::builder()
                .title("An error has occurred")
                .body(e.to_string())
                .can_close(true)
                .build()
                .present(&window);
        }
    }
}

impl AutomataPage {
    pub fn set_automata(
        &self,
        automata: SubAutomata<'static, char, usize>,
        width: i32,
        height: i32,
    ) -> Result<()> {
        unsafe { self.automata.get().write(automata) };
        self.width.set(width);
        self.height.set(height);
        self.update()?;
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        let width = self.width.get();
        let height = self.height.get();
        let automata = unsafe { &*self.automata.get() };
        let texture = get_automata_texture(automata, width, height)?;
        self.image.set_from_paintable(Some(&texture));
        self.image.set_size_request(width, height);

        let css = automata.extract_scc();
        let transerve = css.iter().all(|sub| {
            if !sub.is_orbit() {
                return true;
            }
            sub.is_transverse()
        });
        let stable = css.iter().all(|sub| {
            if !sub.is_orbit() {
                return true;
            }
            sub.is_stable()
        });
        self.transverse
            .set_text(if transerve { "Yes" } else { "No" });
        self.stable.set_text(if stable { "Yes" } else { "No" });
        self.next
            .set_sensitive(css.into_iter().find(|sub| sub.is_orbit()).is_some());
        Ok(())
    }
}

impl ObjectImpl for AutomataPage {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for AutomataPage {}
impl BoxImpl for AutomataPage {}
