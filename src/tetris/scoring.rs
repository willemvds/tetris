use serde;
use typetag;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    OriginalBPS,
    OriginalSega,
    OriginalNintendo,
    DS,
}

#[typetag::serde(tag = "type")]
pub trait System {
    fn lines_cleared(&mut self, level: u8, lines: u8) -> u32;
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OriginalBPS {
    points: u32,
    lines_cleared: u32,
}

impl OriginalBPS {
    pub fn new() -> OriginalBPS {
        OriginalBPS {
            points: 0,
            lines_cleared: 0,
        }
    }
}

#[typetag::serde]
impl System for OriginalBPS {
    fn lines_cleared(&mut self, _level: u8, lines: u8) -> u32 {
        let points = match lines {
            0 => 0,
            1 => 40,
            2 => 100,
            3 => 300,
            _ => 1200,
        };
        self.points += points;

        points
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OriginalSega {
    points: u32,
    lines_cleared: u32,
}

impl OriginalSega {
    pub fn new() -> OriginalSega {
        OriginalSega {
            points: 0,
            lines_cleared: 0,
        }
    }
}

#[typetag::serde]
impl System for OriginalSega {
    fn lines_cleared(&mut self, level: u8, count: u8) -> u32 {
        let points = match level {
            1..=2 => match count {
                1 => 100,
                2 => 400,
                3 => 900,
                4 => 2000,
                _ => 0,
            },
            3..=4 => match count {
                1 => 200,
                2 => 800,
                3 => 1800,
                4 => 4000,
                _ => 0,
            },
            5..=6 => match count {
                1 => 300,
                2 => 1200,
                3 => 2700,
                4 => 6000,
                _ => 0,
            },
            7..=8 => match count {
                1 => 400,
                2 => 1600,
                3 => 3600,
                4 => 8000,
                _ => 0,
            },
            _ => match count {
                1 => 500,
                2 => 2000,
                3 => 4500,
                4 => 10000,
                _ => 0,
            },
        };

        self.points += points;

        points
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OriginalNintendo {
    points: u32,
    lines_cleared: u32,
}

impl OriginalNintendo {
    pub fn new() -> OriginalNintendo {
        OriginalNintendo {
            points: 0,
            lines_cleared: 0,
        }
    }
}

#[typetag::serde]
impl System for OriginalNintendo {
    fn lines_cleared(&mut self, level: u8, lines: u8) -> u32 {
        let points = match lines {
            1 => 40 * level as u32,
            2 => 100 * level as u32,
            3 => 300 * level as u32,
            4 => 1200 * level as u32,
            _ => 0,
        };
        self.points += points;

        points
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DS {
    points: u32,
    lines_cleared: u32,
}

impl DS {
    pub fn new() -> DS {
        DS {
            points: 0,
            lines_cleared: 0,
        }
    }
}

#[typetag::serde]
impl System for DS {
    fn lines_cleared(&mut self, level: u8, lines: u8) -> u32 {
        let points = match lines {
            1 => 800 * level as u32,
            2 => 1200 * level as u32,
            3 => 1800 * level as u32,
            4 => 2000 * level as u32,
            _ => 0,
        };
        self.points += points;

        points
    }
}
