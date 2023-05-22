use std::{fs::File, path::Path};

use destination::filesystem_dest::FileSystemDestination;
use log::LevelFilter;
use simplelog::*;

pub mod plan;
pub mod trigger;
pub mod batch;
pub mod file;
pub mod destination;
use memory_stats::memory_stats;


fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();

    let plan_path = Path::new("/Users/alex/repos/victoryforphil/victory-archive/bk_data/_plan.yaml");
    let loaded_plan = plan::BackupPlan::load_saved(plan_path.to_path_buf().clone()).expect("Failed to load plan");

    let mut plan = plan::BackupPlan::from_saved(loaded_plan);
}