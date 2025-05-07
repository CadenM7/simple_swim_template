#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};

use core::{
    clone::Clone,
    cmp::{min, Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive,
};

const NUM_WINDOWS: usize = 4;
const COL: usize = BUFFER_HEIGHT / 2;
const ROW: usize = BUFFER_WIDTH / 2;


#[derive(Copy, Clone, Eq, PartialEq)]
struct Window {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,

}
pub struct SwimInterface {
    letters: [[[char; BUFFER_WIDTH]; BUFFER_HEIGHT];NUM_WINDOWS],
    num_letters: [usize; NUM_WINDOWS],
    next_letter: [usize; NUM_WINDOWS],
    col: [usize; NUM_WINDOWS],
    row: [usize; NUM_WINDOWS],
    windows: [Window; NUM_WINDOWS],
    active_window: usize,
}

pub fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

pub fn add1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, 1)
}

pub fn sub1<const LIMIT: usize>(value: usize) -> usize {
    safe_add::<LIMIT>(value, LIMIT - 1)
}

impl Default for SwimInterface {
    fn default() -> Self {
        Self {
            letters: [[[ '_' ; BUFFER_WIDTH]; BUFFER_HEIGHT]; NUM_WINDOWS],
            num_letters: [1; NUM_WINDOWS], 
            next_letter: [0; NUM_WINDOWS],
            col: [1; NUM_WINDOWS],
            row: [1; NUM_WINDOWS],
            windows: [
                Window { top: 0, left: 0, bottom: COL - 1, right: ROW - 1 },
                Window { top: 0, left: ROW, bottom: COL - 1, right: BUFFER_WIDTH - 1 },
                Window { top: COL, left: 0, bottom: BUFFER_HEIGHT - 1, right: ROW - 1 },
                Window { top: COL, left: ROW, bottom: BUFFER_HEIGHT - 1, right: BUFFER_WIDTH - 1 },
            ],
            active_window: 0,
        }
    }
}

impl SwimInterface {
    fn letter_columns(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.num_letters[self.active_window]).map(|n| safe_add::<BUFFER_WIDTH>(n, self.col[self.active_window]))
    }
    pub fn tick(&mut self) {
        self.clear_current();
        self.draw_all_windows();
        self.draw_current();
    }

    fn draw_window(&self, active: usize) {
        let window = self.windows[active];
        let mut color = <ColorCode>::new(Color::Black, Color::Black);
        if active == self.active_window {
            color = ColorCode::new(Color::Black, Color::Green);
        } else {
            color = ColorCode::new(Color::Green, Color::Black);
        }

        for x in window.left..=window.right {
            plot('.', x, window.top, color);
            plot('.', x, window.bottom, color);
        }
        for y in window.top..=window.bottom {
            plot('.', window.left, y, color);
            plot('.', window.right, y, color);
        }

        let header = match active {
            0 => "F1",
            1 => "F2",
            2 => "F3",
            3 => "F4",
            _ => "",
        };

        let center = (window.left + window.right) / 2;
        for (i, c) in header.chars().enumerate() {
            plot(c, center + i, window.top, color);
        }
    }

    pub fn draw_all_windows(&self) {
        for i in 0..NUM_WINDOWS {
            self.draw_window(i);
        }
    }

    fn draw_current(&self) {
        let window = self.windows[self.active_window];
        let start_row = window.top + self.row[self.active_window];
        let start_col = window.left + self.col[self.active_window];

        for (i, x) in self.letter_columns().enumerate() {
            let col = start_col + x;
            if col < window.right {
                plot(
                    self.letters[self.active_window][self.row[self.active_window]][i],
                    col,
                    start_row,
                    ColorCode::new(Color::Green, Color::Black),
                );
            }
        }
    }

    fn clear_current(&self) {
        let window = self.windows[self.active_window];
        let start_row = window.top + 1 + self.row[self.active_window];
        let start_col = window.left + 1;

        for x in self.letter_columns() {
            let col = start_col + x;
            if col < window.right {
                plot(' ', col, start_row, ColorCode::new(Color::Black, Color::Black));
            }
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => self.active_window = 0,
            KeyCode::F2 => self.active_window = 1,
            KeyCode::F3 => self.active_window = 2,
            KeyCode::F4 => self.active_window = 3,
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if is_drawable(key) {
            self.letters[self.active_window][self.row[self.active_window]][self.next_letter[self.active_window]] = key;
            self.next_letter[self.active_window] = add1::<BUFFER_WIDTH>(self.next_letter[self.active_window]);
            self.num_letters[self.active_window] = min(self.num_letters[self.active_window] + 1, BUFFER_WIDTH);

            if self.next_letter[self.active_window] == BUFFER_WIDTH - 1 {
                self.row[self.active_window] = min(add1::<BUFFER_HEIGHT>(self.row[self.active_window]), BUFFER_HEIGHT - 1);
                self.num_letters[self.active_window] = 0;
            }

        }
        if key =='\n' {
            self.row[self.active_window] = min(add1::<BUFFER_HEIGHT>(self.row[self.active_window]), BUFFER_HEIGHT - 1);
            self.num_letters[self.active_window] = 1;
            self.next_letter[self.active_window] = 0;
            self.letters[self.active_window][self.row[self.active_window]] = ['_'; BUFFER_WIDTH]; 
            self.draw_current();
        }
    }
}

    

