use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use log::{debug, error, info};
use num_format::{Locale, ToFormattedString};

use crate::{
    batch::{self, FileBatch},
    plan::BackupPlan,
};

pub struct Executor {}

pub struct ExecutorDiscoveryResults {
    pub files: usize,
    pub batches: usize,
    pub batch_time: Duration,
    pub total_time: Duration,
}

impl ExecutorDiscoveryResults {
    pub fn new(
        files: usize,
        batches: usize,
        start_time: Instant,
        batch_time: Instant,
        total_time: Instant,
    ) -> ExecutorDiscoveryResults {
        ExecutorDiscoveryResults {
            files: files,
            batches: batches,
            batch_time: batch_time.duration_since(start_time),
            total_time: total_time.duration_since(start_time),
        }
    }
}

impl Executor {
    pub fn discover(
        plan: &mut BackupPlan,
        batch_size: u64,
    ) -> Result<ExecutorDiscoveryResults, String> {
        // Store start time
        let total_start_time = std::time::Instant::now();

        let mut total_files = 0;
        let mut batch_idx = 0;
        //TODO: Multithread this
        for source in &mut plan.sources {
            //TODO: Make ID also show destintation, such as plan_dest_batch..
            loop {
                let batch_start_time = std::time::Instant::now();
                let source = source.as_mut();
                let mut batch =
                    FileBatch::new(plan.name.to_string() + "_" + batch_idx.to_string().as_str());

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

                plan.batches.push(batch.get_name());
                info!("path: {:?}", plan.path);
                let batch_path = plan
                    .path
                    .join(".vbatches/")
                    .join(batch.get_name().to_string() + ".vbak_batch");
                // Save batch
                let save_size = match batch.save_batch(batch_path.clone()) {
                    Ok(res) => res,
                    Err(err) => {
                        error!("save_batch ERROR: {:?}", err);
                        0
                    }
                };
                let batch_save_time = std::time::Instant::now();

                info!(
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
                    batch_path.clone()
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

        Ok(ExecutorDiscoveryResults::new(
            total_files,
            batch_idx,
            total_start_time,
            total_end_time,
            total_end_time,
        ))
    }

    pub fn process_batch(
        plan: &BackupPlan,
        batch_path: &PathBuf,
    ) -> Result<ExecutorDiscoveryResults, String> {
        let plan_path = plan.path.clone();

        info!("Executor: Loading batch: {:?}", batch_path);
        let batch_start_time = std::time::Instant::now();
        let mut batch = match FileBatch::load_batch(batch_path.clone()) {
            Ok(batch) => batch,
            Err(err) => {
                error!("Executor: Error loading batch: {:?}", err);
                return Err(err);
            }
        };

        let mut writen = 0;
        for file in batch.get_files() {
            // Read file from source

            match plan.sources[0].read_file(file) {
                Ok(file_contents) => file_contents,
                Err(err) => {
                    error!("Executor: Error reading file {:?}: {:?}", file.name, err);
                    continue;
                }
            };

            // replace name by replacing source path with destination path
            //file.path = file.path.replace(self.sources[0].get_name().as_str(), self.destinations[0].get_name().as_str());

            match plan.destinations[0].write_file(file) {
                Ok(_) => {
                    writen += 1;
                }
                Err(err) => {
                    error!("Executor: Error writing file {:?}: {:?}", file.path, err);
                    continue;
                }
            };
        }

        info!(
            "Wrote {} files in {:.4}s",
            writen,
            batch_start_time.elapsed().as_secs_f64()
        );
        return Ok(ExecutorDiscoveryResults {
            files: writen,
            batches: 1,
            batch_time: batch_start_time.elapsed().clone(),
            total_time: batch_start_time.elapsed().clone(),
        });
    }

    pub fn run(plan: &BackupPlan) -> Result<ExecutorDiscoveryResults, String> {
        let plan_path = plan.path.clone();

        debug!("Executor: Running backup plan {}", plan.name);
        let mut combined_results = ExecutorDiscoveryResults::new(
            0,
            0,
            std::time::Instant::now(),
            std::time::Instant::now(),
            std::time::Instant::now(),
        );
        for batch in &plan.batches {
            let batch_path = plan_path
                .join(".vbatches/")
                .join(batch.to_string() + ".vbak_batch");
            let batch_res = Executor::process_batch(plan, &batch_path);

            match batch_res {
                Ok(res) => {
                    combined_results.files += res.files;
                    combined_results.batches += res.batches;
                    combined_results.batch_time += res.batch_time;
                    combined_results.total_time += res.total_time;
                }
                Err(err) => {
                    error!("Executor: Error processing batch: {:?}", err);
                    return Err(err);
                }
            }
        }
        info!(
            "Executor: Total time to process {} batches with {} files: {}ms",
            combined_results.batches,
            combined_results.files.to_formatted_string(&Locale::en),
            combined_results.total_time.as_millis()
        );
        return Ok(combined_results);
    }
}

#[cfg(test)]
mod executor_tests {
    use std::path::PathBuf;

    use crate::{
        destination::filesystem_dest::FileSystemDestination,
        executor::Executor,
        utils::file_utils::{file_generates_folder, file_remove_all, file_test_dir},
    };

    #[test]
    fn test_discover() {
        let n_files = 200;
        let batch_size = 10;
        let test_dir: std::path::PathBuf = file_test_dir("test_discover".to_string());

        let mut plan = crate::plan::BackupPlan::new("plan__test_discover".to_string());
        let mut source_path = test_dir.clone();
        source_path.push("source");

        let mut dest_path = test_dir.clone();
        dest_path.push("dest");

        let _ = file_generates_folder(&source_path, 100, n_files);

        plan.add_source(Box::new(FileSystemDestination::new(
            source_path.to_str().unwrap().to_string(),
        )));
        plan.add_destination(Box::new(FileSystemDestination::new(
            dest_path.to_str().unwrap().to_string(),
        )));
        plan.save_plan(&test_dir);
        {
            let res = Executor::discover(&mut plan, batch_size);

            match res {
                Ok(res) => {
                    assert_eq!(res.files, n_files);
                    assert_eq!(res.batches, n_files / batch_size as usize);
                }
                Err(err) => {
                    panic!("Error: {:?}", err);
                }
            }
        }

        assert_eq!(plan.batches.len(), n_files / batch_size as usize);

        file_remove_all(&test_dir.clone()).expect("Could not remove dest dir");
    }

    #[test]
    fn test_process_batch() {
        let n_files = 200;
        let batch_size = 10;
        let test_dir: std::path::PathBuf = file_test_dir("test_process_batch".to_string());

        let mut plan = crate::plan::BackupPlan::new("plan__test_process_batch".to_string());
        let mut source_path = test_dir.clone();
        source_path.push("source");

        let mut dest_path = test_dir.clone();
        dest_path.push("dest");

        let _ = file_generates_folder(&source_path, 100, n_files);
        plan.add_source(Box::new(FileSystemDestination::new(
            source_path.to_str().unwrap().to_string(),
        )));
        plan.add_destination(Box::new(FileSystemDestination::new(
            dest_path.to_str().unwrap().to_string(),
        )));
        let plan_res = plan.save_plan(&test_dir);
        {
            let res = Executor::discover(&mut plan, batch_size);

            match res {
                Ok(res) => {
                    assert_eq!(res.files, n_files);
                    assert_eq!(res.batches, n_files / batch_size as usize);
                }
                Err(err) => {
                    panic!("Error: {:?}", err);
                }
            }
        }
        let _ = plan.save_plan(&test_dir);
        let test_batch = PathBuf::from(plan.batches[1].clone());

        // Test plan_res is Ok
        let plan_res = Executor::process_batch(&plan, &test_batch);

        match plan_res {
            Ok(res) => {
                assert_eq!(res.files, batch_size as usize);
                assert_eq!(res.batches, 1);
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        }

        // file_remove_all(&test_dir.clone()).expect("Could not remove dest dir");
    }
}
