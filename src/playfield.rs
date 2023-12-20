use crate::tetrominos;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Location {
    Empty,
    Filled(tetrominos::Kind),
}

pub type Shape = [[u8; 4]; 4];
type Matrix = Vec<Vec<Location>>;

// #[derive(Debug, Clone)]
pub struct PlayField {
    pub cols: usize,
    pub rows: usize,
    pub matrix: Matrix,
}

impl PlayField {
    pub fn new(rows: usize, cols: usize) -> PlayField {
        PlayField {
            cols: cols + 4, // We have 1 extra column to the left and 3 extra columns to the right
            rows: rows,
            matrix: vec![vec![Location::Empty; cols]; rows],
        }
    }

    pub fn collission_matrix(&self, x: usize, y: usize, shape: &Shape) -> bool {
        let mut total: u8 = 0;

        for row in 0..4 {
            for col in 0..4 {
                if row + y >= self.matrix.len() {
                    continue;
                }

                if col + x >= self.matrix[row + y].len() {
                    continue;
                }

                total += shape[row][col]
                    & match self.matrix[y + row][x + col] {
                        Location::Empty => 0,
                        Location::Filled(_) => 1,
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
