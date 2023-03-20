use rand::{Rng, distributions::WeightedIndex, prelude::Distribution};

// Board constants
const BOARD_DIMENSION: usize = 4;
const EMPTY_SLOT: u32 = 0;

// 2-tiles should outnumber 4-tiles 4:1
const NEW_TILE_CHOICES: [u32; 2] = [2, 4];
const NEW_TILE_WEIGHTS: [u8; 2] = [4, 1];

pub struct Game {
    board: [[u32; BOARD_DIMENSION]; BOARD_DIMENSION],
}

impl Game {
    /// Generates a new game board in a ready-to-play state.
    /// This means that the board will be empty except for two starting tiles.
    ///
    /// The two tiles will either both be 2's or one 2 and one 4, always in random positions.
    pub fn new() -> Game {
        let game = Game {
            board: [[EMPTY_SLOT; BOARD_DIMENSION]; BOARD_DIMENSION],
        };

        // First tile coordinates
        // let row = rng.gen_range(0..BOARD_DIMENSION);
        // let col = rng.gen_range(0..BOARD_DIMENSION);

        game
    }

    // Generates a new tile - either 2 or 4 according to pre-defined weighted probability
    pub fn generate_tile() -> u32 {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(&NEW_TILE_WEIGHTS).unwrap();

        let tile = NEW_TILE_CHOICES[dist.sample(&mut rng)];

        tile
    }

    /// Prints a text representation of the game board to stdout.
    pub fn print_board(&self) {
        for row in self.board {
            println!("{:?}", row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tile_rng() {
        let num_trials = 100;

        for i in 0..num_trials {
            println!("Test iteration: {i}");

            let mut two_count = 0;
            let mut four_count = 0;

            const TEST_SAMPLE_SIZE: u32 = 10000;

            for _ in 0..TEST_SAMPLE_SIZE {
                let tile = Game::generate_tile();

                if tile == 2 {
                    two_count += 1;
                } else if tile == 4 {
                    four_count += 1;
                }
            }

            let two_dist = two_count as f32 / TEST_SAMPLE_SIZE as f32;
            let four_dist = four_count as f32 / TEST_SAMPLE_SIZE as f32;

            let expected_ratio = NEW_TILE_WEIGHTS[0] as f32;
            let actual_ratio = two_dist / four_dist;

            // Run `cargo test -- --nocapture` to show stdout
            println!("Expected 2:4 ratio: {expected_ratio}:1");
            println!("Actual 2:4 ratio: {actual_ratio}:1");
            
            let error_margin = expected_ratio * 0.20;

            let expected_ratio_range = (expected_ratio - error_margin)..(expected_ratio + error_margin);

            assert!(expected_ratio_range.contains(&actual_ratio));
        }
    }
}

