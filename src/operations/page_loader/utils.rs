use my_service_bus_shared::{
    page_id::{PageId, MESSAGES_IN_PAGE},
    MessageId,
};

pub fn get_load_page_interval(
    min_queue_msg_id: MessageId,
    topic_message_id: MessageId,
    page_id: PageId,
) -> (MessageId, MessageId) {
    let first_message_id_in_page = page_id * MESSAGES_IN_PAGE;

    let from_id = if min_queue_msg_id < first_message_id_in_page {
        first_message_id_in_page
    } else {
        min_queue_msg_id
    };

    let first_message_id_in_next_page = first_message_id_in_page + MESSAGES_IN_PAGE;

    let to_id = if topic_message_id < first_message_id_in_next_page {
        topic_message_id
    } else {
        first_message_id_in_next_page - 1
    };

    (from_id, to_id)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_first_page() {
        let min_queue_msg_id = 0;
        let topic_msg_id = 5;

        let page_id = 0;

        let (from_id, to_id) = get_load_page_interval(min_queue_msg_id, topic_msg_id, page_id);

        assert_eq!(0, from_id);
        assert_eq!(5, to_id);
    }

    #[test]
    fn test_second_page_we_have_whole_page_to_load() {
        let min_queue_msg_id = 5;
        let topic_msg_id = MESSAGES_IN_PAGE * 5 + 5;

        let page_id = 1;

        let (from_id, to_id) = get_load_page_interval(min_queue_msg_id, topic_msg_id, page_id);

        assert_eq!(100_000, from_id);
        assert_eq!(199_999, to_id);
    }

    #[test]
    fn test_second_page_we_are_on_that_page() {
        let min_queue_msg_id = 5;
        let topic_msg_id = 155_555;

        let page_id = 1;

        let (from_id, to_id) = get_load_page_interval(min_queue_msg_id, topic_msg_id, page_id);

        assert_eq!(100_000, from_id);
        assert_eq!(155_555, to_id);
    }

    #[test]
    fn test_we_have_messages_within_page() {
        let min_queue_msg_id = 144_444;
        let topic_msg_id = 155_555;

        let page_id = 1;

        let (from_id, to_id) = get_load_page_interval(min_queue_msg_id, topic_msg_id, page_id);

        assert_eq!(144_444, from_id);
        assert_eq!(155_555, to_id);
    }

    #[test]
    fn test_we_have_first_message_within_page_last_message_next_pages() {
        let min_queue_msg_id = 144_444;
        let topic_msg_id = 555_555;

        let page_id = 1;

        let (from_id, to_id) = get_load_page_interval(min_queue_msg_id, topic_msg_id, page_id);

        assert_eq!(144_444, from_id);
        assert_eq!(199_999, to_id);
    }
}
