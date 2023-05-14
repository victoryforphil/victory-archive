use log::info;

use crate::{destination::Destination, batch::{FileBatch, self}};


pub struct BackupPlan{
    pub name: String,

    pub sources: Vec<Box<dyn Destination>>,
    pub batches: Vec<String>
    
}

impl BackupPlan{
    pub fn new(name: String) -> BackupPlan{
        BackupPlan{
            name: name,
            sources: Vec::new(),
            batches: Vec::new(),
        }
    }


    pub fn add_source(&mut self, source: Box<dyn Destination>){
        self.sources.push(source);
    }

    pub fn discover(&mut self, batch_size: u64 ) -> Result<(), String>{

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
                    Err(err) => return Err(format!("Error: {:?}", err)),
                };
                if files.len() == 0{
                    break;
                }
                batch.add_files(files);
                let batch_end_time = std::time::Instant::now();
                batch_idx += 1;
                total_files += batch.get_length();

                self.batches.push(batch.get_name());
                
                info!("Batch {} discovered with: {} files in {}ms", batch.get_name(), batch.get_length(), batch_end_time.duration_since(batch_start_time).as_millis());
            }
           
        }

        let total_end_time = std::time::Instant::now();
        info!("Total time to discover {} batches with {} files: {}ms", batch_idx, total_files, total_end_time.duration_since(total_start_time).as_millis());
        Ok(())
    }

    
}