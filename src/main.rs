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
) -> Result<Vec<(path::PathBuf, GPXTrackLog)>, String> {
    let mut thread_handles = vec![];
    let dir_entries = fs::read_dir(dir_path);
    if dir_entries.is_err() {
        return Err(format!("Error reading directory: {}", dir_path.display()).to_string());
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
        let handle = thread::spawn(move || {
            if entry.is_err() {
                eprintln!("Error reading directory entry");
                thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                return Err("Failed to read directory entry".to_string());
            }
            let entry = entry.unwrap();
            let path = entry.path();
            let dir_info = fs::read_dir(&path);
            match dir_info {
                Ok(_) => {
                    // If it's a directory, scan it recursively
                    println!("Processing directory: {}", path.display());
                    let result = process_directory(&path, &output_dir, parallel_num);
                    if result.is_ok() {
                        thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                        return Ok(result.unwrap());
                    } else {
                        let err_msg = format!(
                            "Error processing directory {}: {}",
                            path.display(),
                            result.unwrap_err()
                        );
                        eprintln!("{}", err_msg);
                        thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                        return Err(err_msg);
                    }
                }
                Err(_) => {
                    let mut results = Vec::new();
                    // If it's a file, process it
                    // let output_path = get_output_path(&path, &output_dir);
                    let logs = process_media_file(&path);
                    if logs.is_err() {
                        let err_msg = format!(
                            "Error processing file {}: {}",
                            path.display(),
                            logs.unwrap_err()
                        );
                        eprintln!("{}", err_msg);
                        thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                        return Err(err_msg);
                    }
                    let logs = logs.unwrap();
                    results.push((path, logs.clone()));

                    // If the last timestamp is more than merge_threshold seconds before the first timestamp, create a new GPX
                    // let gpx = GPX::new(logs);
                    // if gpx.save(&output_path).is_err() {
                    //     eprintln!("Error saving file: {}", &output_path.display());
                    // }

                    thread_count.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                    return Ok(results);
                }
            }
        });
        thread_handles.push(handle);
    }

    while thread_count.load(std::sync::atomic::Ordering::Acquire) > 0 {
        thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut gpx_track_logs = Vec::new();
    for handle in thread_handles {
        // join() は Result<Result<String, io::Error>, _> を返す
        // 最初の unwrap はスレッドのパニックを処理
        // 次の unwrap_or_else は process_file 内の io::Error を処理 (エラー時はファイルパスを表示)
        match handle.join() {
            Ok(Ok(file_content)) => {
                gpx_track_logs.extend(file_content);
            }
            Ok(Err(e)) => {
                eprintln!("Error processing file: {}", e); // ファイル処理エラー
            }
            Err(e) => {
                eprintln!("Thread panicked: {:?}", e); // スレッドパニックエラー
            }
        }
    }

    Ok(gpx_track_logs)
}

fn process_media_file(file_path: &path::PathBuf) -> Result<GPXTrackLog, String> {
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
    // println!("Extracted data from {}", file_path.display());
    Ok(gpx_tracklog)
}

fn main() {
    let args = Cli::parse();
    let output_dir = args.get_output_path();
    let parallel = args.get_parallel_count();
    let merge_enable = args.get_merge_enabled();
    let merge_threshold = args.get_merge_threshold();

    let input_path = args.get_input_path();
    if input_path.is_dir() {
        println!("Processing directory: {}", input_path.display());
        let logs = process_directory(input_path, &output_dir, parallel);
        if logs.is_err() {
            eprintln!("Error processing directory: {}", logs.unwrap_err());
            return;
        }
        let mut logs = logs.unwrap();

        if merge_enable {
            println!("--- Start merging logs ---");
            logs.sort_by(|a, b| a.0.cmp(&b.0));
            let mut current_log = GPXTrackLog::new();
            let mut output_path = path::PathBuf::new(); // FIXME: なんかファイル名決定方法が間違ってる
            for (path, log) in &logs {
                if output_path.as_os_str().is_empty() {
                    output_path = get_output_path(&path, &output_dir);
                    println!("Output changed: {}", output_path.display());
                }
                println!("Merging: {} -> {}", path.display(), output_path.display());
                if current_log.is_empty() {
                    current_log.extend(log.clone());
                } else if let (Some(last_point), Some(first_point)) =
                    (current_log.last(), log.first())
                {
                    // println!(
                    //     "first: {}, last: {}",
                    //     first_point.to_gpx_string(),
                    //     last_point.to_gpx_string()
                    // );

                    // Check if the time difference is within the merge threshold
                    let first_timestamp = first_point.timestamp();
                    if first_timestamp.is_err() {
                        eprintln!(
                            "First point in log {} has no timestamp: {}",
                            path.display(),
                            first_timestamp.unwrap_err()
                        );
                        let gpx = GPX::new(log.clone());
                        if gpx.save(&get_output_path(path, &output_dir)).is_err() {
                            eprintln!("Error saving file: {}", &path.display());
                        } else {
                            println!("Saved GPX file: {}", path.display());
                        }
                        current_log = GPXTrackLog::new();
                        continue;
                    }
                    let first_timestamp = first_timestamp.unwrap();
                    let last_timestamp = last_point.timestamp();
                    if last_timestamp.is_err() {
                        eprintln!("Last point in current log has no timestamp");
                        let gpx = GPX::new(log.clone());
                        if gpx.save(&get_output_path(path, &output_dir)).is_err() {
                            eprintln!("Error saving file: {}", &path.display());
                        } else {
                            println!("Saved GPX file: {}", path.display());
                        }
                        current_log = GPXTrackLog::new();
                        continue;
                    }
                    let last_timestamp = last_timestamp.unwrap();

                    let time_diff = (first_timestamp - last_timestamp).to_std();
                    if time_diff.is_err() {
                        eprintln!(
                            "Error calculating time difference for logs: {} and {}, {}",
                            path.display(),
                            output_path.display(),
                            time_diff.unwrap_err()
                        );
                        let gpx = GPX::new(log.clone());
                        if gpx.save(&get_output_path(path, &output_dir)).is_err() {
                            eprintln!("Error saving file: {}", &path.display());
                        } else {
                            println!("Saved GPX file: {}", path.display());
                        }
                        current_log = GPXTrackLog::new();
                        continue;
                    }
                    let time_diff = time_diff.unwrap();

                    if time_diff <= *merge_threshold {
                        current_log.extend(log.clone());
                    } else {
                        // Save the current log and start a new one
                        let gpx = GPX::new(current_log.clone());
                        if gpx.save(&output_path).is_err() {
                            eprintln!("Error saving file: {}", &output_path.display());
                        } else {
                            println!("Saved GPX file: {}", &output_path.display());
                        }
                        current_log = log.clone();
                        output_path = get_output_path(&path, &output_dir);
                        println!("Output changed: {}", output_path.display());
                    }
                }
            }

            let gpx = GPX::new(current_log.clone());
            if gpx.save(&output_path).is_err() {
                eprintln!("Error saving file: {}", &output_path.display());
            } else {
                println!("Saved GPX file: {}", output_path.display());
            }
        } else {
            for (path, log) in logs {
                let output_path = get_output_path(&path, &output_dir);
                let gpx = GPX::new(log);
                if gpx.save(&output_path).is_err() {
                    eprintln!("Error saving file: {}", &output_path.display());
                } else {
                    println!("Saved GPX file: {}", output_path.display());
                }
            }
        }
    } else {
        let output_path = get_output_path(input_path, &output_dir);
        let logs = process_media_file(input_path)
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
