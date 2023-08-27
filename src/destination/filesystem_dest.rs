use std::{
    fs,
    path::{self, Path, PathBuf},
};

use log::debug;

use crate::file::VictoryFile;

use super::Destination;

#[derive(Debug)]
pub struct FileSystemDestination {
    path: String,
    walk_itr: walkdir::IntoIter,
}

impl FileSystemDestination {
    pub fn new(path: String) -> FileSystemDestination {
        debug!("Creating FileSystemDestination: {:?}", path);
        let walk_itr = walkdir::WalkDir::new(path.clone())
            .sort_by_file_name()
            .into_iter();
        FileSystemDestination {
            path: path,
            walk_itr: walk_itr,
        }
    }
}

impl Destination for FileSystemDestination {
    fn list_files_next(&mut self, count: u64) -> Result<Vec<VictoryFile>, String> {
        let mut files = Vec::new();
        //TODO: Replace with chunk
        let mut count = count;
        while count > 0 {
            let file = match self.walk_itr.next() {
                Some(file) => file,
                None => return Ok(files),
            };
            let file = match file {
                Ok(file) => Some(file),
                Err(err) => {
                    log::warn!("ListError: {:?}", err);
                    None
                }
            };
            match file {
                Some(file) => {
                    debug!("Found file: {:?}", file);
                    if file.file_type().is_file() {
                        // Delete self.path section of the file path before saving
                        let self_path = Path::new(&self.path);
                        let relative_path = file.path().strip_prefix(self_path).unwrap();
                        let file = VictoryFile::new(&relative_path.to_path_buf());
                        files.push(file);
                        count -= 1;
                    }
                }
                None => (),
            }
        }
        Ok(files)
    }

    fn get_name(&self) -> String {
        self.path.clone()
    }

    fn read_file(&self, file: &mut VictoryFile) -> Result<(), String> {
        let file_path: &Path = Path::new(&file.path);
        let full_path = Path::new(&self.path).join(file_path);
        debug!("[ReadFile] Destination Path: {:?}", self.path);
        debug!("[ReadFile] Reading file: {:?}", full_path);

        let contents = std::fs::read(full_path);

        let contents = match contents {
            Ok(c) => c,
            Err(err) => {
                log::warn!("ReadError: {:?}", err);
                Vec::new()
            }
        };
        file.load_contents(contents)?;
        Ok(())
    }

    fn write_file(&self, file: &mut VictoryFile) -> Result<(), String> {
        debug!("[WriteFile] Destination Path: {:?}", self.path);
        let contents = file.get_contents()?;
        let file_path: &Path = Path::new(&file.path);

        debug!("[WriteFile] File Path: {:?}", file_path);
        let full_path = Path::new(&self.path).join(file_path);
        // if directory, create
        debug!("[WriteFile] Writing file {:?}", full_path);

        if !full_path.parent().unwrap().exists() {
            debug!("[WriteFile] Creating dir: {:?}", full_path);
            match fs::create_dir_all(&full_path) {
                Ok(_) => (),
                Err(err) => {
                    log::warn!(
                        "create_dir_all Error: {:?} with path {:?}",
                        err,
                        full_path.clone()
                    );
                    return Err(format!("create_dir_all Error: {:?}", err));
                }
            }
        }

        // write file
        log::debug!("Writing file: {:?}", &full_path);
        match std::fs::write(&full_path, contents) {
            Ok(_) => Ok(()),
            Err(err) => {
                log::warn!("write Error: {:?}", err);
                Err(format!("write Error: {:?}", err))
            }
        }
    }
}

#[cfg(test)]

mod fs_dest_tests {

    use crate::utils::file_utils::file_cwd;

    use super::*;

    // Workaround to use prinltn! for logs.

    // Get current working directory

    #[test]
    fn test_list_files_next_count() {
        let mut dest = FileSystemDestination::new(file_cwd());
        let files = dest.list_files_next(1000).unwrap();
        assert!(files.len() > 0);

        let mut dest = FileSystemDestination::new(file_cwd());
        let files = dest.list_files_next(1).unwrap();
        assert_eq!(files.len(), 1);

        let mut dest = FileSystemDestination::new(file_cwd());
        let files = dest.list_files_next(0).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_list_files_next_filedata() {
        let mut dest = FileSystemDestination::new(file_cwd() + "/src/destination");

        let mut files = dest.list_files_next(1).unwrap();
        let mut file = files.pop().unwrap();

        dest.read_file(&mut file).expect("Failed to read");

        assert!(file.size > 0);
    }

    #[test]
    fn test_read_file() {
        //Make a temp file
        let mut dest = FileSystemDestination::new(file_cwd());
        let mut files = dest.list_files_next(10).unwrap();

        let mut file = files.pop().unwrap();
        dest.read_file(&mut file).unwrap();
        assert!(file.size > 0);

        assert!(file.state == crate::file::FileState::Read);
    }
}
