#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Kind {
    Stick,
    Seven,
    Hook,
    Square,
    Snake,
    Pyramid,
    Zig,
}

type Form = [[u8; 4]; 4];

#[derive(Debug)]
pub struct Tetromino {
    pub kind: Kind,
    forms: [Form; 4],
}

#[rustfmt::skip]
const STICK: Tetromino = Tetromino{
    kind: Kind::Stick,
    forms: [
        [
            [0,0,0,0],
            [1,1,1,1],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,0,1,0],
            [0,0,1,0],
            [0,0,1,0],
            [0,0,1,0],
        ],
        [
            [0,0,0,0],
            [0,0,0,0],
            [1,1,1,1],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
        ],
    ],
};

#[rustfmt::skip]
const SEVEN: Tetromino = Tetromino{
    kind: Kind::Seven,
    forms: [
        [
            [0,0,1,0],
            [1,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [0,1,0,0],
            [0,1,1,0],
            [0,0,0,0],
        ],
        [
            [0,0,0,0],
            [1,1,1,0],
            [1,0,0,0],
            [0,0,0,0],
        ],
        [
            [1,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
    ]
};

#[rustfmt::skip]
const HOOK: Tetromino = Tetromino{
    kind: Kind::Hook,
    forms: [
        [
            [1,0,0,0],
            [1,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,1,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
        [
            [0,0,0,0],
            [1,1,1,0],
            [0,0,1,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [0,1,0,0],
            [1,1,0,0],
            [0,0,0,0],
        ],
    ]
};

#[rustfmt::skip]
const SQUARE: Tetromino = Tetromino{
    kind: Kind::Square,
    forms: [
        [
            [0,1,1,0],
            [0,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,1,0],
            [0,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,1,0],
            [0,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,1,0],
            [0,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
    ]
};

#[rustfmt::skip]
const SNAKE: Tetromino = Tetromino{
    kind: Kind::Snake,
    forms: [
        [
            [0,1,1,0],
            [1,1,0,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [0,1,1,0],
            [0,0,1,0],
            [0,0,0,0],
        ],
        [
            [0,0,0,0],
            [0,1,1,0],
            [1,1,0,0],
            [0,0,0,0],
        ],
        [
            [1,0,0,0],
            [1,1,0,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
    ]
};

#[rustfmt::skip]
const PYRAMID: Tetromino = Tetromino{
    kind: Kind::Pyramid,
    forms: [
        [
            [0,1,0,0],
            [1,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [0,1,1,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
        [
            [0,0,0,0],
            [1,1,1,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [1,1,0,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
    ]
};

#[rustfmt::skip]
const ZIG: Tetromino = Tetromino{
    kind: Kind::Zig,
    forms: [
        [
            [1,1,0,0],
            [0,1,1,0],
            [0,0,0,0],
            [0,0,0,0],
        ],
        [
            [0,0,1,0],
            [0,1,1,0],
            [0,1,0,0],
            [0,0,0,0],
        ],
        [
            [0,0,0,0],
            [1,1,0,0],
            [0,1,1,0],
            [0,0,0,0],
        ],
        [
            [0,1,0,0],
            [1,1,0,0],
            [1,0,0,0],
            [0,0,0,0],
        ],
    ]
};

pub fn from_kind(k: Kind) -> &'static Tetromino {
    match k {
        Kind::Stick => &STICK,
        Kind::Seven => &SEVEN,
        Kind::Hook => &HOOK,
        Kind::Square => &SQUARE,
        Kind::Snake => &SNAKE,
        Kind::Pyramid => &PYRAMID,
        Kind::Zig => &ZIG,
    }
}
