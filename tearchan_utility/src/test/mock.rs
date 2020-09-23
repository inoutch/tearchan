pub struct MockFunc {
    calls: Vec<Vec<String>>,
}

impl MockFunc {
    pub fn new() -> MockFunc {
        MockFunc { calls: vec![] }
    }

    pub fn call(&mut self, logs: Vec<String>) {
        self.calls.push(logs);
    }

    pub fn logs(&self, index: usize) -> Option<&Vec<String>> {
        self.calls.get(index)
    }

    pub fn print_logs(&self) {
        self.calls.iter().for_each(|x| {
            println!("{}", x.join(", "));
        });
    }

    pub fn clear(&mut self) {
        self.calls.clear();
    }
}
