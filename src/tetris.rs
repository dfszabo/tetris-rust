use crate::tetris::MoveAction::{DOWN, LEFT, RIGHT, ROTATE};
use core::cmp;
use std::iter::Iterator;

pub(crate) const SCREEN_WIDTH: u32 = 400;
pub(crate) const SCREEN_HEIGHT: u32 = 800;
pub(crate) const RECT_DIM: u32 = 40;
pub(crate) const WIDTH: u8 = (SCREEN_WIDTH / RECT_DIM) as u8;
pub(crate) const HEIGHT: u8 = (SCREEN_HEIGHT / RECT_DIM) as u8;

#[warn(non_upper_case_globals)]
pub static tetrominos: [[u8; 16]; 7] = [
    [0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0], // S
    [0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0], // T
    [0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0], // Z
    [0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0], // L
    [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0], // O
    [0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0], // J
    [0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0], // I
];

#[derive(PartialEq)]
pub enum MoveAction {
    LEFT,
    RIGHT,
    DOWN,
    ROTATE,
    NONE,
    QUIT,
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub kind: usize,
    pub rotation: u8,
    pub x: i8,
    pub y: i8,
}

pub struct Game {
    pub board: Vec<Vec<u8>>,
    pub score: u32,
    pub curr_piece: Piece,
    pub next_piece_kind: usize,
    pub target_piece: Piece,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: vec![vec![0u8; 10]; 20],
            score: 0,
            curr_piece: Piece {
                kind: 0,
                rotation: 0,
                x: 0,
                y: 5,
            },
            next_piece_kind: 0,
            target_piece: Piece {
                kind: 100,
                rotation: 100,
                x: 100,
                y: 100,
            },
        }
    }

    pub fn rotate(x: u8, y: u8, rot: u8) -> usize {
        match rot % 4 {
            0 => (x * 4u8 + y) as usize,
            1 => (12u8 + x - (y * 4u8)) as usize,
            2 => (15u8 - (x * 4u8) - y) as usize,
            3 => (3u8 - x + (y * 4u8)) as usize,
            _ => panic!("Impossible"),
        }
    }

    pub fn does_piece_fit(&self, tetr_idx: usize, rotation: u8, x: i8, y: i8) -> bool {
        for tetr_x in 0..4u8 {
            for tetr_y in 0..4u8 {
                let rotated_index = Game::rotate(tetr_x, tetr_y, rotation);

                let abs_x = x + tetr_x as i8;
                let abs_y = y + tetr_y as i8;

                // if the absolute indexes are valid ones for the board
                if abs_x >= 0 && abs_x < HEIGHT as i8 && abs_y >= 0 && abs_y < WIDTH as i8 {
                    if tetrominos[tetr_idx][rotated_index] != 0
                        && self.board[abs_x as usize][abs_y as usize] != 0
                    {
                        return false;
                    }
                }
                // else if some absolute index is out of bounds and also the tetromino
                // field is not 0 that mean the piece would would go outside the board
                // so return false
                else if tetrominos[tetr_idx][rotated_index] != 0 {
                    return false;
                }
            }
        }

        // if control reached this point that means it can fit
        true
    }

    // Search for filled lines and removes them. Also updating the score.
    pub fn find_and_remove_solved_lines(&mut self) {
        let mut found_lines = 0;
        let mut line_start_idx = 0;

        // Search for filled lines on the lines where the current piece is.
        // Starting the search from the bottom.
        for row_idx in (0..4).rev() {
            let abs_row_idx = (row_idx + self.curr_piece.x) as usize;

            // if its outside the board then continue with nex iteration
            if abs_row_idx >= HEIGHT as usize {
                continue;
            }

            // check if the line is filled with blocks
            let mut found = true;
            for idx in 0..WIDTH as usize {
                if self.board[abs_row_idx][idx] == 0 {
                    found = false;
                    break;
                }
            }

            // if a filled line was found
            if found {
                // if its the first one we have found, then save its index
                if found_lines == 0 {
                    line_start_idx = abs_row_idx;
                }

                // found another line so increment the counter
                found_lines += 1;
            }
        }

        // if there were not any filled lines then exit
        if found_lines == 0 {
            return;
        }

        // removing the filled lines
        for row in (found_lines as usize..line_start_idx + 1).rev() {
            for col in 0..WIDTH as usize {
                self.board[row][col] = self.board[row - found_lines as usize][col];
            }
        }

        // updating the score
        self.score += found_lines * 10;
    }

    // Updating the board by setting the blocks where the current piece is to @value
    pub fn update_current_piece(&mut self, value: u8) {
        for row in 0..4 {
            for col in 0..4 {
                let rotated_idx = Game::rotate(row, col, self.curr_piece.rotation);

                if tetrominos[self.curr_piece.kind][rotated_idx] == 1 {
                    let abs_row = row as i8 + self.curr_piece.x;
                    let abs_col = col as i8 + self.curr_piece.y;

                    assert!(abs_col < WIDTH as i8 && abs_col >= 0);
                    assert!(abs_row < HEIGHT as i8 && abs_row >= 0);

                    self.board[abs_row as usize][abs_col as usize] = value;
                }
            }
        }
    }

    // adding the current piece blocks to the board
    pub fn add_current_piece(&mut self) {
        self.update_current_piece(self.curr_piece.kind as u8 + 1);
    }

    // removing the current piece blocks from the board
    pub fn remove_current_piece(&mut self) {
        self.update_current_piece(0);
    }

    // Moving down the current piece on the board. If can't then return false, otherwise
    // true.
    pub fn move_piece_down(&mut self) -> bool {
        if self.does_piece_fit(
            self.curr_piece.kind,
            self.curr_piece.rotation,
            self.curr_piece.x + 1,
            self.curr_piece.y,
        ) {
            self.curr_piece.x += 1;
            true
        } else {
            false
        }
    }

    pub fn move_piece_left(&mut self) -> bool {
        if self.does_piece_fit(
            self.curr_piece.kind,
            self.curr_piece.rotation,
            self.curr_piece.x,
            self.curr_piece.y - 1,
        ) {
            self.curr_piece.y -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_piece_right(&mut self) -> bool {
        if self.does_piece_fit(
            self.curr_piece.kind,
            self.curr_piece.rotation,
            self.curr_piece.x,
            self.curr_piece.y + 1,
        ) {
            self.curr_piece.y += 1;
            true
        } else {
            false
        }
    }

    pub fn rotate_piece(&mut self) -> bool {
        if self.does_piece_fit(
            self.curr_piece.kind,
            (self.curr_piece.rotation + 1) % 4,
            self.curr_piece.x,
            self.curr_piece.y,
        ) {
            self.curr_piece.rotation = (self.curr_piece.rotation + 1) % 4;
            true
        } else {
            false
        }
    }

    // calculates how uneven the neighboring columns height
    pub fn blocks_bumpiness(&self) -> (u64, u64) {
        let mut bumpiness_factor = 0u64;
        let mut heights = vec![0u8; WIDTH as usize];

        for col in 0..WIDTH as usize - 1 {
            for row in 0..HEIGHT as usize {
                if self.board[row][col] != 0 {
                    heights[col] = HEIGHT - row as u8;
                    break;
                }
            }
        }

        for idx in 0..WIDTH as usize - 1 {
            bumpiness_factor += (heights[idx + 1] as i8 - heights[idx] as i8).abs() as u64;
        }

        (bumpiness_factor, *heights.iter().max().unwrap() as u64)
    }

    pub fn blocks_hole_factor(&self) -> u64 {
        let mut hole_factor = 0;

        for col in 0..WIDTH as usize {
            let mut column_holes = 0;
            let mut row = 0;

            // find the first non empty block
            while row < HEIGHT as usize && self.board[row][col] == 0 {
                row += 1;
            }

            // if no such block found then continue with the next column
            if row == HEIGHT as usize {
                continue;
            }

            // count the holes
            // TODO: this is a simplistic way for now, considering every non filled block
            // as a hole
            for row in row..HEIGHT as usize {
                if self.board[row][col] == 0 {
                    column_holes += row as u64;
                }
            }

            hole_factor += column_holes;
        }

        hole_factor
    }

    pub fn block_line_continuity(&self) -> u64 {
        let mut continuity_factor = 0;

        for row in (0..HEIGHT as usize).rev() {
            let mut curr_line_continuity = 0;
            let mut col = 0;

            while col < WIDTH as usize && self.board[row][col] != 0 {
                col += 1;
                curr_line_continuity += 1;
            }
            continuity_factor += curr_line_continuity * curr_line_continuity * (row as u64);
        }

        continuity_factor
    }

    pub fn block_line_filledness(&self) -> u64 {
        let mut overall_filledness_factor = 0;

        for row in (0..HEIGHT as usize).rev() {
            let mut line_filledness = 0;

            for col in 0..WIDTH as usize {
                if self.board[row][col] != 0 {
                    line_filledness += 1;
                }
            }

            overall_filledness_factor += line_filledness * line_filledness * row;
        }

        overall_filledness_factor as u64
    }

    fn fitness(&mut self, fitness_params: [u64; 6]) -> u64 {
        let mut filled_lines = 0;

        for row in 0..HEIGHT as usize {
            let mut filled = true;
            for col in 0..WIDTH as usize {
                if self.board[row][col] == 0 {
                    filled = false;
                    break;
                }
            }
            if filled {
                filled_lines += 1;
            }
        }

        filled_lines *= 10;
        filled_lines = filled_lines * filled_lines;

        let hole_fact = self.blocks_hole_factor();
        let hole_fact = hole_fact * hole_fact;
        let (bumpiness, max_height) = self.blocks_bumpiness();
        let continuity = self.block_line_continuity();
        let filledness = self.block_line_filledness();

        let fitness = 1_000_000_000_000_000_000u64;
        let fitness = cmp::max(
            0,
            fitness - (fitness_params[0] as f64 * 10f64 * hole_fact as f64) as u64,
        );
        let fitness = cmp::max(0, fitness - fitness_params[1] * 2500 * bumpiness);
        let fitness = cmp::max(
            0,
            fitness - (fitness_params[2] as f64 * 20f64 * max_height as f64) as u64,
        );
        let fitness = cmp::max(
            0,
            fitness + (fitness_params[3] as f64 / 50f64 * continuity as f64) as u64,
        );
        let fitness = cmp::max(
            0,
            fitness + (fitness_params[4] as f64 * 50f64 * filledness as f64) as u64,
        );
        let fitness = cmp::max(
            0,
            fitness + (fitness_params[5] as f64 * 300f64 * filled_lines as f64) as u64,
        );

        // println!(
        //     "hole_fact[{}], bumpiness[{}], continuity[{}], filled_lines[{}] => fintess[{}]",
        //     hole_fact, bumpiness, continuity, filled_lines, fitness
        // );

        fitness
    }

    // minor run time optimization
    // some pieces symmetric, therefore pointless to test some of its rotations
    // for example: cube (name: O) rotation is pointless
    pub fn max_rotation(kind: usize) -> u8 {
        match kind {
            0 | 2 | 6 => 2, // S, Z, I
            4 => 1,         // O
            _ => 4,
        }
    }

    pub fn bot(&mut self, fitness_params: [u64; 6]) -> MoveAction {
        let kind = self.curr_piece.kind;
        let mut best_fitness = 0u64;
        let mut best_piece = self.curr_piece;
        let original_piece = self.curr_piece; // save the original values

        if self.target_piece.kind == 100 {
            for rotation in 0..Game::max_rotation(kind) {
                for col in 0..WIDTH as i8 + 5 {
                    let col = col - 2;
                    let mut curr_piece_x = self.curr_piece.x;

                    // if the piece cannot even placed then continue with next iteration
                    if !self.does_piece_fit(kind, rotation, curr_piece_x, col) {
                        continue;
                    }

                    // pushing down the piece until it would stuck into its final place
                    while self.does_piece_fit(kind, rotation, curr_piece_x + 1, col) {
                        curr_piece_x += 1;
                    }

                    // evaluate the resulting game board goodness

                    // Step 1: adding the piece to the board
                    self.curr_piece = Piece {
                        kind,
                        rotation,
                        x: curr_piece_x,
                        y: col,
                    };
                    self.add_current_piece();

                    let kind_2 = self.next_piece_kind;
                    self.curr_piece = Piece {
                        kind: self.next_piece_kind,
                        rotation: 0,
                        x: 0,
                        y: 5,
                    };
                    let original_piece_2 = self.curr_piece;

                    for rotation_2 in 0..Game::max_rotation(kind_2) {
                        for col_2 in 0..WIDTH as i8 + 5 {
                            let col_2 = col_2 - 2;
                            let mut curr_piece_x_2 = self.curr_piece.x;

                            // if the piece cannot even placed then continue with next iteration
                            if !self.does_piece_fit(kind_2, rotation_2, curr_piece_x_2, col_2) {
                                continue;
                            }

                            // pushing down the piece until it would stuck into its final place
                            while self.does_piece_fit(kind_2, rotation_2, curr_piece_x_2 + 1, col_2) {
                                curr_piece_x_2 += 1;
                            }

                            // evaluate the resulting game board goodness

                            // Step 1: adding the piece to the board
                            self.curr_piece = Piece {
                                kind: kind_2,
                                rotation: rotation_2,
                                x: curr_piece_x_2,
                                y: col_2,
                            };
                            self.add_current_piece();

                            // Step 2: calculate the fitness
                            let fitness = self.fitness(fitness_params);

                            // Step 3: remove the piece and restore position
                            self.remove_current_piece();
                            self.curr_piece = original_piece_2;

                            // check whether this move is better then the current best
                            if fitness > best_fitness {
                                best_piece = Piece {
                                    kind,
                                    rotation,
                                    x: curr_piece_x,
                                    y: col,
                                };
                                best_fitness = fitness;
                            }
                        }
                    }

                    // adding back the outer loops piece to be able to remove it
                    self.curr_piece = Piece {
                        kind,
                        rotation,
                        x: curr_piece_x,
                        y: col,
                    };

                    // Step 3: remove the piece and restore position
                    self.remove_current_piece();
                    self.curr_piece = original_piece;
                }
            }

            self.target_piece = best_piece;
        }
        // decide the next move based on the best_piece coordinate and rotation
        // on default just push it down
        let mut bot_action = DOWN;

        if original_piece.rotation != self.target_piece.rotation {
            bot_action = ROTATE;
        }
        // if the @y coordinates are different then
        else if original_piece.y != self.target_piece.y {
            // move RIGHT if the current piece is left to the best move
            if original_piece.y < self.target_piece.y {
                bot_action = RIGHT;
            }
            // else the current piece is on the right to the best move so move LEFT
            else {
                bot_action = LEFT;
            }
        }

        bot_action
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rotate() {
        let idx = Game::rotate(0u8, 0u8, 0u8);
        assert_eq!(idx, 0usize);

        let idx = Game::rotate(3u8, 1u8, 1u8);
        assert_eq!(idx, 11usize);

        let idx = Game::rotate(3u8, 1u8, 2u8);
        assert_eq!(idx, 2usize);

        let idx = Game::rotate(3u8, 1u8, 3u8);
        assert_eq!(idx, 4usize);
    }

    #[test]
    fn test_does_piece_fit() {
        let mut game = Game::new();
        game.board[(HEIGHT - 1) as usize][0] = 1;

        assert_eq!(game.does_piece_fit(3, 0, -2, 0), false);
        assert_eq!(game.does_piece_fit(3, 0, 0, -2), false);
        assert_eq!(game.does_piece_fit(3, 0, HEIGHT as i8 - 3, 0), false);
    }

    #[test]
    fn test_find_and_remove_solved_lines() {
        let mut game = Game::new();
        game.board[(HEIGHT - 1) as usize][0] = 1;

        game.find_and_remove_solved_lines();
        assert_eq!(game.score, 0);

        for idx in 0..WIDTH as usize {
            game.board[(HEIGHT - 1) as usize][idx] = 1;
        }

        game.curr_piece.x = HEIGHT as i8 - 3;

        game.find_and_remove_solved_lines();
        assert_eq!(game.board[(HEIGHT - 1) as usize], vec![0u8; WIDTH as usize]);
        assert_eq!(game.score, 10);

        for idx in 0..WIDTH as usize {
            game.board[(HEIGHT - 1) as usize][idx] = 1;
            game.board[(HEIGHT - 2) as usize][idx] = 1;
        }

        game.find_and_remove_solved_lines();
        assert_eq!(game.board[(HEIGHT - 1) as usize], vec![0u8; WIDTH as usize]);
        assert_eq!(game.board[(HEIGHT - 2) as usize], vec![0u8; WIDTH as usize]);
        assert_eq!(game.score, 30);
    }

    #[test]
    fn test_add_current_piece() {
        let mut game = Game::new();

        game.add_current_piece();
        assert_eq!(game.board[0][6], 1);

        game.remove_current_piece();
        assert_eq!(game.board[0][6], 0);
    }
}
