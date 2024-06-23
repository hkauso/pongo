//! Base values for the base template (`templates/base.html`)
use std::env;

#[derive(Debug, Clone)]
pub struct BaseStore {
    /// `SITE_NAME` variable
    pub site_name: String,
    /// `INFO_URL` variable, "what" in the footer
    pub info_url: String,
    /// `BODY_EMBED` variable, HTML that is embedded on every page
    pub body_embed: String,
    /// `GUPPY_ROOT` variable, for guppy auth (required)
    pub guppy_root: String,
    /// `STATIC_DIR` variable, where Pongo's static assets are located
    pub static_dir: String,
}

impl BaseStore {
    pub fn new() -> Self {
        Self {
            site_name: match env::var("SITE_NAME") {
                Ok(s) => s,
                Err(_) => String::from("Pongo"),
            },
            info_url: match env::var("INFO_URL") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            body_embed: match env::var("BODY_EMBED") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            guppy_root: match env::var("GUPPY_ROOT") {
                Ok(s) => s,
                Err(_) => panic!("Guppy is required to use Pongo."),
            },
            static_dir: match env::var("STATIC_DIR") {
                Ok(s) => s,
                Err(_) => {
                    panic!("No static dir has been provided! Pongo needs client files to function.")
                }
            },
        }
    }
}
