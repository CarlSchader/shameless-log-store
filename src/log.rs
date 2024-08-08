pub struct Log {
    pub timestamp: u32,
    pub payload: String, 
}

impl Log {
    pub fn to_line(&self) -> String {
        return String::from(format!("{} {}", self.timestamp, self.payload))
    }
}
