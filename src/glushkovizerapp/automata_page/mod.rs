mod imp;

use adw::subclass::prelude::*;
use glushkovizer::automata::SubAutomata;
use gtk::glib;
use std::io::Result;

glib::wrapper! {
    pub struct AutomataPage(ObjectSubclass<imp::AutomataPage>) @extends gtk::Box, gtk::Widget;
}

impl AutomataPage {
    pub fn new(
        automata: SubAutomata<'static, char, usize>,
        width: i32,
        height: i32,
    ) -> Result<Self> {
        let page: Self = glib::Object::new();
        let inner: &imp::AutomataPage = imp::AutomataPage::from_obj(&page);
        inner.set_automata(automata, width, height).map(|_| page)
    }
}
