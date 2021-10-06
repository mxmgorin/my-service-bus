use my_service_bus_shared::{date_time::DateTimeAsMicroseconds, debug::LockItem};

use crate::{
    app::AppContext,
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
        let date = DateTimeAsMicroseconds::new(itm.date);
        result.append_line(
            format!("{} {} [{}]", date.to_rfc3339(), itm.to_string(), itm.id,).as_str(),
        );
    }

    result.to_string_utf8().unwrap()
}
