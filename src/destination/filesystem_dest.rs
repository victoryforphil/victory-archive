use std::fs;

use log::debug;

use crate::file::VictoryFile;

use super::Destination;

#[derive(Debug)]
pub struct FileSystemDestination{
    path: String,
    walk_itr: walkdir::IntoIter,
}

impl FileSystemDestination{
    pub fn new(path: String) -> FileSystemDestination{
        let walk_itr = walkdir::WalkDir::new(path.clone()).sort_by_file_name().into_iter();
        FileSystemDestination{
            path: path,
            walk_itr: walk_itr,
        }
    }
}


impl Destination for FileSystemDestination{
    fn list_files_next(&mut self, count: u64) -> Result<Vec<VictoryFile>, String> {
        let mut files = Vec::new();
        //TODO: Replace with chunk
        for _ in 0..count{
            let file = match self.walk_itr.next(){
                Some(file) => file,
                None => return Ok(files),
            };
            let file = match file{
                Ok(file) => Some(file),
                Err(err) => {
                    log::warn!("ListError: {:?}", err);
                    None
                
                },
            };
            match file{
                Some(file) => {
                    debug!("Found file: {:?}", file);
                    if file.file_type().is_dir(){
                        continue;
                    }
                    let file = VictoryFile::new(file.path().to_str().unwrap().to_string());
                    files.push(file);
                },
                None => (),
            }
        }
        Ok(files)
    }

    fn get_name(&self) -> String {
        self.path.clone()
    }

    fn read_file(&self, file: &mut VictoryFile) -> Result<(), String> {
        let contents = std::fs::read(file.path.clone());

        let contents = match contents{
            Ok(c) => {c},
            Err(err) => {
                log::warn!("ReadError: {:?}", err);
                Vec::new()
            },
        };
        file.load_contents(contents)?;
        Ok(())
    }

    fn write_file(&self, file: &mut VictoryFile) -> Result<(), String> {
        let contents = file.get_contents()?;
        
        // if directory, create
        let path = std::path::Path::new(&file.path);
        let dir = path.parent().unwrap();
        if !dir.exists(){
            match fs::create_dir_all(dir){
                Ok(_) => (),
                Err(err) => {

                    log::warn!("create_dir_all Error: {:?} with path {:?}", err, dir.clone());
                    return Err(format!("create_dir_all Error: {:?}", err));
                },
            }
        }

        // write file

        match std::fs::write(&file.path, contents){
            Ok(_) => Ok(()),
            Err(err) => {
                log::warn!("write Error: {:?}", err);
                Err(format!("write Error: {:?}", err))
            },
        }
    }
}

#[cfg(test)]

mod fs_dest_tests {
    use walkdir::WalkDir;

    use super::*;
    use crate::file::VictoryFile;
    use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

    // Get current working directory
    fn get_cwd() -> String{
        let cwd = std::env::current_dir().unwrap();
        cwd.to_str().unwrap().to_string()
    }

    fn get_files_in_dir(path: String) -> Vec<String>{
        let mut files = Vec::new();
        //use walkdir
        for entry in WalkDir::new(path) {
            let entry = entry.unwrap();
            let path = entry.path().to_str().unwrap().to_string();
            files.push(path);
        }
        
        files
    }

    #[test]
    fn test_list_files_next_count(){
        let mut dest = FileSystemDestination::new(get_cwd());
        let files = dest.list_files_next(1000).unwrap();
        assert!(files.len() > 0);

        let mut dest = FileSystemDestination::new(get_cwd());
        let files = dest.list_files_next(1).unwrap();
        assert_eq!(files.len(), 1);

        let mut dest = FileSystemDestination::new(get_cwd());
        let files = dest.list_files_next(0).unwrap();
        assert_eq!(files.len(), 0);
      
    }

    #[test]
    fn test_list_files_next_filedata(){
        let mut dest = FileSystemDestination::new(get_cwd());
        
        let files = get_files_in_dir(get_cwd());

        let mut files = files.iter();
        let file = files.next().unwrap();
        let file = VictoryFile::new(file.to_string());

        
        let mut file2 = dest.list_files_next(1).unwrap();
        let file2 = file2.pop().unwrap();
        assert_eq!(file, file2);   
    }

    #[test]
    fn test_list_files_next_lengths(){
        let mut dest = FileSystemDestination::new(get_cwd());
        
        let files = get_files_in_dir(get_cwd());

        let mut files = files.iter();
        let file = files.next().unwrap();
        let file = VictoryFile::new(file.to_string());

        
        let mut file2 = dest.list_files_next(1).unwrap();
        let file2 = file2.pop().unwrap();
        assert_eq!(file, file2);   
    }

    #[test]
    fn test_read_file(){
       //Make a temp file
         let mut dest = FileSystemDestination::new(get_cwd());
            let mut files = dest.list_files_next(10).unwrap();
           
            let mut file = files.pop().unwrap();
            dest.read_file(&mut file).unwrap();
            assert!(file.size > 0);
  
            assert!(file.state == crate::file::FileState::Read);

    }

}