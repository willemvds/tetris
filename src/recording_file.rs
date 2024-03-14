use crate::tetris::recordings;
use crate::tetris::rules;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingFile {
    pub recording: recordings::Recording,
    pub rules: rules::Rules,
}

impl RecordingFile {
    pub fn new(rules: rules::Rules, recording: recordings::Recording) -> RecordingFile {
        RecordingFile { recording, rules }
    }
}
