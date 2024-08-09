use std::{cmp, fs, io::{BufRead, BufReader, Read, Seek, SeekFrom}};
use crate::log;

pub struct LogFileHeaders {
    start_timestamp: u64,
    end_timestamp: u64,
    log_count: u64,
}

const TAIL_BUFFER_SIZE: usize = 1024;

pub fn tail_file(mut file: &fs::File, lines: usize, line_offset: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut found_lines: Vec<String> = Vec::new();
    let mut byte_buffer: [u8; TAIL_BUFFER_SIZE] = [0; TAIL_BUFFER_SIZE];
    let mut position = file.seek(SeekFrom::End(0))?;
    let mut line_offset = line_offset;

    while found_lines.len() < lines && position > 0 {
        position = position.saturating_sub(TAIL_BUFFER_SIZE as u64);
        position = file.seek(SeekFrom::Start(cmp::max(0, position)))?;
        let read_count = file.read(&mut byte_buffer)?;
        let mut first_slice_of_buffer = true;

        for slice in byte_buffer[0 .. read_count].split(|byte| *byte == b'\n').rev() {
            let new_line = String::from_utf8(slice.to_vec())?;
            if first_slice_of_buffer {
                first_slice_of_buffer = false;
                if let Some(last) = found_lines.last_mut() {
                    *last += &new_line;
                    continue;
                } 
            }
            if line_offset > 0 {
                line_offset -= 1;
            } else {
                found_lines.push(new_line);
                if found_lines.len() == lines {
                    return Ok(found_lines);
                }
            }
        }
    }

    return Ok(found_lines);
}

pub fn read_headers(file: &fs::File) -> Result<LogFileHeaders, Box<dyn std::error::Error>> {
    let mut buff = BufReader::new(file);
 
    let mut start_timestamp = String::new();
    buff.read_line(&mut start_timestamp)?;
    let start_timestamp: u64 = start_timestamp.trim().parse()?;

    let mut end_timestamp = String::new();
    buff.read_line(&mut end_timestamp)?;
    let end_timestamp: u64 = end_timestamp.trim().parse()?;
    
    let mut log_count = String::new();
    buff.read_line(&mut log_count)?;
    let log_count: u64 = log_count.trim().parse()?;

    return Ok(LogFileHeaders {start_timestamp, end_timestamp, log_count});
}

pub fn read_logs(file: &fs::File, limit: usize, offset: usize) -> Result<Vec<log::Log>, Box<dyn std::error::Error>> {
    let header = read_headers(file)?;
    let mut logs: Vec<log::Log> = Vec::new();

    let lines = tail_file(file, limit, offset)?;
    let mut i = 0;
    while i < lines.len() && logs.len() < limit && ((logs.len() + offset) as u64) < header.log_count {
        logs.push(log::Log::from_string(&lines[i])?);
        i += 1;
    } 
    
    return Ok(logs);
}

pub fn file_string_to_logs(file_string: &String) -> Result<Vec<log::Log>, Box<dyn std::error::Error>> {
    let mut log_vec: Vec<log::Log> = Vec::new();
    let mut start_timestamp: u64 = 0;
    let mut end_timestamp: u64 = 0;
    let mut log_count: usize = 0;

    // these values are the timestamps found in the logs themselves
    // they should be consistant with the header timestamps
    let mut found_start_timestamp: u64 = u64::max_value();
    let mut found_end_timestamp: u64 = 0;
    let mut previous_timestamp: u64 = 0;

    let mut i = 0;
    for line in file_string.split("\n") {
        if i == 0 {
            start_timestamp = line.parse()?;
        } else if i == 1 {
            end_timestamp = line.parse()?;
        } else if i == 2 {
            log_count = line.parse()?;
        } else {
            let new_log = log::Log::from_string(&line.to_string())?;

            if new_log.timestamp < previous_timestamp {
                return Err("logs in file string out of order".into());
            }

            if new_log.timestamp < found_start_timestamp {
                found_start_timestamp = new_log.timestamp;
            }
            if new_log.timestamp > found_end_timestamp {
                found_end_timestamp  = new_log.timestamp;
            }

            previous_timestamp = new_log.timestamp;
            log_vec.push(new_log);
        }

        i += 1;
    }
    
    if log_vec.len() == 0 {
        return Err("file string must have at least one log".into());
    }

    if log_vec.len() != log_count {
        return Err("file string log count line not consistant with the number of logs in the string".into());
    }

    if found_start_timestamp != start_timestamp {
        return Err("header start timestamp isn't consistant with logs".into());
    }
    
    if found_end_timestamp != end_timestamp {
        return Err("header end timestamp isn't consistant with logs".into());
    }

    if end_timestamp < start_timestamp {
        return Err("header end timestamp is less than header start timestamp".into());
    }

    return Ok(log_vec);
}

pub fn logs_to_file_string(logs: &Vec<log::Log>) -> Result<String, Box<dyn std::error::Error>> {
    let log_count = logs.len();
    if log_count == 0 {
        return Err("logs Vec cannot be empty".into());
    }

    let mut start_timestamp = u64::max_value();
    let mut end_timestamp = u64::min_value();
    let mut previous_timestamp = u64::min_value();
    let mut body_string = String::from("");

    for log in logs {
        if log.timestamp > previous_timestamp {
            return Err("logs are out of order".into());
        }

        if log.timestamp < start_timestamp {
            start_timestamp = log.timestamp;
        }

        if log.timestamp > end_timestamp {
            end_timestamp = log.timestamp;
        }

        previous_timestamp = log.timestamp;
        body_string = body_string + &log.to_line();
    }
    
    let mut file_string = format!("{start_timestamp}\n{end_timestamp}\n{log_count}\n");
    file_string = file_string + &body_string;

    return Ok(file_string);
}
