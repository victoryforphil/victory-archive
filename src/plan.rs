use std::{path::{Path, PathBuf}, io::Write};

use log::*;
use serde::{Serialize, Deserialize};

use crate::{destination::{Destination, filesystem_dest::FileSystemDestination}, batch::{FileBatch}};
use num_format::{Locale, ToFormattedString};


pub struct BackupPlan{
    pub name: String,
    pub sources: Vec<Box<dyn Destination>>,
    pub batches: Vec<String>
    
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq,)]
pub struct BackupPlanSave{
    pub name: String,
    pub sources: Vec<String>,
    pub batches: Vec<String>,
    
}
impl BackupPlan{
    pub fn new(name: String) -> BackupPlan{
        BackupPlan{
            name: name,
            sources: Vec::new(),
            batches: Vec::new(),
        }
    }

    pub fn from_saved(plan: BackupPlanSave) -> BackupPlan{
        let mut sources = Vec::new();
        //TODO: Dynamic way (enums?) to store what destinations are used
        for source in plan.sources{
            sources.push(Box::new(FileSystemDestination::new(source)) as Box<dyn Destination>);
        }
        let mut batches = Vec::new();
        for batch in plan.batches{
            batches.push(batch);
        }
        BackupPlan{
            name: plan.name,
            sources: sources,
            batches: batches,
        }
    }

    pub fn get_saved(&self) -> BackupPlanSave{
        let mut sources = Vec::new();
        for source in &self.sources{
            sources.push(source.get_name());
        }
        let mut batches = Vec::new();
        for batch in &self.batches{
            batches.push(batch.clone());
        }
        BackupPlanSave{
            name: self.name.clone(),
            sources: sources,
            batches: batches,
        }
    }
    

    pub fn add_source(&mut self, source: Box<dyn Destination>){
        self.sources.push(source);
    }

    pub fn save_plan(&self, path: PathBuf) -> Result<usize, String>{
        let mut file = match std::fs::File::create(path){
            Ok(file) => file,
            Err(err) => return Err(format!("Error: {:?}", err)),
        };
        let yaml = serde_yaml::to_string(&self.get_saved()).expect("Error serializing plan");
        match file.write_all(yaml.as_bytes()){
            Ok(_) => Ok(yaml.len()),
            Err(err) => Err(format!("plan_save_error: {:?}", err)),
        }
    }

    pub fn discover(&mut self, batch_size: u64, save_path: String) -> Result<(), String>{

        // Store start time
        let total_start_time = std::time::Instant::now();
      
        let mut total_files = 0;
        let mut batch_idx = 0;
        //TODO: Multithread this
        for source in &mut self.sources{
            //TODO: Make ID also show destintation, such as plan_dest_batch..
            loop{
                let batch_start_time = std::time::Instant::now();
                let mut batch = FileBatch::new(self.name.to_string() + "_" + batch_idx.to_string().as_str());
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

                self.batches.push(batch.get_name());
                
                let batch_path = Path::new(save_path.as_str()).join(batch.get_name().to_string() + ".vbak_batch");
                // Save batch
                let save_size = match batch.save_batch(batch_path){
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
                    batch_save_time.duration_since(batch_end_time).as_micros() as f64 / 1000. , save_path.clone());
            }
           
        }

        let total_end_time = std::time::Instant::now();
        info!("Total time to discover {} batches with {} files: {}ms", batch_idx, total_files.to_formatted_string(&Locale::en), total_end_time.duration_since(total_start_time).as_millis());
        Ok(())
    }

    
}