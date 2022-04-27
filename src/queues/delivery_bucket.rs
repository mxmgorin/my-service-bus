use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;

pub struct DeliveryBucket {
    pub ids: QueueWithIntervals,
    pub confirmed: usize,
}

impl DeliveryBucket {
    pub fn new(ids: QueueWithIntervals) -> Self {
        Self { ids, confirmed: 0 }
    }

    pub fn confirmed(&mut self, confirmed: &QueueWithIntervals) {
        self.confirmed += confirmed.len() as usize;

        for id in confirmed {
            self.ids.remove(id);
        }
    }

    pub fn confirm_everything(&mut self) {
        self.confirmed += self.ids.len() as usize;
        self.ids.clean();
    }
}
