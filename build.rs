#[cfg(not(feature = "no-blp"))]
use std::{
    fs::read_dir,
    io::{Error, ErrorKind},
    process::Command,
};

use std::io::Result;

fn main() -> Result<()> {
    #[cfg(not(feature = "no-blp"))]
    {
        read_dir("resources")?.try_for_each(|entry| {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) != Some("blp") {
                return Result::Ok(());
            }
            let path_out = path.with_extension("ui");
            let out = path_out.to_str().ok_or(Error::new(
                ErrorKind::InvalidInput,
                "Path contains invalid characters",
            ))?;
            let r#in = path.to_str().ok_or(Error::new(
                ErrorKind::InvalidInput,
                "Path contains invalid characters",
            ))?;
            Command::new("blueprint-compiler")
                .args(["compile", "--output", out, r#in])
                .spawn()
                .expect(r#"Failed to start "blueprint-compiler""#)
                .wait()
                .map(|_| ())
        })?;
    }
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "glushkovizer.gresource",
    );
    Ok(())
}
