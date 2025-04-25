use serde::{ Serialize, Deserialize };
// Response structures
#[derive(Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

// Request structures
#[derive(Deserialize)]
pub struct ArduinoCommand {
    pub command: String,
    pub args: Vec<String>,
}
