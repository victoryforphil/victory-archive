use std::{
    fs::File,
    path::{Path, PathBuf},
};

use log::LevelFilter;
use simplelog::*;
use victory_archive::utils::file_utils::file_generates_folder;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
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
    // Get This repo root directory
    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Save path as source_dir + /test_data
    let dir = source_dir.join("test_data");
    let _ = file_generates_folder(&dir, 1_000_000, 2500);
}
