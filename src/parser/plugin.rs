use std::path::PathBuf;
use std::path::Path;
use std::boxed::Box;

use parser::MUSC;

pub struct Plugin {
    path: Box<PathBuf>,
    pub name: String,
    pub version: f32,
    pub num_records: i32,
    pub next_object_id: u32,
    pub author: String,
    pub description: String,
    pub masters: Vec<String>,
    pub overrides: Vec<u32>,
    pub intv: u32, // unknown
    pub incc: u32, // unknown
    pub music: Vec<MUSC>,
}

impl Plugin {
    pub fn new(p: &Path) -> Plugin {
//        let data = Box<String>::new("Test");
        Plugin {
            path: Box::new(p.to_owned()),
            name: String::from(p.file_name().unwrap().to_str().unwrap()),
            num_records: 0,
            next_object_id: 0,
            author: String::from(""),
            description: String::from(""),
            masters: vec!(),
            overrides: vec!(),
            intv: 0,
            incc: 0,
            version: 0.0,
            music: vec!(),
        }
    }
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}