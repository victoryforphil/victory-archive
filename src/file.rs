use serde::{Serialize, Deserialize};

#[derive( Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FileState{
    Discovered,
    Inspected,
    Read,
    Stored,
    Error,
    Skipped,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq,)]
pub struct VictoryFile{
    pub name: String,
    pub path: String,
    pub extension: String,
    pub state: FileState,
    pub contents: Option<Vec<u8>>,
    pub size: u64,
    pub hash: String,
}

impl VictoryFile{
    pub fn new(path: String) -> VictoryFile{
        let name = path.split("/").last().unwrap().to_string();
        let extension = name.split(".").last().unwrap().to_string();
        VictoryFile{
            name: name,
            path: path,
            extension: extension,
            state: FileState::Discovered,
            contents: None,
            size: 0,
            hash: "".to_string(),
        }
    }
}
