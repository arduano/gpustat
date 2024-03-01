use std::collections::VecDeque;

pub struct GraphViewerData {
    historical: VecDeque<Option<f32>>,
}

impl GraphViewerData {
    pub fn new() -> Self {
        Self {
            historical: VecDeque::new(),
        }
    }

    pub fn update(&mut self, value: Option<f32>) {
        self.historical.push_front(value);
        self.trim_length()
    }

    fn trim_length(&mut self) {
        let max_len = 5000;

        while self.historical.len() > max_len {
            self.historical.pop_back();
        }
    }

    pub fn get_value_at(&self, index: usize) -> Option<f32> {
        self.historical.get(index).copied().flatten()
    }
}
