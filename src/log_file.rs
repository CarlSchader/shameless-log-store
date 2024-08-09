use std::{cmp, fs, io::{Read, Seek, SeekFrom}};

const TAIL_BUFFER_SIZE: usize = 1024;

pub fn tail_file(mut file: &fs::File, lines: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut found_lines: Vec<String> = Vec::new();
    let mut byte_buffer: [u8; TAIL_BUFFER_SIZE] = [0; TAIL_BUFFER_SIZE];
    let mut position = file.seek(SeekFrom::End(0))?;

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
            found_lines.push(new_line);
        }
    }

    return Ok(found_lines);
}
