use clap::Parser;
use rust_htslib::bam::{self, Read};
use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

/// The arguments end up in the Cli struct
#[derive(Parser, Debug)]
#[clap(author, version, about="Tool to extract QC metrics from cram or bam", long_about = None)]
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

    let thread_pool = rust_htslib::tpool::ThreadPool::new(args.threads).unwrap();

    let start_time = Instant::now();
    // Read though once to get exact number of records
    // This could be estimated instead
    let mut bam2 = bam::Reader::from_path(&path).unwrap();
    bam2.set_thread_pool(&thread_pool).unwrap();
    let number_of_records = bam2.records().count();

    // Get the current time after executing the line of code
    let end_time = Instant::now();
    // Calculate the elapsed time
    let elapsed_time = end_time.duration_since(start_time);
    // Print the elapsed time
    println!("Time elapsed: {:?}", elapsed_time);

    // Read for main function
    let mut bam = bam::Reader::from_path(&path).unwrap();
    bam.set_thread_pool(&thread_pool).unwrap();

    // Calculate how many splits
    let split_file_n_times = args.split;
    let records_per_file = number_of_records / split_file_n_times;
    let remainder = number_of_records % split_file_n_times;
    let chunk_size =
        records_per_file + (remainder as f32 / split_file_n_times as f32).ceil() as usize;

    // Put them into a vector
    let mut chunk_index_vector: Vec<usize> = Vec::new();
    for chunk_index in 1..=(number_of_records - 1) / chunk_size + 1 {
        chunk_index_vector.push(chunk_index);
    }

    // Open BAM writers for each unique chunk index
    // Don't want to open a writer for each record
    let mut writers: Vec<Option<Rc<RefCell<bam::Writer>>>> =
        vec![None; (number_of_records - 1) / chunk_size + 1];

    for chunk_index in chunk_index_vector {
        let file_name = format!(
            "{:03}.{}",
            chunk_index,
            path.file_name().unwrap().to_str().unwrap()
        );
        let mut header = bam::Header::from_template(bam.header());

        let args: Vec<String> = env::args().collect();
        let command_line = args.join(" ");

        // Convert string literals to byte slices
        let id = "splitbam";
        let pn = "splitbam";
        let vn = "0.1.0";
        let cl = command_line.as_str();

        // Create a new program group record
        let mut pg_record = bam::header::HeaderRecord::new(b"PG");

        pg_record.push_tag(b"ID", id);
        pg_record.push_tag(b"PN", pn);
        pg_record.push_tag(b"VN", vn);
        pg_record.push_tag(b"CL", cl);

        header.push_record(&pg_record);

        // Create writer
        let bam_writer = {
            let mut writer = bam::Writer::from_path(&file_name, &header, bam::Format::Bam).unwrap();
            let _ = writer.set_thread_pool(&thread_pool);
            writer
        };
        let writer = Rc::new(RefCell::new(bam_writer));
        writers[chunk_index - 1] = Some(writer);
    }

    // Write records to file
    let mut record = rust_htslib::bam::Record::new();
    let mut i = 0;
    while let Some(_r) = bam.read(&mut record) {
        let chunk_index = i / chunk_size + 1;
        let mut writer = writers[chunk_index - 1].as_ref().unwrap().borrow_mut();
        i += 1;
        if let Err(err) = writer.write(&mut record) {
            eprintln!("Error writing record: {}", err);
        }
    }
}
