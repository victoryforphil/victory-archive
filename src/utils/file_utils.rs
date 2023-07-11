use std::{path::PathBuf, io::Write};

use walkdir::WalkDir;

//Generate a fake file of a given size at a given path and return the path
pub fn file_generates(path:PathBuf, size:usize) -> Result<PathBuf, String>{
    let mut file = match std::fs::File::create(path.clone()){
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

pub fn file_test_dir() -> PathBuf{
    let mut path = PathBuf::from(std::env::var("CARGO_TARGET_TMPDIR").unwrap_or("./target".to_string()));
    path.push("test");
    // Create the directory if it doesn't exist
    if !path.exists(){
        match std::fs::create_dir_all(&path){
            Ok(_) => (),
            Err(err) => panic!("Error creating test directory: {:?}", err),
        }
    }
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
        Err(err) => Err(format!("Error: {:?}", err)),
    }
}

#[cfg(test)]
mod file_utils_tests{
    use log::info;


    #[test]
    fn test_dir(){
        let path = super::file_test_dir();
        assert!(path.exists());
    }

    #[test]
    fn test_file_generates(){
        let path = super::file_test_dir();
        info!("test_file_generates path: {:?}", path);
        let path = path.join("test_file_generates");
        let path = super::file_generates(path, 1000).unwrap();
        assert!(path.exists());
        assert_eq!(path.metadata().unwrap().len(), 1000);

        // Load the file and check the contents
        let contents = std::fs::read(path.clone()).unwrap();
        for i in 0..1000{
            assert_eq!(contents[i], (i + i % 255) as u8);
        }

        // Remove the file
        super::file_remove_all(&super::file_test_dir()).unwrap();

        // Check that the file is gone
        assert!(!path.exists());

    }

    #[test]

    fn test_file_remove_all(){
        let path = super::file_test_dir();
        info!("test_file_remove_all path: {:?}", path);
        let path = path.join("test_file_remove_all");
        let path = super::file_generates(path, 1000).unwrap();
        assert!(path.exists());

        // Remove the file
        super::file_remove_all(& super::file_test_dir()).unwrap();

        // Check that the file is gone
        assert!(!path.exists());
        
    }

    #[test]
    fn test_get_files_in_dir(){
        let path = super::file_test_dir();
        info!("test_get_files_in_dir path: {:?}", path);
        let path = path.join("test.file");
        let path = super::file_generates(path, 1000).unwrap();
        assert!(path.exists());

        // Get the files in the directory
        let files = super::file_files_in_dir(path.parent().unwrap().to_path_buf()).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[1].metadata().unwrap().len(), 1000);
    }
}