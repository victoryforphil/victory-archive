use crate::file::VictoryFile;

pub mod filesystem_dest;

pub trait Destination {
    fn list_files_next(&mut self, count: u64) -> Result<Vec<VictoryFile>, String>;
    fn read_file(&self, file: &mut VictoryFile) -> Result<(), String>;
    fn write_file(&self, file: &mut VictoryFile) -> Result<(), String>;
    fn get_name(&self) -> String;
}
