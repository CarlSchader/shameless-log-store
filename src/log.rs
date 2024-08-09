#[derive(Debug)]
pub struct Log {
    pub timestamp: u64,
    pub payload: String, 
}

impl Log {
    pub fn from_string(input_string: &String) -> Result<Log, String> {
        if input_string.len() == 0 {
            return Err("input string cannot be empty".to_string());
        }

        if let Some(first_space_index) = input_string.find(' ') {
            let timestamp = &input_string[..first_space_index];
            let timestamp:u64 = match timestamp.parse() {
                Err(e) => return Err(format!("couldn't parse timestamp to u64: {timestamp} -- {e}")),
                Ok(x) => x,
            };
            let payload = (&input_string[first_space_index + 1..]).to_string();
            return Ok(Log{timestamp, payload});
        } else {
            let timestamp:u64 = match input_string.parse() {
                Err(e) => return Err(format!("couldn't parse timestamp to u64: {input_string} -- {e}")),
                Ok(x) => x,
            };        
            return Ok(Log{timestamp, payload: String::from("")});
        }
    }

    pub fn to_string(&self) -> String {
        return format!("{} {}", self.timestamp, self.payload);
    }
}
