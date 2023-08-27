use std::{path::PathBuf, io::Write};

use log::debug;
use walkdir::WalkDir;

//Generate a fake file of a given size at a given path and return the path
pub fn file_generates(path:&PathBuf, size:usize) -> Result<&PathBuf, String>{
    let mut file = match std::fs::File::create(path){
        Ok(file) => file,
        Err(err) => return Err(format!("Error: {:?}", err)),
    };
    let mut data:Vec<u8> = Vec::new();
    for i in 0..size{
        data.push((i + i % 255) as u8);
    }
    match file.write_all(data.as_slice()){
        Ok(_) => Ok(path),
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}

pub fn file_generates_folder(path:&PathBuf, size:usize, count:usize) -> Result<&PathBuf, String>{
    match std::fs::create_dir_all(path.clone()){
        Ok(_) => (),
        Err(err) => return Err(format!("Error: {:?}", err)),
    }
   for i in 0..count{
        let mut file_path = path.clone();
        file_path.push(format!("file_{}", i));
        file_generates(&file_path, size)?;
    }
    Ok(path)
}

pub fn file_test_dir(test_name:String) -> PathBuf{
    let mut path = PathBuf::from(std::env::var("CARGO_TARGET_TMPDIR").unwrap_or("./target".to_string()));
    path.push(test_name);
    // Create the directory if it doesn't exist
    if !path.exists(){
        debug!("Creating test directory: {:?}", path);
        match std::fs::create_dir_all(&path){
            Ok(_) => (),
            Err(err) => panic!("Error creating test directory: {:?}", err),
        }
    }
    path
}

pub fn file_clear_test_dirs() -> PathBuf{
    let path = PathBuf::from(std::env::var("CARGO_TARGET_TMPDIR").unwrap_or("./target".to_string()));
    file_remove(&path).unwrap_or(());
    path
}


pub fn file_cwd() -> String{
    let cwd = PathBuf::from("./");
    cwd.to_str().unwrap().to_string()
}

pub fn file_files_in_dir(path: PathBuf) -> Result<Vec<PathBuf>, String>{
    let mut files = Vec::new();
    //use walkdir
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path().to_path_buf();
        files.push(path);
    }
    
    Ok(files)
}

pub fn file_remove(path: &PathBuf) -> Result<(), String>{
    match std::fs::remove_file(path){
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}
pub fn file_remove_all(path: &PathBuf) -> Result<(), String>{
    match std::fs::remove_dir_all(path){
        Ok(_) => Ok(()),
        Err(err) => Err(format!("remove_dir_all: {:?}", err)),
    }
}

#[cfg(test)]
mod file_utils_tests{
    use log::info;


    #[test]
    fn test_dir(){
        let path = super::file_test_dir("test_dir".to_string());
        assert!(path.exists());
    }

    #[test]
    fn test_file_generates(){
        let path = super::file_test_dir("test_file_generates".to_string());
        info!("test_file_generates path: {:?}", path);
        let path = path.join("test_file_generates");
        let path = super::file_generates(&path, 1000).unwrap();
        assert!(path.exists());
        assert_eq!(path.metadata().unwrap().len(), 1000);

        // Load the file and check the contents
        let contents = std::fs::read(path.clone()).unwrap();
        for i in 0..1000{
            assert_eq!(contents[i], (i + i % 255) as u8);
        }

        // Remove the file
        super::file_remove_all(&super::file_test_dir("test_file_generates".to_string())).unwrap();

        // Check that the file is gone
        assert!(!path.exists());
        
    }

    #[test]
    fn test_file_generates_folder(){
        let path = super::file_test_dir("test_file_generates_folder".to_string());
        info!("test_file_generates_folder path: {:?}", path);
        let path = path.join("test_file_generates_folder");
        let path = super::file_generates_folder(&path, 250, 10).unwrap();
        assert!(path.exists());

        // Load the file and check the contents
        let files = super::file_files_in_dir(path.clone()).unwrap();
        assert_eq!(files.len(), 11);

        // Remove the file
        super::file_remove_all(&super::file_test_dir("test_file_generates_folder".to_string())).unwrap();

        // Check that the file is gone
        assert!(!path.exists());

    }

    #[test]

    fn test_file_remove_all(){
        let test_path = super::file_test_dir("test_file_remove_all".to_string());
        info!("test_file_remove_all path: {:?}", &test_path);
        let path = super::file_generates_folder(&test_path, 1000, 10).unwrap();
        assert!(test_path.exists());

        // Remove the file
        super::file_remove_all(&test_path).unwrap();

        // Check that the file is gone
        assert!(!path.exists());
        
    }

    #[test]
    fn test_get_files_in_dir(){
        let test_path = super::file_test_dir("test_get_files_in_dir".to_string());
        

        info!("test_get_files_in_dir path: {:?}", test_path);
        let path = test_path.join("test.file");
        
        let path = super::file_generates(&path, 1000).unwrap();
        assert!(path.exists());

        // Get the files in the directory
        let files = super::file_files_in_dir(path.parent().unwrap().to_path_buf()).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[1].metadata().unwrap().len(), 1000);

        // Remove the file
        super::file_remove_all(&test_path).unwrap();
       
    }
}