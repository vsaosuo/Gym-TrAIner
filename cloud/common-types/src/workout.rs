use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Feedback {
    pub class: String,
    pub correction: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WorkoutType {
    Squat,
    Pushup,
}
