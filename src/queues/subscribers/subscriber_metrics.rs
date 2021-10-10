use std::{sync::Arc, time::Duration};

use my_service_bus_shared::date_time::DateTimeAsMicroseconds;

use crate::{
    metric_data::{MetricOneSecond, MetricsHistory},
    queues::TopicQueue,
    tcp::tcp_server::ConnectionId,
    topics::Topic,
};

use super::SubscriberId;

pub const DELIVERY_MODE_READY_TO_DELIVER: u8 = 0;
pub const DELIVERY_MODE_RENTED: u8 = 1;
pub const DELIVERY_MODE_ON_DELIVERY: u8 = 2;

#[derive(Clone)]
pub struct SubscriberMetrics {
    pub topic: Arc<Topic>,
    pub queue: Arc<TopicQueue>,
    pub start_delivery_time: DateTimeAsMicroseconds,
    pub delivered_amount: MetricOneSecond,
    pub delivery_microseconds: MetricOneSecond,
    pub active: u8,
    pub delivery_history: MetricsHistory,

    pub connection_id: ConnectionId,
    pub subscriber_id: SubscriberId,

    pub delivery_mode: u8,
}

impl SubscriberMetrics {
    pub fn new(
        subscriber_id: SubscriberId,
        connection_id: ConnectionId,
        topic: Arc<Topic>,
        queue: Arc<TopicQueue>,
    ) -> Self {
        Self {
            subscriber_id,
            start_delivery_time: DateTimeAsMicroseconds::now(),
            delivered_amount: MetricOneSecond::new(),
            delivery_microseconds: MetricOneSecond::new(),
            active: 0,
            delivery_history: MetricsHistory::new(),
            connection_id,
            topic,
            queue,
            delivery_mode: DELIVERY_MODE_READY_TO_DELIVER,
        }
    }

    pub fn one_second_tick(&mut self) {
        if self.active > 0 {
            self.active -= 1;
        }

        let delivered_amount = self.delivered_amount.get_and_reset();
        let delivery_microseconds = self.delivery_microseconds.get_and_reset();

        if delivered_amount > 0 {
            let delivered = delivery_microseconds / delivered_amount;
            self.delivery_history.put(delivered as i32);
        }
    }

    pub fn set_delivered_statistic(
        &mut self,
        delivered_messages: usize,
        delivery_duration: Duration,
    ) {
        self.delivery_mode = DELIVERY_MODE_READY_TO_DELIVER;
        self.delivered_amount.increase(delivered_messages);
        self.delivery_microseconds
            .increase(delivery_duration.as_micros() as usize);
    }

    pub fn set_not_delivered_statistic(
        &mut self,

        delivered_messages: i32,
        delivery_duration: Duration,
    ) {
        self.delivery_mode = DELIVERY_MODE_READY_TO_DELIVER;
        let value = delivery_duration.as_micros() as i32 / -delivered_messages;
        self.delivery_history.put(value);
    }

    pub fn set_started_delivery(&mut self) {
        self.start_delivery_time = DateTimeAsMicroseconds::now();
        self.active = 2;
        self.delivery_mode = DELIVERY_MODE_ON_DELIVERY;
    }
}
