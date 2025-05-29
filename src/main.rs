mod cli;
mod container;
mod gpx;
mod zdr055;

use std::fs::{self};
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::{path, thread};

use cli::Cli;
use gpx::track_log::GPXTrackLog;
use gpx::GPX;
use zdr055::{ZDR055MediaData, ZDR055PositionData};

fn get_output_path(in_file: &path::PathBuf, out_dir: &path::PathBuf) -> path::PathBuf {
    let filename = in_file.file_stem().unwrap().to_str().unwrap();
    out_dir.join(format!("{}.gpx", filename))
}

fn process_directory(
    dir_path: &path::PathBuf,
    output_dir: &path::PathBuf,
    parallel_num: usize,
    merge_threshold: humantime::Duration,
) {
    let dir_entries = fs::read_dir(dir_path);
    if dir_entries.is_err() {
        eprintln!("Error reading directory: {}", dir_path.display());
        return;
    }
    let dir_entries = dir_entries.unwrap();
    let thread_count = std::sync::Arc::new(AtomicUsize::new(0));
    for entry in dir_entries {
        // limit parallel processing count by parallel_num

        let output_dir = output_dir.clone();
        let thread_count = std::sync::Arc::clone(&thread_count);
        while parallel_num > 0
            && thread_count.load(std::sync::atomic::Ordering::Acquire) >= parallel_num
        {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        let thread_count = std::sync::Arc::clone(&thread_count);
        thread_count.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        thread::spawn(move || {
            if entry.is_err() {
                eprintln!("Error reading directory entry");
                thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                return;
            }
            let entry = entry.unwrap();
            let path = entry.path();
            let dir_info = fs::read_dir(&path);
            let mut tail_timestamp: Option<chrono::DateTime<chrono_tz::Tz>> = None;
            // let mut gpx;
            let mut tracklogs = GPXTrackLog::new();
            match dir_info {
                Ok(_) => {
                    // If it's a directory, scan it recursively
                    println!("Processing directory: {}", path.display());
                    process_directory(&path, &output_dir, parallel_num, merge_threshold);
                    thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                    return;
                }
                Err(_) => {
                    // If it's a file, process it
                    let output_path = get_output_path(&path, &output_dir);
                    let logs = process_file(&path);
                    if logs.is_err() {
                        eprintln!("Error processing file: {}", logs.unwrap_err());
                        thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                        return;
                    }
                    let logs = logs.unwrap();
                    let last_trackpoint = logs.last();
                    if last_trackpoint.is_none() {
                        eprintln!("No trackpoints found in file: {}", path.display());
                        thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                        return;
                    }

                    if !tail_timestamp.is_none() {
                        let head = logs.first();
                        if head.is_none() {
                            eprintln!("No trackpoints found in file: {}", path.display());
                            thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                            return;
                        }

                        let head_timestamp = head.unwrap().timestamp().unwrap();

                        let merge_chrono_duration =
                            chrono::Duration::from_std(merge_threshold.into()).unwrap();
                        if tail_timestamp.unwrap() + merge_chrono_duration < head_timestamp {
                            // If the last timestamp is more than merge_threshold seconds before the first timestamp, create a new GPX
                            let gpx = GPX::new(tracklogs);
                            if gpx.save(&output_path).is_err() {
                                eprintln!("Error saving file: {}", &output_path.display());
                            }
                        } else {
                            // Otherwise, merge with the existing GPX
                            tracklogs.extend(logs.clone());
                        }
                    } else {
                        // If this is the first file, initialize tracklogs
                        tracklogs = logs.clone();
                    }
                    tail_timestamp = Some(last_trackpoint.unwrap().timestamp().unwrap());

                    // let gpx = GPX::new(logs);
                    thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                }
            }
        });
    }

    while thread_count.load(std::sync::atomic::Ordering::Acquire) > 0 {
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn process_file(file_path: &path::PathBuf) -> Result<GPXTrackLog, String> {
    println!("Processing file: {}", file_path.display());
    let mut gpx_tracklog = GPXTrackLog::new();

    let file = ZDR055MediaData::new(file_path.to_str().unwrap());
    let stream_data = file
        .extract_stream_data()
        .map_err(|e| format!("Failed to extract stream data: {}", e))?;

    for line in stream_data.iter() {
        // line は ZDR055 独自ログデータなので ZDR055PositionData に変換する
        let log = ZDR055PositionData::from_str(line)
            .map_err(|e| format!("Failed to parse line: {}", e))?;

        // GPX 形式に変換して gpx_points に追加する
        let gpx_point = log.to_gpx_point();
        gpx_tracklog.push(gpx_point);
    }
    println!("Extracted data from {}", file_path.display());
    Ok(gpx_tracklog)
}

fn main() {
    let args = Cli::parse();
    let output_dir = args.get_output_path();
    let parallel = args.get_parallel_count();

    let input_path = args.get_input_path();
    if input_path.is_dir() {
        println!("Processing directory: {}", input_path.display());
        process_directory(input_path, &output_dir, parallel);
    } else {
        let output_path = get_output_path(input_path, &output_dir);
        let logs = process_file(input_path)
            .map_err(|e| {
                eprintln!("Error processing file: {}", e);
                e
            })
            .unwrap();

        let gpx = GPX::new(logs);
        if gpx.save(&output_path).is_err() {
            eprintln!("Error saving file: {}", &output_path.display());
        }
    }
}
