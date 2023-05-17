use log::debug;
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
    pub size: usize,
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

    pub fn load_contents(&mut self, contents: Vec<u8>) -> Result<(), String>{
        self.size = contents.len();
        self.contents = Some(contents);
        self.state = FileState::Read;
        debug!("Loaded contents for file: {:?} with size {:.1}MB", self.path, self.size as f64 / 1000000.0);
        Ok(())
    }

    pub fn get_contents(&self) -> Result<Vec<u8>, String>{
        match &self.contents{
            Some(contents) => Ok(contents.clone()),
            None => Err(format!("Error: File {:?} has no contents", self.path)),
        }
    }

    pub fn clear_contents(&mut self){
        self.contents = None;
        self.state = FileState::Stored;
    }
    
}
