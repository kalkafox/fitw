pub struct App {
    pub hash: String,
    pub date: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            hash: String::new(),
            date: String::new(),
        }
    }
}
