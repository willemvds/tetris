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

const ROWS_PADDING: usize = 6; // 2 bottom, 4 top
const COLS_PADDING: usize = 6; // 3 left, 3 right

pub const ERR_PLAYFIELD_MIN_SIZE: &str = "smallest possible playfield size is 4 rows x 4 cols";

impl PlayField {
    // The rows and cols specified here is for the size of the well (the inner part where the
    // tetrominos(blocks) are placed). The total size of the playfield matrix will be larger
    // to add required padding and space required for gameplay mechanics.
    //
    // Details of the padding are as follows:
    // - TOP: 4 empty rows
    // - LEFT: 3 edge columns
    // - RIGHT: 3 edge columns
    // - BOTTOM: 2 edge rows
    pub fn new(rows: usize, cols: usize) -> Result<PlayField, &'static str> {
        if rows < 4 || cols < 4 {
            return Err(ERR_PLAYFIELD_MIN_SIZE);
        }

        let matrix_rows = rows + ROWS_PADDING;
        let matrix_cols = cols + COLS_PADDING;

        let mut pf = PlayField {
            cols,
            rows,
            matrix: vec![vec![Location::Edge; matrix_cols]; matrix_rows],
        };

        // clear the top 4 rows
        for row in 0..4 {
            for col in 0..cols + COLS_PADDING {
                pf.matrix[row][col] = Location::Empty;
            }
        }

        // cut out the well
        let row_offset = 4;
        let col_offset = COLS_PADDING / 2;
        for row in row_offset..rows + row_offset {
            for col in col_offset..cols + col_offset {
                pf.matrix[row][col] = Location::Empty;
            }
        }

        Ok(pf)
    }

    pub fn well_x(&self) -> usize {
        return COLS_PADDING / 2;
    }

    fn well_y(&self) -> usize {
        return 4;
    }

    pub fn has_collission(&self, shape_y: usize, shape_x: usize, shape: &Shape) -> bool {
        let mut total: u8 = 0;

        let mut shape_rows = 4;
        let shape_height = 4;
        if shape_y + shape_height >= self.rows + ROWS_PADDING {
            shape_rows = self.rows + ROWS_PADDING - shape_y;
        }

        let mut shape_cols = 4;
        let shape_width = 4;
        if shape_x + shape_width >= self.cols + COLS_PADDING {
            shape_cols = self.cols + COLS_PADDING - shape_x;
        }

        for row in 0..shape_rows {
            for col in 0..shape_cols {
                total += shape[row][col]
                    & match self.matrix[shape_y + row][shape_x + col] {
                        Location::Empty => 0,
                        _default => 1,
                    }
            }
        }

        total > 0
    }

    pub fn clear_full_rows(&mut self) -> usize {
        let row_offset = self.well_y();
        let col_offset = self.well_x();

        let mut lines_cleared = 0;

        for row in row_offset..self.rows + row_offset {
            let mut col_count = 0;
            for col in col_offset..self.cols + col_offset {
                if self.matrix[row][col] != Location::Empty {
                    col_count += 1;
                }
            }
            if col_count == self.cols {
                for clear_col in col_offset..self.cols + col_offset {
                    self.matrix[row][clear_col] = Location::Empty;
                }
                lines_cleared += 1;
            }
        }

        lines_cleared
    }

    pub fn collapse(&mut self) {
        let row_offset = self.well_y();
        let col_offset = self.well_x();

        for row in (row_offset..self.rows + row_offset).rev() {
            let mut has_block = false;
            for col in col_offset..self.cols + col_offset {
                if self.matrix[row][col] != Location::Empty {
                    has_block = true;
                    break;
                }
            }

            if !has_block {
                for ir in (1..=row).rev() {
                    for c in col_offset..self.cols + col_offset {
                        self.matrix[ir][c] = self.matrix[ir - 1][c];
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_collission_empty() {
        let pf = PlayField::new(10, 10);
        let shape: Shape = [[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]];

        assert_eq!(false, pf.has_collission(5, 5, &shape));
    }

    #[test]
    fn test_has_collission_hit() {
        let mut pf = PlayField::new(10, 10);
        pf.matrix[0][0] = Location::Filled(tetrominos::Kind::Hook);
        let shape: Shape = [[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]];

        assert_eq!(true, pf.has_collission(0, 0, &shape));
    }
}
