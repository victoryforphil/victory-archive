use std::{fs::File, path::{Path, self}};


use log::{LevelFilter, info};
use simplelog::*;

use memory_stats::memory_stats;
use victory_archive::{destination::filesystem_dest::FileSystemDestination, plan};


fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();


    let saved_plan = plan::BackupPlan::load_saved(path::Path::new("/Users/alex/repos_back/_plan.yaml").to_path_buf()).unwrap();

    info!("Plan loaded: {:?}", saved_plan);

    let mut plan = plan::BackupPlan::from_saved(saved_plan);
    plan.run();



}