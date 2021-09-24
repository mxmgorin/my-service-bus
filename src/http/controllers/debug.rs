use my_service_bus_shared::debug::LockItem;

use crate::{
    app::AppContext,
    date_time::MyDateTime,
    http::{HttpFailResult, HttpOkResult},
    utils::StringBuilder,
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let logs = app.get_locks().await;

    let text = compile_result(&logs);

    Ok(HttpOkResult::Text { text })
}

fn compile_result(items: &[LockItem]) -> String {
    let mut result = StringBuilder::new();

    for itm in items {
        let date = MyDateTime::new(itm.date);
        result.append_line(format!("{} {} [{}]", date.to_iso_string(), itm.data, itm.id,).as_str());
    }

    result.to_string_utf8().unwrap()
}
