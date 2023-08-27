use std::{
    fs::File,
    path::{self, Path},
};

use log::{info, LevelFilter};
use simplelog::*;

use victory_archive::{executor::Executor, plan};

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .unwrap();
    let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data");
    let test_dest_dir = test_data_dir.join("_dest");
    let test_meta_dir = test_data_dir.join("__vk");
    let test_plan_path = test_meta_dir.join("TestData.yaml");
    let saved_plan = plan::BackupPlan::load_saved(test_plan_path.to_path_buf()).unwrap();

    info!("Plan loaded: {:?}", saved_plan);

    let mut plan = plan::BackupPlan::from_saved(saved_plan);

    let res = Executor::run(&plan);
    match res {
        Ok(_) => info!("Plan executed successfully"),
        Err(e) => info!("Plan failed to execute: {:?}", e),
    }
    // plan.run();
}
