pub struct PageSizeMetrics {
    pub messages_amount: usize,
    pub data_size: usize,
    pub persist_size: usize,
}

impl PageSizeMetrics {
    pub fn new() -> Self {
        Self {
            messages_amount: 0,
            data_size: 0,
            persist_size: 0,
        }
    }

    pub fn append(&mut self, other: &PageSizeMetrics) {
        self.messages_amount += other.messages_amount;
        self.data_size += other.data_size;
        self.persist_size += other.persist_size;
    }

    pub fn update(&mut self, data: &PageSizeMetrics) {
        self.messages_amount = data.messages_amount;
        self.data_size = data.data_size;
        self.persist_size = data.persist_size;
    }
}
