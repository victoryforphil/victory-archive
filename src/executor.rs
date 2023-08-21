use std::{time::{Duration, Instant}};

use log::{error, debug, info};
use num_format::{ToFormattedString, Locale};

use crate::{plan::BackupPlan, batch::FileBatch};

pub struct Executor{
}

pub struct ExecutorDiscoveryResults{
    pub files: usize,
    pub batches: usize,
    pub batch_time: Duration, 
    pub total_time: Duration,
}

impl ExecutorDiscoveryResults{
    pub fn new(files: usize, batches: usize, start_time: Instant,  batch_time: Instant, total_time: Instant) -> ExecutorDiscoveryResults{
        ExecutorDiscoveryResults{
            files: files,
            batches: batches,
            batch_time: batch_time.duration_since(start_time),
            total_time: total_time.duration_since(start_time),
        }
    }
}

impl Executor{

    pub fn discover(plan: &mut BackupPlan, batch_size:u64) -> Result<ExecutorDiscoveryResults, String>{
        
        // Store start time
        let total_start_time = std::time::Instant::now();
      
        let mut total_files = 0;
        let mut batch_idx = 0;
        //TODO: Multithread this
        for source in &mut plan.sources{
            //TODO: Make ID also show destintation, such as plan_dest_batch..
            loop{
                let batch_start_time = std::time::Instant::now();
                let source = source.as_mut();
                let mut batch = FileBatch::new(plan.name.to_string() + "_" + batch_idx.to_string().as_str());

                let files = match source.list_files_next(batch_size){
                    Ok(files) => files,
                    Err(err) =>  { 
                        error!("list_files_next ERROR: {:?}", err);
                        Vec::new()
                    },
                };

                if files.len() == 0{
                    break;
                }

                batch.add_files(files);
                let batch_end_time = std::time::Instant::now();
                batch_idx += 1;
                total_files += batch.get_length();

                plan.batches.push(batch.get_name());
                
                let batch_path = plan.path.join(".vbatches/").join(batch.get_name().to_string() + ".vbak_batch");
                // Save batch
                let save_size = match batch.save_batch(batch_path.clone()){
                    Ok(res) => res,
                    Err(err) => {
                        error!("save_batch ERROR: {:?}", err);
                        0
                    },
                };
                let batch_save_time = std::time::Instant::now();
                
                debug!("Batch {}:
                    \t- Length: {}
                    \t- Disk size: {} kb
                    \t- Time to discover: {:.2}ms
                    \t- Time to save: {:.2}ms
                    \t- Path: {:?}", 
                    batch.get_name(), (batch.get_length() as u64).to_formatted_string(&Locale::en) ,save_size/ 1024,
                    batch_end_time.duration_since(batch_start_time).as_micros()as f64 / 1000.,
                    batch_save_time.duration_since(batch_end_time).as_micros() as f64 / 1000. , batch_path.clone());
            }
           
        }

        let total_end_time = std::time::Instant::now();

        info!("Total time to discover {} batches with {} files: {}ms", 
            batch_idx, 
            total_files.to_formatted_string(&Locale::en), 
            total_end_time.duration_since(total_start_time).as_millis()
        );

        Ok(ExecutorDiscoveryResults::new(total_files, batch_idx, total_start_time, total_end_time, total_end_time))

    }


}

#[cfg(test)]
mod executor_tests{
    use crate::{utils::file_utils::{file_test_dir, file_generates_folder, file_remove_all}, destination::{filesystem_dest::FileSystemDestination}, executor::Executor};

    #[test]
    fn test_discover(){
        let n_files = 200;
        let batch_size = 10;
        let test_dir: std::path::PathBuf = file_test_dir("test_discover".to_string());


        let mut plan = crate::plan::BackupPlan::new("plan__test_discover".to_string());
        let mut source_path = test_dir.clone();
        source_path.push("source");
       

        let mut dest_path = test_dir.clone();
        dest_path.push("dest");
      

        let _ = file_generates_folder(&source_path, 100, n_files);

        plan.add_source(Box::new(FileSystemDestination::new(source_path.to_str().unwrap().to_string())));
        plan.add_destination(Box::new(FileSystemDestination::new(dest_path.to_str().unwrap().to_string())));
        plan.save_plan(&test_dir);
        {
            let res = Executor::discover(&mut plan, batch_size);

            match res{
            Ok(res) => {
                assert_eq!(res.files, n_files);
                assert_eq!(res.batches, n_files / batch_size as usize);
            },
            Err(err) => {
                panic!("Error: {:?}", err);
            }
            }
        }

        assert_eq!(plan.batches.len(), n_files / batch_size as usize);

        file_remove_all(&test_dir.clone()).expect("Could not remove dest dir");
        

    }
}