pub struct Log {
    pub timestamp: u32,
    pub payload: String, // need to read up on lifetimes and make this &[u8] byte array
}

impl Log {
    pub fn to_string(&self) -> String {
        return String::from(format!("{} {}", self.timestamp, self.payload))
    }
}
