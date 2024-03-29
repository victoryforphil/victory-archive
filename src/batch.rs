use std::{
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::file::VictoryFile;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FileBatch {
    pub files: Vec<VictoryFile>,
    name: String,
}

impl FileBatch {
    pub fn new(name: String) -> FileBatch {
        FileBatch {
            files: Vec::new(),
            name: name,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_length(&self) -> usize {
        self.files.len()
    }

    pub fn add_file(&mut self, file: VictoryFile) {
        self.files.push(file);
    }

    pub fn add_files(&mut self, files: Vec<VictoryFile>) {
        for file in files {
            self.add_file(file);
        }
    }

    pub fn save_batch(&self, path: PathBuf) -> Result<usize, String> {
        match path.parent() {
            Some(parent) => match std::fs::create_dir_all(parent) {
                Ok(_) => (),
                Err(err) => return Err(format!("save_batch Error: {:?}", err)),
            },
            None => return Err(format!("save_batch Error: Invalid path {:?}", path)),
        }

        let mut file = match std::fs::File::create(path) {
            Ok(file) => file,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        /*
        let serialized = match bincode::serialize(&self){
            Ok(serialized) => serialized,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        match file.write_all(serialized.as_slice()){
            Ok(_) => Ok(serialized.len()),
            Err(err) => Err(format!("Error: {:?}", err)),
        } */

        // Save as Yaml
        let yaml = serde_yaml::to_string(&self).expect("Error serializing plan");
        match file.write_all(yaml.as_bytes()) {
            Ok(_) => Ok(yaml.len()),
            Err(err) => Err(format!("plan_save_error: {:?}", err)),
        }
    }

    pub fn load_batch(path: PathBuf) -> Result<FileBatch, String> {
        let mut file = match std::fs::File::open(path.clone()) {
            Ok(file) => file,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        /*
         let mut serialized = Vec::new();
        match file.read_to_end(&mut serialized) {
            Ok(_) => (),
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        let batch = match bincode::deserialize(&serialized) {
            Ok(batch) => batch,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
         */

        // Load from Yaml
        let mut yaml = String::new();
        match file.read_to_string(&mut yaml) {
            Ok(_) => (),
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        let batch: FileBatch = match serde_yaml::from_str(&yaml) {
            Ok(batch) => batch,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        Ok(batch)
    }

    pub fn get_files(&mut self) -> &mut Vec<VictoryFile> {
        &mut self.files
    }
}

#[cfg(test)]
mod file_batch_tests {
    use super::*;
    use crate::file::VictoryFile;

    #[test]
    fn test_add_file() {
        let mut batch = FileBatch::new("test".to_string());
        let file = VictoryFile::new(&PathBuf::from("test".to_string()));
        batch.add_file(file.clone());
        assert_eq!(batch.files.len(), 1);
        assert_eq!(batch.files[0], file);
    }

    #[test]
    fn test_add_files() {
        let mut batch = FileBatch::new("test".to_string());
        let file = VictoryFile::new(&PathBuf::from("test".to_string()));
        let file2 = VictoryFile::new(&PathBuf::from("tes2".to_string()));
        batch.add_files(vec![file.clone(), file2.clone()]);
        assert_eq!(batch.files.len(), 2);
        assert_eq!(batch.files[0], file);
        assert_eq!(batch.files[1], file2);
    }

    #[test]
    fn test_get_name() {
        let batch = FileBatch::new("test".to_string());
        assert_eq!(batch.get_name(), "test".to_string());
    }
    //Test save_batch in a temp dir
    #[test]
    fn test_save_batch() {
        let mut batch = FileBatch::new("test".to_string());
        let file = VictoryFile::new(&PathBuf::from("test".to_string()));
        batch.add_file(file.clone());
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_batch.victory");
        batch.save_batch(path.clone()).unwrap();
        let batch2 = FileBatch::load_batch(path.clone()).unwrap();
        assert_eq!(batch, batch2);
        // Delete the file
        std::fs::remove_file(path.clone()).unwrap();
    }

    #[test]
    fn test_load_batch() {
        let mut batch = FileBatch::new("test".to_string());
        let file = VictoryFile::new(&PathBuf::from("test".to_string()));
        batch.add_file(file.clone());
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_batch.victory");

        println!("Path: {:?}", path);

        batch.save_batch(path.clone()).unwrap();
        let batch2 = FileBatch::load_batch(path.clone()).unwrap();
        assert_eq!(batch, batch2);

        // Delete the file
        std::fs::remove_file(path.clone()).unwrap();
    }
}
