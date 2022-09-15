pub mod log;
pub mod utils;

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use log::Log;
use utils::*;

use chrono::NaiveTime;
use structopt::StructOpt;
use threadpool::ThreadPool;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    #[structopt(default_value = "v1", short = "v", long = "version")]
    version: String,
    #[structopt(default_value = "20", short = "l", long = "limit")]
    limit: i32,
    #[structopt(short = "it", long = "init_time", parse(try_from_str = from_time))]
    init_time: Option<NaiveTime>,
    #[structopt(short = "et", long = "end_time", parse(try_from_str = from_time))]
    end_time: Option<NaiveTime>,
}

fn main() {
    let ncpus = num_cpus::get();
    let thread_pool = ThreadPool::new(ncpus * 2);
    // let time = Instant::now();
    let args = Cli::from_args();
    let log_map = Arc::new(Mutex::new(HashMap::<usize, usize>::new()));
    let mut file_paths = Vec::<PathBuf>::new();

    println!("-- Checking files to process --");
    file_paths = get_log_paths(file_paths, &args.path, &args.init_time, &args.end_time);

    println!("-- Starting to process {} files --", file_paths.len());
    for path in file_paths {
        let all_logs = Arc::clone(&log_map);

        thread_pool.execute(move || {
            let mut thread_hist = HashMap::<usize, usize>::new();

            let f = fs::read_to_string(path).expect("could not read file");

            f.lines().for_each(|line| {
                // TODO: Support parsing by version
                let log = line.split(' ').collect::<Log>();

                // Hardcoding bins for the moment
                if log.target_processing_time > 0.0 && log.target_processing_time < 5.0 {
                    thread_hist.entry(0).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                } else if log.target_processing_time > 5.0 && log.target_processing_time < 10.0 {
                    thread_hist.entry(5).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                } else if log.target_processing_time > 10.0 && log.target_processing_time < 15.0 {
                    thread_hist.entry(10).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                } else if log.target_processing_time > 15.0 && log.target_processing_time < 20.0 {
                    thread_hist.entry(15).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                } else if log.target_processing_time > 20.0 && log.target_processing_time < 25.0 {
                    thread_hist.entry(20).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                } else if log.target_processing_time > 25.0 && log.target_processing_time < 30.0 {
                    thread_hist.entry(25).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                    // println!("{:?}", log);
                } else if log.target_processing_time > 30.0 {
                    thread_hist.entry(30).and_modify(|e: &mut usize| *e += 1).or_insert(1);
                    // println!("{:?}", log);
                };

            });

            for (key, value) in thread_hist.iter() {
                all_logs
                    .lock()
                    .unwrap()
                    .entry(key.to_owned())
                    .and_modify(|e| *e += value)
                    .or_insert(*value);
            }
        });
    }

    thread_pool.join();

    let all_logs = log_map.lock().unwrap();
    let mut count_vec: Vec<(&usize, &usize)> = all_logs.iter().collect();
    count_vec.sort_by(|a, b| a.1.cmp(b.1));

    let mut total: usize = 0;
    for (threshold, count) in count_vec.iter() {
        println!("{} - {}", threshold, count);
        total += **count;

    }

    println!("total: {}", total);
}
