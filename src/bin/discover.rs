use std::{fs::File, path::Path};
use log::LevelFilter;
use simplelog::*;
use memory_stats::memory_stats;
use victory_archive::{destination::filesystem_dest::FileSystemDestination, plan};


fn main() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();


    let mut plan = plan::BackupPlan::new("Test".to_string());
    plan.add_source(Box::new(FileSystemDestination::new("/Users/alex/repos".to_string())));
    plan.discover(10_000, "./bk_data/".to_string()).unwrap();
    let plan_path = Path::new("./bk_data/_plan.yaml");
    plan.save_plan(plan_path.to_path_buf()).unwrap();
    
    if let Some(usage) = memory_stats() {
        println!("Current physical memory usage: {:.4}mb", usage.physical_mem / 1024 / 1024);
        println!("Current virtual memory usage: {:.4}mb", usage.virtual_mem / 1024 / 1024);
    } else {
        println!("Couldn't get the current memory usage :(");
    }


}