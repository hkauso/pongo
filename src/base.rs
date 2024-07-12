//! Base values for the base template (`templates/base.html`)
use std::env;

#[derive(Debug, Clone)]
pub struct BaseStore {
    /// `PO_SITE_NAME` variable
    pub site_name: String,
    /// `PO_BODY_EMBED` variable, HTML that is embedded on every page
    pub body_embed: String,
    /// `PO_STARSTRAW` variable, for starstraw auth api nesting location (required)
    pub starstraw: String,
    /// `PO_STATIC_DIR` variable, where Pongo's static assets are located
    pub static_dir: String,
    /// `PO_NESTED` variable, where Pongo is nested (relative to root)
    pub nested: String,
}

impl BaseStore {
    pub fn new() -> Self {
        Self {
            site_name: match env::var("PO_SITE_NAME") {
                Ok(s) => s,
                Err(_) => String::from("Pongo"),
            },
            body_embed: match env::var("PO_BODY_EMBED") {
                Ok(s) => s,
                Err(_) => String::new(),
            },
            starstraw: match env::var("PO_STARSTRAW") {
                Ok(s) => s,
                Err(_) => panic!("Starstraw is required to use Pongo."),
            },
            static_dir: match env::var("PO_STATIC_DIR") {
                Ok(s) => s,
                Err(_) => {
                    panic!("No static dir has been provided! Pongo needs client files to function.")
                }
            },
            nested: match env::var("PO_NESTED") {
                Ok(s) => s,
                Err(_) => String::from("@pongo"),
            },
        }
    }
}
