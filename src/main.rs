use std::fs::File;

use destination::filesystem_dest::FileSystemDestination;
use log::LevelFilter;
use simplelog::*;

pub mod plan;
pub mod trigger;
pub mod batch;
pub mod file;
pub mod destination;


fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();


    let mut plan = plan::BackupPlan::new("Test".to_string());
    plan.add_source(Box::new(FileSystemDestination::new("/Users/alex/repos".to_string())));
    plan.discover(10_000).unwrap();
}