use crate::tetris::recordings;
use crate::tetris::rules;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingFile {
    pub version: u32,
    pub recording: recordings::Recording,
    pub rules: rules::Rules,
    pub final_score: u32,
    pub final_lines_cleared: u32,
}

impl RecordingFile {
    pub fn new(
        version: u32,
        rules: rules::Rules,
        recording: recordings::Recording,
        final_score: u32,
        final_lines_cleared: u32,
    ) -> RecordingFile {
        RecordingFile {
            version,
            recording,
            rules,
            final_score,
            final_lines_cleared,
        }
    }
}
