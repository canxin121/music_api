use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistUpdateSubscriptionResult {
    pub errors: Vec<(String, String)>,
}
