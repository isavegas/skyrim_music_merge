mod game;
pub use game::*;
use winreg::RegKey;

mod parser;
pub use parser::*;

pub fn read_load_order() -> Result<Vec<String>, ()> {
    let mut list: Vec<String> = vec![];
    unimplemented!()
}

pub fn registry_install_path(key: String) -> Result<String, String> {
    RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(key, winreg::enums::KEY_READ)
        .map_err(|e| e.to_string())?
        .get_value("installed path")
        .map_err(|e| e.to_string())
}
