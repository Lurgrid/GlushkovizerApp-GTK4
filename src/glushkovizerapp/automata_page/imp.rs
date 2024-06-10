use crate::utils::get_automata_texture;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use glushkovizer::automata::SubAutomata;
use gtk::{glib, template_callbacks, CompositeTemplate, Image};
use std::{cell::Cell, io::Result};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/sagbot/GlushkovApp/automata_page.ui")]
pub struct AutomataPage {
    automata: Cell<Option<SubAutomata<'static, char, usize>>>,
    #[template_child]
    pub image: TemplateChild<Image>,
}

#[glib::object_subclass]
impl ObjectSubclass for AutomataPage {
    const NAME: &'static str = "AutomataPage";
    type Type = super::AutomataPage;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl AutomataPage {
    pub fn set_automata(
        &self,
        automata: SubAutomata<'static, char, usize>,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let texture = get_automata_texture(&automata, width, height)?;
        self.automata.set(Some(automata));
        self.image.set_from_paintable(Some(&texture));
        self.image.set_size_request(width, height);
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
