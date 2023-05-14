use crate::file::VictoryFile;

use super::Destination;

#[derive(Debug)]
pub struct FileSystemDestination{
    path: String,
    walk_itr: walkdir::IntoIter,
}

impl FileSystemDestination{
    pub fn new(path: String) -> FileSystemDestination{
        let walk_itr = walkdir::WalkDir::new(path.clone()).into_iter();
        FileSystemDestination{
            path: path,
            walk_itr: walk_itr,
        }
    }
}


impl Destination for FileSystemDestination{
    fn list_files_next(&mut self, count: u64) -> Result<Vec<VictoryFile>, String> {
        let mut files = Vec::new();
        for _ in 0..count{
            let file = match self.walk_itr.next(){
                Some(file) => file,
                None => return Ok(files),
            };
            let file = match file{
                Ok(file) => file,
                Err(err) => return Err(format!("Error: {:?}", err)),
            };
            let file = VictoryFile::new(file.path().to_str().unwrap().to_string());
            files.push(file);
        }
        Ok(files)
    }

    fn get_name(&self) -> String {
        self.path.clone()
    }
}

#[cfg(test)]
mod fs_dest_tests {
    use walkdir::WalkDir;

    use super::*;
    use crate::file::VictoryFile;

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

}