use std::fmt;

#[derive(Debug, Clone)]
pub enum Game {
    Morrowind,
    Oblivion,
    Skyrim,
    SkyrimSE,
    SkyrimVR,
    Fallout3,
    Fallout4,
    Fallout4VR,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Morrowind => "Morrowind",
            Self::Oblivion => "Oblivion",
            Self::Skyrim => "Skyrim",
            Self::SkyrimSE => "Skyrim Special Edition",
            Self::SkyrimVR => "Skyrim VR",
            Self::Fallout3 => "Fallout 3",
            Self::Fallout4 => "Fallout 4",
            Self::Fallout4VR => "Fallout 4 VR",
        })
    }
}

impl Game {
    // Skyrim and Fallout 4 don't have their core esm files
    // listed in the load order, so we use this to include them.
    pub fn implicit_modules(&self) -> Vec<String> {
        match self {
            Self::Skyrim | Self::SkyrimSE | Self::SkyrimVR => vec![
                "Skyrim.esm".to_string(),
                "Update.esm".to_string(),
            ],
            Self::Fallout4 | Self::Fallout4VR => vec![
                "Fallout4.esm".to_string(),
            ],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSettings {
    pub id: String,
    pub name: String,
    pub location: String,
    pub implicit_modules: Vec<String>,
    pub load_order: Vec<String>,
}

impl GameSettings {
    pub fn new<S>(game: Game, location: S) -> Self
    where
        S: Into<String>,
    {
        GameSettings {
            id: format!("{}", game), // TODO: Figure out why I have this
            name: format!("{}", game),
            location: location.into(),
            implicit_modules: game.implicit_modules(),
            load_order: vec![],
        }
    }
    // Note that Morrowind includes its load order in Morrwind.ini
    // in the base install directory!
    pub fn read_load_order() {
        unimplemented!()
    }
}
