use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub command: String,
    pub args: Vec<String>,
}
