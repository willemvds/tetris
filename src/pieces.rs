use crate::types::*;

// mod pieces {
    #[rustfmt::skip]
    pub const J_PIECE: Piece = Piece::J(FourPiece{
        // kind: PieceKind::Skew,
        tetros: [
            Tetromino{
                height: 3,
                width: 2,
                shape: [
                    [0, 1, 0, 0],
                    [0, 1, 0, 0],
                    [1, 1, 0, 0],
                    [0, 0, 0, 0]
                ],
            },
            Tetromino{
                height: 2,
                width: 3,
                shape: [
                    [1, 1, 1, 0],
                    [0, 0, 1, 0],
                    [0, 0, 0, 0],
                    [0, 0, 0, 0]
                ],
            },
            Tetromino{
                height: 3,
                width: 2,
                shape: [
                    [1, 1, 0, 0],
                    [1, 0, 0, 0],
                    [1, 0, 0, 0],
                    [0, 0, 0, 0]
                ],
            },
            Tetromino{
                height: 2,
                width: 3,
                shape: [
                    [1, 0, 0, 0],
                    [1, 1, 1, 0],
                    [0, 0, 0, 0],
                    [0, 0, 0, 0]
                ],
            },

        ]
    });
// }