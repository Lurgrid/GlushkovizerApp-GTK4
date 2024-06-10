use adw::prelude::*;
use glushkovizer::prelude::*;
use gtk::{gdk::Texture, gdk_pixbuf::PixbufLoader};
use std::{
    fmt::Display,
    hash::Hash,
    io::{Error, Result, Write},
    process::{Command, Stdio},
};

/// Renvoie une Texture représentant le graph, en cas d'erreur renvoie cette
/// erreur
pub fn get_automata_texture<'a, T, V>(
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
pub fn get_svg<'a, T, V>(g: &impl ToDot<'a, T, V>, inverse: bool) -> Result<String>
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
}
