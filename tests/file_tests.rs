use std::{path::PathBuf, io::Write};

//Generate a fake file of a given size at a given path and return the path
fn generate_fake_file(path:PathBuf, size:usize) -> Result<PathBuf, String>{
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




#[test]
fn test_add() {
    
}