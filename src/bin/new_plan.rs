#![feature(iter_next_chunk)]
use std::{fs::File, path::Path};


use log::LevelFilter;
use simplelog::*;

use memory_stats::memory_stats;
use victory_archive::{destination::filesystem_dest::FileSystemDestination, plan};


fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();


    let mut plan: plan::BackupPlan = plan::BackupPlan::new("RepoBackup".to_string());
    plan.add_source(Box::new(FileSystemDestination::new("/Users/alex/repos/".to_string())));
    plan.add_destination(Box::new(FileSystemDestination::new("/Users/alex/repos_back/".to_string())));
    plan.discover(10_000, "/Users/alex/repos_back/".to_string()).unwrap();
    let plan_path = Path::new("/Users/alex/repos_back/_plan.yaml");
    plan.save_plan(plan_path.to_path_buf()).unwrap();
    
    if let Some(usage) = memory_stats() {
        println!("Current physical memory usage: {:.4}mb", usage.physical_mem / 1024 / 1024);
        println!("Current virtual memory usage: {:.4}mb", usage.virtual_mem / 1024 / 1024);
    } else {
        println!("Couldn't get the current memory usage :(");
    }


}