use crate::tetrominos;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Location {
    Empty,
    Edge,
    Filled(tetrominos::Kind),
}

pub type Shape = [[u8; 4]; 4];
type Matrix = Vec<Vec<Location>>;

#[derive(Debug)]
pub struct PlayField {
    pub cols: usize,
    pub rows: usize,
    pub matrix: Matrix,
}

impl PlayField {
    pub fn new(rows: usize, cols: usize) -> PlayField {
        let mut pf = PlayField {
            cols: cols + 4, // We have 1 extra column to the left and 3 extra columns to the right
            rows: rows + 1, // We have 1 extra row at the bottom
            matrix: vec![vec![Location::Edge; cols + 4]; rows + 1],
        };

        for row in 0..rows - 1 {
            for col in 1..cols + 1 {
                pf.matrix[row][col] = Location::Empty;
            }
        }

        pf
    }

    pub fn has_collission(&self, shape_y: usize, shape_x: usize, shape: &Shape) -> bool {
        let mut total: u8 = 0;

        for row in 0..4 {
            for col in 0..4 {
                if row + shape_y >= self.matrix.len() {
                    continue;
                }

                if col + shape_x >= self.matrix[row + shape_y].len() {
                    continue;
                }

                total += shape[row][col]
                    & match self.matrix[shape_y + row][shape_x + col] {
                        Location::Empty => 0,
                        _default => 1,
                    }
            }
        }

        total > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collission_matrix_empty() {
        let pf = PlayField::new(10, 10);
        let shape: Shape = [[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]];

        assert_eq!(false, pf.collission_matrix(0, 0, &shape));
    }

    #[test]
    fn test_collission_matrix_hit() {
        let mut pf = PlayField::new(10, 10);
        pf.matrix[0][0] = Location::Filled(tetrominos::Kind::Hook);
        let shape: Shape = [[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]];

        assert_eq!(true, pf.collission_matrix(0, 0, &shape));
    }
}
