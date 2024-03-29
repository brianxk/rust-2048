use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom};
use std::collections::LinkedList;
use hex_color::HexColor;

mod counted_channel;

pub const BOARD_DIMENSION: usize = 4;
const NUM_TILES: usize = BOARD_DIMENSION * BOARD_DIMENSION;

#[derive(PartialEq, Clone)]
pub struct Tile {
    pub value: u32,
    pub id: usize,
    pub background_color: String,
    pub text_color: String,
    pub row: usize,
    pub col: usize,
    pub merged: Option<Box<Tile>>,
}

impl Tile {
    fn new(value: u32, id: usize, background_color: String, text_color: String, row: usize, col: usize) -> Tile {
        Tile {
            value,
            id,
            background_color,
            text_color,
            row,
            col,
            merged: None,
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "value: {}\nid: {}\nrow: {}\n col:{}",
               self.value,
               self.id,
               self.row,
               self.col)
    }
}

/// Struct that holds the choices for new tiles and the probability with which they will appear.
#[derive(PartialEq, Clone)]
struct NewTileParams {
    tile_choices: [u32; 2],
    tile_weights: [u8; 2],
}

impl NewTileParams {
    /// Represents the index position for accessing parameters related to 2-tiles in the
    /// `tile_choices` and `tile_weights` arrays.
    const TWO: usize = 0;
    
    /// Represents the index position for accessing parameters related to 4-tiles in the
    /// `tile_choices` and `tile_weights` arrays.
    const FOUR: usize = 1;

    /// Initializes the default settings for new tile creation such that 2-tiles appear more
    /// frequently than 4-tiles at a 4:1 ratio.
    fn new() -> Self {
        NewTileParams {
            tile_choices: [2, 4],
            tile_weights: [4, 1],
        }
    }
}

#[derive(PartialEq)]
pub struct Colors {
    pub background_dark: &'static str,
    pub background_light: &'static str,
    pub text_dark: &'static str,
    pub text_light: &'static str,
    pub button: &'static str,
    pub button_hover: &'static str,
    pub board: &'static str,
    pub cell: &'static str,
    pub opacity: &'static str,
    // Tile's color will be determined within the `Game` struct based on the Tile's individual value.
}

impl Colors {
    pub const fn new() -> Self {
        Colors {
            background_dark: "#072931",
            background_light: "#072931",
            text_dark: "#072931",
            text_light: "#f2ba0d",
            button: "#92cdb9",
            button_hover: "#b4ddcf",
            board: "#022244",
            cell: "#92cdb9",
            // opacity is meant to be appended to any of the above colors, so is not prepended with #.
            opacity: "99", // Equivalent to opacity: 0.6;
        }
    }
}

pub struct InvalidMove;

pub enum InputResult<'a> {
    Ok(usize, Vec<&'a Tile>, bool),
    Err(InvalidMove),
}

#[derive(PartialEq, Clone)]
pub struct Game {
    pub board: [[Option<Tile>; BOARD_DIMENSION]; BOARD_DIMENSION],
    new_tile_params: NewTileParams,
    free_slots: Vec<(usize, usize)>,
    pub score: u32,
    id_list: LinkedList<usize>,
    game_won: bool, // Will be initialized to false, but the frontend will have 
                        // the freedom to set this to `true` depending on when a
                        // certain tile value is reached. This means that 2048 does
                        // not strictly need to be the winning tile.
}

impl Game {
    /// Generates a new game board in a ready-to-play state.
    ///
    /// This means that the board will be empty save for two starting tiles.
    ///
    /// The two tiles will either both be 2's or one 2 and one 4, always in random positions.
    const WINNING_TILE: u32 = 2048;

    pub fn new() -> Game {
        const EMPTY_TILE: Option<Tile> = None;
        const EMPTY_ROW: [Option<Tile>; BOARD_DIMENSION] = [EMPTY_TILE; BOARD_DIMENSION];
        
        // Tile IDs will be recycled, but we are making the number of available IDs 1 greater than
        // the maximum number of tiles. This is because a new tile should not recycle an ID from a
        // tile that was just merged on the current turn. The edge case here is the entire board is
        // occupied with 16 tiles but a player move is still possible; in this case the new tile
        // created after this move will need a 17th ID to use.
        let tile_ids: [usize; NUM_TILES + 1] = std::array::from_fn(|i| i as usize);

        let mut game = Game {
            board: [EMPTY_ROW; BOARD_DIMENSION],
            new_tile_params: NewTileParams::new(),
            free_slots: Vec::with_capacity(BOARD_DIMENSION * BOARD_DIMENSION),
            score: 0,
            id_list: LinkedList::from(tile_ids),
            game_won: false,
        };

        // If first tile is 4, second tile must be 2.
        // If first tile is 2, second tile may either be 2 or 4.
        let first_tile_value = game.generate_tile_value();
        let second_tile_value;
        
        if first_tile_value == game.new_tile_params.tile_choices[NewTileParams::FOUR] {
            second_tile_value = game.new_tile_params.tile_choices[NewTileParams::TWO];
        } else {
            second_tile_value = game.generate_tile_value();
        }

        // let first_tile_value = 16384;
        // let second_tile_value = 131072;
        // let first_tile_value = 2048;
        // let second_tile_value = 1024;

        let first_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");
        let first_tile_id = game.get_id().unwrap();
        let (background_color, text_color) = game.get_tile_colors(first_tile_value);

        let first_tile = Tile::new(first_tile_value,
                                   first_tile_id, 
                                   background_color,
                                   text_color,
                                   first_tile_pos.0,
                                   first_tile_pos.1);

        game.board[first_tile_pos.0][first_tile_pos.1] = Some(first_tile);
        
        let second_tile_pos = game.get_random_free_slot().expect("New game board, should not panic.");
        let second_tile_id = game.get_id().unwrap();
        let (background_color, text_color) = game.get_tile_colors(second_tile_value);

        let second_tile = Tile::new(second_tile_value,
                                    second_tile_id,
                                    background_color,
                                    text_color,
                                    second_tile_pos.0,
                                    second_tile_pos.1);

        game.board[second_tile_pos.0][second_tile_pos.1] = Some(second_tile);

        game
    }

    /// Returns the next available ID. Will return None if all IDs are used.
    fn get_id(&mut self) -> Option<usize> {
        self.id_list.pop_front()
    }

    /// Receives a vector of IDs to recycle
    fn recycle_ids(&mut self, ids: Vec<usize>) {
        for id in ids {
            self.id_list.push_back(id);
        }
    }

    /// Generates a new tile - either 2 or 4 according to the weights defined in
    /// `self.new_tile_params`
    fn generate_tile_value(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(self.new_tile_params.tile_weights).unwrap();

        let tile = self.new_tile_params.tile_choices[dist.sample(&mut rng)];

        tile
    }

    /// Updates the list of free slots.
    fn update_free_slots(&mut self) {
        self.free_slots.clear();

        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let None = self.board[row][col] {
                    self.free_slots.push((row, col));
                }
            }
        }
    }

    /// Returns a vec of all current tiles.
    pub fn get_tiles(&self) -> Vec<&Tile> {
        let mut tiles = Vec::new();

        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let Some(tile) = &self.board[row][col] {
                    tiles.push(tile);
                }
            }
        }

        tiles
    }

    /// Returns the coordinates of a free board slot at random. 
    /// Will return `None` if no free slots exist, indicating the game is over.
    fn get_random_free_slot(&mut self) -> Option<(usize, usize)> {
        self.update_free_slots();

        let mut rng = rand::thread_rng();

        self.free_slots.choose(&mut rng).copied()
    }

    /// Prints a text representation of the game board to stdout.
    pub fn print_board(&self) {
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                match &self.board[row][col] {
                    Some(u) => print!("{:^10}", u.value),
                    None => print!("{:^10}", '-'),
                }
            }
            println!();
        }
    }

    /// Sets the `merged` field to false for all Tiles before any move is calculated.
    fn reset_merged_flags(&mut self) {
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if self.board[row][col].is_some() {
                    self.board[row][col].as_mut().unwrap().merged = None;
                }
            }
        }
    }

    /// Returns: 
    /// 1) `true` if no more moves are possible.
    /// 2) `false` if more moves are still possible.
    ///
    /// The frontend will be responsible to determining if the game is won. See `Game` struct
    /// definition for more details.
    pub fn game_over(&self) -> bool {
        let directions = ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"];

        for direction in directions {
            let mut current_game = self.clone();

            if let InputResult::Ok(_, _, _) = current_game.receive_input(direction) {
                return false
            }
        }

        true
    }

    /// Receives the user's input and slides tiles in the specified direction.
    pub fn receive_input(&mut self, input: &str) -> InputResult {
        let mut move_occurred = false;
        let mut recycled_ids: Vec<usize> = Vec::new();
        self.reset_merged_flags();

        let mut winning_tile_reached = false;

        // i in the loops below represents the index difference between the Tile's starting slot
        // and its destination slot.
        // i will be incremented each time the Tile is shifted by one slot and until it can 
        // no longer be shifted.
        match input {
            "ArrowUp" | "KeyK" | "KeyW" => {
                for col in 0..BOARD_DIMENSION {
                    for row in 1..BOARD_DIMENSION {
                        let mut i = 1;

                        if let Some(mut tile) = self.board[row][col].take() {
                            // Loop until an occupied cell is found.
                            while row.checked_sub(i).is_some_and(|diff| self.board[diff][col].is_none()) {
                                i += 1;
                            }

                            // If subtraction causes an underflow, there is no tile between current
                            // tile and the board edge; slide the current tile to its destination.

                            // If no underflow occurs, there must be another tile present: perform
                            // merging logic.

                            // Double merges should not be allowed e.g. [2, 2, 2, 2] -> [0, 0, 4, 4] is a correct merge.
                            if row.checked_sub(i).is_some_and(|diff| self.board[diff][col].as_ref().unwrap().value == tile.value && self.board[diff][col].as_ref().unwrap().merged.is_none()) {
                                let removed_tile = self.board[row - i][col].take().unwrap();

                                winning_tile_reached = self.merge_tiles(&mut tile, removed_tile.clone(), &mut recycled_ids);
                                self.update_tile_and_board(tile, removed_tile.row, removed_tile.col);
                                move_occurred = true;
                            } else {
                                self.update_tile_and_board(tile, row - (i - 1), col);

                                if i > 1 {
                                    move_occurred = true;
                                }
                            }
                        }
                    }
                }
            },
            "ArrowDown" | "KeyJ" | "KeyS" => {
                for col in 0..BOARD_DIMENSION {
                    for row in (0..BOARD_DIMENSION - 1).rev() {
                        let mut i = 1;

                        if let Some(mut tile) = self.board[row][col].take() {
                            while row.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[sum][col].is_none()) {
                                i += 1;
                            }

                            // See comments for the "ArrowUp" case for an explanation of this merging logic
                            if row.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[sum][col].as_ref().unwrap().value == tile.value && self.board[sum][col].as_ref().unwrap().merged.is_none()) {
                                let removed_tile = self.board[row + i][col].take().unwrap();

                                winning_tile_reached = self.merge_tiles(&mut tile, removed_tile.clone(), &mut recycled_ids);
                                self.update_tile_and_board(tile, removed_tile.row, removed_tile.col);
                                move_occurred = true;
                            } else {
                                self.update_tile_and_board(tile, row + (i - 1), col);

                                if i > 1 {
                                    move_occurred = true;
                                }
                            }
                        }
                    }
                }
            }
            "ArrowLeft" | "KeyH" | "KeyA" => {
                for row in 0..BOARD_DIMENSION {
                    for col in 1..BOARD_DIMENSION {
                        let mut i = 1;

                        if let Some(mut tile) = self.board[row][col].take() {
                            while col.checked_sub(i).is_some_and(|diff| self.board[row][diff].is_none()) {
                                i += 1
                            }

                            // See comments for the "ArrowUp" case for an explanation of this merging logic
                            if col.checked_sub(i).is_some_and(|diff| self.board[row][diff].as_ref().unwrap().value == tile.value && self.board[row][diff].as_ref().unwrap().merged.is_none()) {
                                let removed_tile = self.board[row][col - i].take().unwrap();
                                
                                winning_tile_reached = self.merge_tiles(&mut tile, removed_tile.clone(), &mut recycled_ids);
                                self.update_tile_and_board(tile, removed_tile.row, removed_tile.col);

                                move_occurred = true;
                                // TODO: update background color to reflect the new value
                            } else {
                                self.update_tile_and_board(tile, row, col - (i - 1));

                                if i > 1 {
                                    move_occurred = true;
                                }
                            }
                        }
                    }
                }
            },
            "ArrowRight" | "KeyL" | "KeyD" => {
                for row in 0..BOARD_DIMENSION {
                    for col in (0..BOARD_DIMENSION - 1).rev() {
                        if let Some(mut tile) = self.board[row][col].take() {
                            let mut i = 1;

                            while col.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[row][sum].is_none()) {
                                i += 1;
                            }

                            // See comments for the "ArrowUp" case for an explanation of this merging logic
                            if col.checked_add_max(i, BOARD_DIMENSION).is_some_and(|sum| self.board[row][sum].as_ref().unwrap().value == tile.value && self.board[row][sum].as_ref().unwrap().merged.is_none()) {
                                let removed_tile = self.board[row][col + i].take().unwrap();

                                winning_tile_reached = self.merge_tiles(&mut tile, removed_tile.clone(), &mut recycled_ids);
                                self.update_tile_and_board(tile, removed_tile.row, removed_tile.col);

                                move_occurred = true;
                            } else {
                                self.update_tile_and_board(tile, row, col + (i - 1));

                                if i > 1 {
                                    move_occurred = true;
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }

        match move_occurred {
            true => match self.get_random_free_slot() {
                Some((i, j)) => {
                    // New tile ID should not use the ID of a tile that was merged this turn.
                    let new_id = self.get_id().unwrap();
                    self.recycle_ids(recycled_ids);

                    let new_tile_value = self.generate_tile_value();
                    let (tile_background, tile_text) = self.get_tile_colors(new_tile_value);

                    let new_tile = Tile::new(new_tile_value, new_id, tile_background, tile_text, i, j);
                    self.board[i][j] = Some(new_tile);

                    // Game can only be won the first time a winning tile is reached.
                    let mut send_game_won = false;

                    if !self.game_won && winning_tile_reached {
                        send_game_won = true;
                        self.game_won = true;
                    }

                    InputResult::Ok(new_id, self.get_tiles(), send_game_won)
                },
                None => unreachable!(),
            }
                ,
            false => InputResult::Err(InvalidMove),
        }
    }

    /// Accepts two Tile references and performs necessary steps in merging them. This involves
    /// storing the removed Tile in the resultant Tile's `merged` field and updating the Vec
    /// of recycled IDs with the removed Tile's ID. 
    ///
    /// The resultant Tile needs to maintain a clone of the removed Tile so that the frontend has
    /// access to the removed Tile's coordinates. This is necessary because the removed Tile needs
    /// to be moved into its final position before being deleted for animation integrity.
    ///
    /// The resultant Tile's value is doubled to reflect the merge and the score is incremented by 
    /// this new value. Finally the resultant Tile's color is also updated to reflect its new value.
    ///
    /// If the resultant Tile's value is equal to self.WINNING_TILE, return true. Else return false.
    fn merge_tiles(&mut self, merged_tile: &mut Tile, removed_tile: Tile, recycled_ids: &mut Vec<usize>) -> bool {
        recycled_ids.push(removed_tile.id);
        merged_tile.merged = Some(Box::new(removed_tile));

        merged_tile.value *= 2;
        self.score += merged_tile.value;

        (merged_tile.background_color, merged_tile.text_color) = self.get_tile_colors(merged_tile.value);

        if merged_tile.value == Game::WINNING_TILE {
            true
        } else {
            false
        }
    }

    /// Receives a tile, the new row and col indexes, and updates both the tile's internal row and
    /// col fields and places the tile in self.board's new location.
    fn update_tile_and_board(&mut self, mut tile: Tile, new_row: usize, new_col: usize) {
        tile.row = new_row;
        tile.col = new_col;

        self.board[new_row][new_col] = Some(tile);
    }

    /// Returns tuple of (background_color, text_color) based on tile_value input.
    /// Background color is based on a color interpolation algorithm:
    /// 1) 4 base colors are initialized in an array.
    /// 2) Every 4th power of 2 uses the next base color from the array.
    /// 3) All powers of 2 between multiples of 4 are interpolated between the two base colors.
    fn get_tile_colors(&self, tile_value: u32) -> (String, String) {
        let base_colors: [&str; 4] = [
                                      "#f2ba0d", // Yellow // Yellow // Yellow
                                      "#F50A40", // Magenta
                                      "#3949AB", // Blue
                                      "#6A0DAD", // Purple
                                      ];

        let num_interpolation_steps = 3;

        // Minus 1 is because tiles start at 2^1 rather than 2^0.
        let log_2 = (log_2(tile_value) - 1) as usize;
        let base_color_index = (log_2 / num_interpolation_steps) % base_colors.len();
        let interpolation_offset = (log_2 % num_interpolation_steps) as f32;

        let other_color_index;

        if base_color_index == base_colors.len() - 1 {
            other_color_index = 0;
        } else {
            other_color_index = base_color_index + 1;
        }

        let base_color = HexColor::parse(base_colors[base_color_index]).unwrap();
        let other_color = HexColor::parse(base_colors[other_color_index]).unwrap();

        let interpolated_color = interpolate_hex_colors(&base_color, &other_color, interpolation_offset / num_interpolation_steps as f32);
        let tile_background = interpolated_color.to_string();

        let relative_luminance = 0.2126 * interpolated_color.r as f32 +
                                 0.7152 * interpolated_color.g as f32 +
                                 0.0722 * interpolated_color.b as f32;

        let relative_luminance = relative_luminance / 255.0;

        let tile_text;
        let colors = Colors::new();

        // log!("Tile value:", tile_value);
        // log!("Relative luminance:", relative_luminance);
        
        if relative_luminance <= 0.35 {
            tile_text = colors.text_light;
            // log!("Light text.")
        } else {
            tile_text = colors.text_dark;
            // log!("Dark text.")
        }

        (tile_background.to_string(), tile_text.to_string())
    }
}

// Helper functions

/// Computes log base 2 for a u32.
fn log_2(mut num: u32) -> u32 {
    let mut log = 0;

    while num > 1 {
        num /= 2;
        log += 1;
    }

    log
}

fn interpolate_hex_colors(color1: &HexColor, color2: &HexColor, t: f32) -> HexColor {
    let r = interpolate_component(color1.r, color2.r, t);
    let g = interpolate_component(color1.g, color2.g, t);
    let b = interpolate_component(color1.b, color2.b, t);

    let hex_formatted = format!("#{}{}{}", r, g, b);
    HexColor::parse_rgb(&hex_formatted).expect(&hex_formatted)
}

fn interpolate_component(c1: u8, c2: u8, t: f32) -> String {
    let result = ((1.0 - t) * c1 as f32 + t * c2 as f32).round() as i32;
    let clamped_result = result.max(0).min(255) as u8;
    format!("{:02X}", clamped_result)
}

trait CheckedAdd {
    fn checked_add_max(self, rhs: usize, max: usize) -> Option<usize>;
}

/// Similar to the builtin `checked_add()` method but allows for defining a custom max
impl CheckedAdd for usize {
    fn checked_add_max(self, rhs: Self, max: Self) -> Option<Self> {
        let sum = self + rhs;

        if sum < max {
            Some(sum)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Ensure that the generation of 2-tiles outnumbers the generation of 4-tiles 4:1 given a
    /// sufficiently large sample size and across multiple trials.
    fn test_new_tile_rng() {
        let game = Game::new();
        let num_trials = 100;

        for i in 0..num_trials {
            println!("Test iteration: {i}");

            let mut two_count = 0;
            let mut four_count = 0;

            const SAMPLE_SIZE: u32 = 10000;

            for _ in 0..SAMPLE_SIZE {
                let tile = game.generate_tile_value();

                if tile == game.new_tile_params.tile_choices[NewTileParams::TWO] {
                    two_count += 1;
                } else {
                    four_count += 1;
                }
            }

            let two_dist = two_count as f32 / SAMPLE_SIZE as f32;
            let four_dist = four_count as f32 / SAMPLE_SIZE as f32;

            let expected_ratio = game.new_tile_params.tile_weights[NewTileParams::TWO] as f32;
            let actual_ratio = two_dist / four_dist;

            // Run `cargo test -- --nocapture` to show stdout
            println!("Expected 2:4 ratio: {expected_ratio}:1");
            println!("Actual 2:4 ratio: {actual_ratio}:1");
            
            let error_margin = expected_ratio * 0.20;
            let expected_ratio_range = (expected_ratio - error_margin)..=(expected_ratio + error_margin);

            assert!(expected_ratio_range.contains(&actual_ratio));
        }
    }

    #[test]
    /// Ensure that the maintainance and random selection of free slots is working correctly. 
    fn test_updating_and_randomly_selecting_free_slots() {
        let mut game = Game::new();
        const NUM_STARTING_TILES: usize = 2;

        // Ensure that number of starting tiles is correct.
        game.update_free_slots();
        assert_eq!(game.free_slots.len(), NUM_TILES - NUM_STARTING_TILES);

        // Fill all empty slots with placeholders.
        for _ in NUM_STARTING_TILES..NUM_TILES {
            let coord = game.get_random_free_slot();

            match coord {
                Some((row, col)) => game.board[row][col] = 
                    Some(Tile::new(0, 0, "orange".to_string(), "pink".to_string(), row, col)),
                None => panic!("Game board filled up unexpectedly."),
            }
        }

        // Ensure that all board slots are filled.
        game.update_free_slots();
        assert_eq!(game.free_slots.len(), 0);

        // Brute force assurance that all board slots are filled.
        for row in 0..BOARD_DIMENSION {
            for col in 0..BOARD_DIMENSION {
                if let None = game.board[row][col] {
                    panic!("Free board slots remain after filling with placeholders.");
                }
            }
        }

        // Ensure that attempting to obtain a free slot when the board is full returns `None`.
        assert_eq!(game.get_random_free_slot(), None);
    }

    #[test]
    /// A new game should:
    ///
    /// 1) Be empty save for two initial tiles.
    /// 2) Have one 4-tile and one 2-tile -OR- two 2-tiles.
    ///
    /// Running multiple trials due the random nature of new game generation.
    fn test_new_game() {
        const NUM_STARTING_TILES: usize = 2;
        let num_trials = 100;

        for _ in 0..num_trials {
            let game = Game::new();
            let mut starting_tiles = Vec::new();

            for row in 0..BOARD_DIMENSION {
                for col in 0..BOARD_DIMENSION {
                    if let Some(u) = &game.board[row][col] {
                        starting_tiles.push(u);
                    }
                }
            }

            // Check that number of starting tiles is correct.
            assert_eq!(starting_tiles.len(), NUM_STARTING_TILES);
            
            // Check that starting tiles are valid.
            assert!(game.new_tile_params.tile_choices.contains(&starting_tiles[0].value));
            assert!(game.new_tile_params.tile_choices.contains(&starting_tiles[1].value));
            
            // Check condition 2)
            if starting_tiles[0].value == starting_tiles[1].value {
                assert_eq!(starting_tiles[0].value, game.new_tile_params.tile_choices[NewTileParams::TWO]);
            }
        }
    }

    #[test]
    /// Tests whether tiles are generating the correct colors.
    fn test_color_generator() {
        let game = Game::new();

        let base: u32 = 2;
        let mut power = 1;
        let max_power = 30;

        while power < max_power {
            let tile_value = base.pow(power);
            let tile_color = game.get_tile_colors(tile_value);
            print_color_to_stdout(tile_color.0, tile_value);
            power += 1;
        }
    }

    fn print_color_to_stdout(hex_color: String, tile_value: u32) {
        let color = &hex_color[1..];

        // Convert the hexadecimal color to RGB values
        let red = u8::from_str_radix(&color[0..2], 16).unwrap_or(0);
        let green = u8::from_str_radix(&color[2..4], 16).unwrap_or(0);
        let blue = u8::from_str_radix(&color[4..6], 16).unwrap_or(0);

        // Generate the ANSI escape code for the RGB color
        let formatted_color = format!("\x1b[38;2;{};{};{}m", red, green, blue);
        // ANSI escape code for resetting text color
        let reset_code = "\x1b[0m";

        println!("Tile color is {}{}{}", formatted_color, tile_value, reset_code);
    }
}

