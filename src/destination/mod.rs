
use crate::file::VictoryFile;

pub mod filesystem_dest;

pub trait Destination{
    fn list_files_next(&mut self, count: u64) -> Result<Vec<VictoryFile>, String> ;
    fn get_name(&self) -> String;
}