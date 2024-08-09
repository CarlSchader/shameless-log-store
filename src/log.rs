#[derive(Debug)]
#[derive(Clone)]
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

pub fn merge_logs(mut logs_a: Vec<Log>, mut logs_b: Vec<Log>) -> Result<Vec<Log>, String> {
    let mut returned_logs = vec![Log{timestamp: 0, payload: "".to_string()}; logs_a.len() + logs_b.len()];
    let mut i = returned_logs.len() - 1;
    let mut previous_timestamp = u64::max_value();

    while logs_a.len() > 0 && logs_b.len() > 0 {
        let next_log: Log;
        if logs_b.last().unwrap().timestamp < logs_a.last().unwrap().timestamp {
            next_log = logs_b.pop().unwrap();
        } else {
            next_log = logs_a.pop().unwrap();
        }
        
        if next_log.timestamp > previous_timestamp {
            return Err("logs are out of order".to_string());            
        }

        previous_timestamp = next_log.timestamp;
        returned_logs[i] = next_log;
        i = i.saturating_sub(1);
    }

    return Ok(returned_logs);
}
