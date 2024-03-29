use std::{
    io::Write,
    path::{Path, PathBuf},
};

use log::*;
use serde::{Deserialize, Serialize};

use crate::{
    batch::FileBatch,
    destination::{filesystem_dest::FileSystemDestination, Destination},
};
use num_format::{Locale, ToFormattedString};

/// A backup plan is a collection of sources and batches
///

pub struct BackupPlan {
    pub name: String,
    pub path: PathBuf,
    pub sources: Vec<Box<dyn Destination>>,
    pub destinations: Vec<Box<dyn Destination>>,
    pub batches: Vec<String>,
}

/// Savable version of the BackupPlan
/// # Fields:
/// - name: The name of the backup plan
/// - sources: The sources of the backup plan
/// - batches: The batches of the backup plan
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BackupPlanSave {
    pub name: String,
    pub path: String,
    pub sources: Vec<String>,
    pub batches: Vec<String>,
    pub destinations: Vec<String>,
}
impl BackupPlan {
    pub fn new(name: String) -> BackupPlan {
        BackupPlan {
            name: name,
            sources: Vec::new(),
            batches: Vec::new(),
            destinations: Vec::new(),
            path: PathBuf::new(),
        }
    }

    /// Generates a new backup plan from a saved backup plan (loaded from a file)
    ///
    /// # Arguments
    ///
    /// * `plan` - The saved backup plan
    pub fn from_saved(plan: BackupPlanSave) -> BackupPlan {
        let mut sources = Vec::new();
        //TODO: Dynamic way (enums?) to store what destinations are used
        for source in plan.sources {
            sources.push(Box::new(FileSystemDestination::new(source)) as Box<dyn Destination>);
        }
        let mut destinations = Vec::new();
        for destination in plan.destinations {
            destinations
                .push(Box::new(FileSystemDestination::new(destination)) as Box<dyn Destination>);
        }

        let mut batches = Vec::new();
        for batch in plan.batches {
            batches.push(batch);
        }
        BackupPlan {
            name: plan.name,
            sources: sources,
            batches: batches,
            destinations: destinations,
            path: PathBuf::from(plan.path),
        }
    }

    /// Generates a saved backup plan from a backup plan
    ///
    /// # Returns
    ///
    /// * `BackupPlanSave` - The saved backup plan
    pub fn get_saved(&self) -> BackupPlanSave {
        let mut sources = Vec::new();
        for source in &self.sources {
            sources.push(source.get_name());
        }

        let mut destinations = Vec::new();
        for destination in &self.destinations {
            destinations.push(destination.get_name());
        }

        let mut batches = Vec::new();
        for batch in &self.batches {
            batches.push(batch.clone());
        }
        BackupPlanSave {
            name: self.name.clone(),
            sources: sources,
            batches: batches,
            path: self.path.to_str().unwrap().to_string(),
            destinations: destinations,
        }
    }

    pub fn add_source(&mut self, source: Box<dyn Destination>) {
        self.sources.push(source);
    }

    pub fn add_destination(&mut self, destination: Box<dyn Destination>) {
        self.destinations.push(destination);
    }

    pub fn save_plan(&mut self, path: &PathBuf) -> Result<usize, String> {
        // Save path minus the file name

        if !path.exists() {
            info!("Creating path {:?}", path);
            match std::fs::create_dir_all(path) {
                Ok(_) => (),
                Err(err) => return Err(format!("Error: {:?}", err)),
            }
        }

        self.path = path.to_path_buf();
        let mut file_path: PathBuf = path.clone();
        file_path.push(format!("{}.yaml", self.name));

        let mut file = match std::fs::File::create(file_path) {
            Ok(file) => file,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        let yaml = serde_yaml::to_string(&self.get_saved()).expect("Error serializing plan");
        match file.write_all(yaml.as_bytes()) {
            Ok(_) => Ok(yaml.len()),
            Err(err) => Err(format!("plan_save_error: {:?}", err)),
        }
    }

    pub fn load_saved(path: PathBuf) -> Result<BackupPlanSave, String> {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        let plan_save: BackupPlanSave = match serde_yaml::from_reader(file) {
            Ok(plan_save) => plan_save,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        Ok(plan_save)
    }

    pub fn discover(&mut self, batch_size: u64, save_path: String) -> Result<(), String> {
        // Store start time
        let total_start_time = std::time::Instant::now();

        let mut total_files = 0;
        let mut batch_idx = 0;
        //TODO: Multithread this
        for source in &mut self.sources {
            //TODO: Make ID also show destintation, such as plan_dest_batch..
            loop {
                let batch_start_time = std::time::Instant::now();
                let mut batch =
                    FileBatch::new(self.name.to_string() + "_" + batch_idx.to_string().as_str());
                let files = match source.list_files_next(batch_size) {
                    Ok(files) => files,
                    Err(err) => {
                        error!("list_files_next ERROR: {:?}", err);
                        Vec::new()
                    }
                };
                if files.len() == 0 {
                    break;
                }
                batch.add_files(files);
                let batch_end_time = std::time::Instant::now();
                batch_idx += 1;
                total_files += batch.get_length();

                self.batches.push(batch.get_name());

                let batch_path = Path::new(save_path.as_str())
                    .join(batch.get_name().to_string() + ".vbak_batch");
                // Save batch
                let save_size = match batch.save_batch(batch_path) {
                    Ok(res) => res,
                    Err(err) => {
                        error!("save_batch ERROR: {:?}", err);
                        0
                    }
                };
                let batch_save_time = std::time::Instant::now();

                debug!(
                    "Batch {}:
                    \t- Length: {}
                    \t- Disk size: {} kb
                    \t- Time to discover: {:.2}ms
                    \t- Time to save: {:.2}ms
                    \t- Path: {:?}",
                    batch.get_name(),
                    (batch.get_length() as u64).to_formatted_string(&Locale::en),
                    save_size / 1024,
                    batch_end_time.duration_since(batch_start_time).as_micros() as f64 / 1000.,
                    batch_save_time.duration_since(batch_end_time).as_micros() as f64 / 1000.,
                    save_path.clone()
                );
            }
        }

        let total_end_time = std::time::Instant::now();

        info!(
            "Total time to discover {} batches with {} files: {}ms",
            batch_idx,
            total_files.to_formatted_string(&Locale::en),
            total_end_time.duration_since(total_start_time).as_millis()
        );

        Ok(())
    }
    fn process_batch(&self, batch_name: String) {
        let plan_path = self.path.clone();
        let batch_start_time = std::time::Instant::now();
        let batch_path = plan_path.join(batch_name.to_string() + ".vbak_batch");

        info!("Loading batch {:?}", batch_path);

        let mut batch = match FileBatch::load_batch(batch_path) {
            Ok(batch) => batch,
            Err(err) => {
                error!("Error loading batch: {:?}", err);
                return;
            }
        };

        let mut writen = 0;
        for file in batch.get_files() {
            // Read file from source

            match self.sources[0].read_file(file) {
                Ok(file_contents) => file_contents,
                Err(err) => {
                    error!("Error reading file {:?}: {:?}", file.name, err);
                    continue;
                }
            };

            // replace name by replacing source path with destination path
            //file.path = file.path.replace(self.sources[0].get_name().as_str(), self.destinations[0].get_name().as_str());
            //debug!("Writing file {:?}", file.path);

            match self.destinations[0].write_file(file) {
                Ok(_) => {
                    writen += 1;
                }
                Err(err) => {
                    error!("Error writing file {:?}: {:?}", file.path, err);
                    continue;
                }
            };
        }

        info!(
            "Wrote {} files in {:.2}s",
            writen,
            batch_start_time.elapsed().as_secs_f64()
        );
    }
    // 1. Load Batch
    // 2. Read file from batch using destination
    // 3. Read file contents
    // 4. Write file contents to destination
    pub fn run(&mut self) {
        info!("Running backup plan {}", self.name);
        let _plan_path = self.path.clone();
        for batch_name in &self.batches {
            self.process_batch(batch_name.to_string());
        }
    }
}
