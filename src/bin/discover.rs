use log::LevelFilter;
use memory_stats::memory_stats;
use simplelog::*;
use std::{fs::File, path::Path};
use victory_archive::{
    destination::filesystem_dest::FileSystemDestination, executor::Executor, plan,
};

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .unwrap();

    let mut plan: plan::BackupPlan = plan::BackupPlan::new("TestData".to_string());
    let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data");
    let test_dest_dir = test_data_dir.join("_dest");
    let test_meta_dir = test_data_dir.join("__vk");

    plan.add_source(Box::new(FileSystemDestination::new(
        test_data_dir.to_str().unwrap().to_string(),
    )));
    plan.add_destination(Box::new(FileSystemDestination::new(
        test_dest_dir.to_str().unwrap().to_string(),
    )));

    plan.save_plan(&test_meta_dir.to_path_buf()).unwrap();
    let res = Executor::discover(&mut plan, 50);

    plan.save_plan(&test_meta_dir.to_path_buf()).unwrap();

    if let Some(usage) = memory_stats() {
        println!(
            "Current physical memory usage: {:.4}mb",
            usage.physical_mem / 1024 / 1024
        );
        println!(
            "Current virtual memory usage: {:.4}mb",
            usage.virtual_mem / 1024 / 1024
        );
    } else {
        println!("Couldn't get the current memory usage :(");
    }
}
