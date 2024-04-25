use std::collections::HashMap;
use rust_htslib::bam::{self, Read, Writer};
use std::cell::RefCell;
use std::rc::Rc;

use rayon::prelude::*;

use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
   
    let path = "test/test.bam";

    let read_thread_pool = rust_htslib::tpool::ThreadPool::new(36).unwrap();
    
    let mut bam = bam::Reader::from_path(&path).unwrap();
    bam.set_thread_pool(&read_thread_pool).unwrap();
    
    let header = bam::Header::from_template(bam.header());
    let start_time = Instant::now();

    let number_of_records = bam.records().fold(0, |count, _| count + 1);

    
    let split_file_n_times = 10;
    // Get the current time after executing the line of code
    let end_time = Instant::now();
    // Calculate the elapsed time
    let elapsed_time = end_time.duration_since(start_time);
    // Print the elapsed time
     println!("Time elapsed: {:?}", elapsed_time);
                        
    let records_per_file = number_of_records / split_file_n_times;
    let remainder = number_of_records % split_file_n_times;
    let chunk_size = records_per_file + (remainder as f32 / split_file_n_times as f32).ceil() as usize;
    let chunk_size = 1000000;
    
    let mut chunk_index_vector: Vec<usize> = Vec::new();
    for chunk_index in 1..=(number_of_records - 1) / chunk_size + 1 {
        chunk_index_vector.push(chunk_index);
    }
    let thread_pool = rust_htslib::tpool::ThreadPool::new(36).unwrap();

    // Open BAM writers for each unique chunk index
    let mut writers: Vec<Option<Rc<RefCell<bam::Writer>>>> = vec![None; (number_of_records - 1) / chunk_size + 1];
    for chunk_index in chunk_index_vector {
        println!("{} {} {}", chunk_index, chunk_size, number_of_records);
        

        let file_name = format!("chunk_{}.bam", chunk_index);
        let header = bam::Header::from_template(bam.header());
        // ADD PG LINE
        let bam_writer = {
            let mut writer = bam::Writer::from_path(&file_name, &header, bam::Format::Bam).unwrap();
            writer.set_thread_pool(&thread_pool);
            writer
        };
        let writer = Rc::new(RefCell::new(bam_writer 
        ));
        writers[chunk_index - 1] = Some(writer);
        
    }
    bam.records().enumerate().for_each(|(i,r)| {
        let chunk_index = i / chunk_size + 1;
        let mut writer = writers[chunk_index - 1].as_ref().unwrap().borrow_mut();
        let record = r.unwrap();
        println!("{}", record.tid());
        if let Err(err) = writer.write(&record) {
                    eprintln!("Error writing record: {}", err);
                            // Handle error as needed
                                 }

    });
    /*for (i, r) in bam.records().enumerate() {
        
        let record = r.unwrap();
        out.write(&record).unwrap();
    }*/

}
