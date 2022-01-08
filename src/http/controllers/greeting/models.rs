use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GreetingJsonResult {
    pub session: String,
}
