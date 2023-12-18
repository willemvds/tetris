pub type Shape = [[u8; 4]; 4];

#[derive(Debug)]
pub struct Tetromino {
    pub height: u8,
    pub width: u8,
    pub shape: Shape,
}

#[derive(Debug)]
pub struct TwoPiece {
    pub tetros: [Tetromino; 2],
}

#[derive(Debug)]
pub struct FourPiece {
    pub tetros: [Tetromino; 4],
}

#[derive(Debug)]
pub enum Piece {
    Straight(StraightPiece),
    Square(SquarePiece),
    L(LPiece),
    Skew(TwoPiece),
    T(FourPiece),
    J(FourPiece),
    Z(TwoPiece),
}

impl Piece {
    pub fn rotations(&self) -> usize {
        match self {
            Piece::Straight(_) => 2, // I
            Piece::Square(_) => 1,   // O
            Piece::L(_) => 4,
            Piece::Skew(_) => 2, // S
            Piece::T(_) => 4,
            Piece::J(_) => 4,
            Piece::Z(_) => 2,
        }
    }

    pub fn rotation_after(&self, current: usize) -> usize {
        let rotation_after = current + 1;

        if rotation_after >= self.rotations() {
            return 0;
        }

        rotation_after
    }

    pub fn tetro(&self, rot: usize) -> &Tetromino {
        match self {
            Piece::Straight(sp) => &(sp.tetros[rot]),
            Piece::Square(sp) => &(sp.tetros[rot]),
            Piece::L(lp) => &(lp.tetros[rot]),
            Piece::Skew(sp) => &(sp.tetros[rot]),
            Piece::T(tp) => &(tp.tetros[rot]),
            Piece::J(jp) => &(jp.tetros[rot]),
            Piece::Z(zp) => &(zp.tetros[rot]),
            // _ => panic!("piece_tetro BAD - this is getting hairy"),
        }
    }
}

#[derive(Debug)]
pub struct StraightPiece {
    pub tetros: [Tetromino; 2],
}

#[rustfmt::skip]
pub const STRAIGHT_PIECE: Piece = Piece::Straight(StraightPiece{
    tetros: [
        Tetromino{
            height: 4,
            width: 1,
            shape: [
                [1, 0, 0, 0],
                [1, 0, 0, 0],
                [1, 0, 0, 0],
                [1, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 1,
            width: 4,
            shape: [
                [1, 1, 1, 1],
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
    ]
});

#[derive(Debug)]
pub struct SquarePiece {
    pub tetros: [Tetromino; 1],
}

#[rustfmt::skip]
pub const SQUARE_PIECE: Piece = Piece::Square(SquarePiece{
    tetros: [
        Tetromino{
            height: 2,
            width: 2,
            shape: [
                [1, 1, 0, 0],
                [1, 1, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
    ]
});

#[derive(Debug)]
pub struct LPiece {
    pub tetros: [Tetromino; 4],
}

#[rustfmt::skip]
pub const L_PIECE: Piece = Piece::L(LPiece{
    tetros: [
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [1, 0, 0, 0],
                [1, 0, 0, 0],
                [1, 1, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [0, 0, 1, 0],
                [1, 1, 1, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [1, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [1, 1, 1, 0],
                [1, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },

    ]
});

#[rustfmt::skip]
pub const SKEW_PIECE: Piece = Piece::Skew(TwoPiece{
    tetros: [
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [0,1,1,0],
                [1,1,0,0],
                [0,0,0,0],
                [0,0,0,0]
            ],
        },
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [1,0,0,0],
                [1,1,0,0],
                [0,1,0,0],
                [0,0,0,0]
            ],
        },
    ]
});

#[rustfmt::skip]
pub const Z_PIECE: Piece = Piece::Z(TwoPiece{
    tetros: [
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [1, 1, 0, 0],
                [0, 1, 1, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [0, 1, 0, 0],
                [1, 1, 0, 0],
                [1, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
    ]
});

#[rustfmt::skip]
pub const T_PIECE: Piece = Piece::T(FourPiece{
    tetros: [
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [1, 1, 1, 0],
                [0, 1, 0, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [0, 1, 0, 0],
                [1, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 2,
            width: 3,
            shape: [
                [0, 1, 0, 0],
                [1, 1, 1, 0],
                [0, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },
        Tetromino{
            height: 3,
            width: 2,
            shape: [
                [1, 0, 0, 0],
                [1, 1, 0, 0],
                [1, 0, 0, 0],
                [0, 0, 0, 0]
            ],
        },

    ]
});
