use clap::Parser;
use rust_htslib::bam::{self, Read, Writer};
use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

/// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[clap(author, version, about = "Tool to split one ubam file into multiple", long_about = None)]
struct Args {
    /// bam file to split
    #[clap(value_parser)]
    input: String,

    /// Number of parallel decompression & writer threads to use
    #[clap(short, long, value_parser, default_value_t = 4)]
    threads: u32,

    /// Number of files to split bam to
    #[clap(short, long, value_parser)]
    split: usize,
}

fn main() {
    let args = Args::parse();
    let path = Path::new(&args.input);
    let threads = args.threads; 
    let thread_pool_first = rust_htslib::tpool::ThreadPool::new(args.threads).unwrap();

    let start_time = Instant::now();
    // Read through once to get the exact number of records
    let mut bam2 = bam::Reader::from_path(&path).unwrap();
    bam2.set_thread_pool(&thread_pool_first).unwrap();
    let number_of_records = bam2.records().count();

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("Time elapsed: {:?}", elapsed_time);

    let mut bam = bam::Reader::from_path(&path).unwrap();

    let split_file_n_times = args.split;
    let records_per_file = number_of_records / split_file_n_times;
    let remainder = number_of_records % split_file_n_times;
    let chunk_size = records_per_file + (remainder as f32 / split_file_n_times as f32).ceil() as usize;

    // Vector to hold senders for each thread
    let mut senders = Vec::new();
    let mut handles = Vec::new();
    
    let thread_count = (threads as f32 / split_file_n_times as f32).ceil() as u32;

    for chunk_index in 0..split_file_n_times {
        let file_name = format!(
            "{:03}.{}",
            chunk_index + 1,
            path.file_name().unwrap().to_str().unwrap()
        );

        let mut header = bam::Header::from_template(bam.header());

        let args: Vec<String> = env::args().collect();
        let command_line = args.join(" ");
        let id = "splitbam";
        let pn = "splitbam";
        let vn = "0.1.0";
        let cl = command_line.as_str();

        let mut pg_record = bam::header::HeaderRecord::new(b"PG");
        pg_record.push_tag(b"ID", id);
        pg_record.push_tag(b"PN", pn);
        pg_record.push_tag(b"VN", vn);
        pg_record.push_tag(b"CL", cl);

        header.push_record(&pg_record);

        // Create a separate channel for each thread
        let (tx, rx) = mpsc::channel();
        senders.push(tx);

        let bam_header = Arc::new(header);
        let handle = thread::spawn(move || {
            let writer_path = file_name;
            let mut bam_writer = Writer::from_path(writer_path, &bam_header, bam::Format::Bam).unwrap();
            let local_thread_pool = rust_htslib::tpool::ThreadPool::new(thread_count).unwrap(); // Create a separate ThreadPool for each thread
            let _ = bam_writer.set_thread_pool(&local_thread_pool);

            // Loop to receive and write records
            while let Ok(record) = rx.recv() {
                if let Err(err) = bam_writer.write(&record) {
                    eprintln!("Error writing record: {}", err);
                }
            }
        });

        handles.push(handle);
    }

    // This block is executed once to distribute records to threads
    let mut record = bam::Record::new();
    let mut i = 0;
    while let Some(Ok(_)) = bam.read(&mut record) {
        let chunk_index = i / chunk_size;
        let chunk_sender = &senders[chunk_index];
        chunk_sender.send(record.clone()).unwrap();
        i += 1;
    }

    // Close all senders by dropping them
    drop(senders);

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}
