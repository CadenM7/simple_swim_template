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

#[derive(Copy, Clone, Eq, PartialEq)]
struct Window {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,

}
pub struct SwimInterface {
    letters: [[[char; BUFFER_WIDTH]; BUFFER_HEIGHT];NUM_WINDOWS],
    num_letters: usize,
    next_letter: usize,
    col: usize,
    row: usize,
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
        let mid_x = BUFFER_WIDTH / 2;
        let mid_y = BUFFER_HEIGHT / 2;
        Self {
            letters: [[[ '_' ; BUFFER_WIDTH]; BUFFER_HEIGHT]; NUM_WINDOWS],
            num_letters: 1,
            next_letter: 0,
            col: 1,
            row: 1,
            windows: [
                Window { top: 0, left: 0, bottom: mid_y - 1, right: mid_x - 1 },
                Window { top: 0, left: mid_x, bottom: mid_y - 1, right: BUFFER_WIDTH - 1 },
                Window { top: mid_y, left: 0, bottom: BUFFER_HEIGHT - 1, right: mid_x - 1 },
                Window { top: mid_y, left: mid_x, bottom: BUFFER_HEIGHT - 1, right: BUFFER_WIDTH - 1 },
            ],
            active_window: 0,
        }
    }
}

impl SwimInterface {
    fn letter_columns(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.num_letters).map(|n| safe_add::<BUFFER_WIDTH>(n, self.col))
    }
    pub fn tick(&mut self) {
        self.clear_current();
        self.draw_current();
        self.draw_all_windows();
    }

    fn clear_current(&self) {
        for x in self.letter_columns() {
            plot(' ', x, self.row, ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn draw_window(&self, active: usize) {
        let window = self.windows[active];
        let mut color = ColorCode::new(Color::Black, Color::Black);
        if active == self.active_window {
            color = ColorCode::new(Color::Green, Color::Black);
        } else {
            color = ColorCode::new(Color::White, Color::Black);
        }

        for x in window.left..=window.right {
            plot('.', x, window.top, color);
            plot('.', x, window.bottom, color);
        }
        for y in window.top..=window.bottom {
            plot('.', window.left, y, color);
            plot('.', window.right, y, color);
        }
    }

    pub fn draw_all_windows(&self) {
        for i in 0..NUM_WINDOWS {
            self.draw_window(i);
        }
    }


    fn draw_current(&self) {
        for (i, x) in self.letter_columns().enumerate() {
            plot(
                self.letters[self.active_window][i][self.next_letter],
                x,
                self.row,
                ColorCode::new(Color::Green, Color::Black),
            );
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
            self.letters[self.active_window][self.row][self.next_letter] = key;
            self.next_letter = add1::<BUFFER_WIDTH>(self.next_letter);
            self.num_letters = min(self.num_letters + 1, BUFFER_WIDTH);

            if self.next_letter == BUFFER_WIDTH - 1 {
                self.row = min(add1::<BUFFER_HEIGHT>(self.row), BUFFER_HEIGHT - 1);
                self.num_letters = 0;
            }

        }
        if key =='\n' {
            self.row = min(add1::<BUFFER_HEIGHT>(self.row), BUFFER_HEIGHT - 1);
            self.num_letters = 1;
            self.next_letter = 0;
            self.letters[self.active_window][self.row] = ['_'; BUFFER_WIDTH]; 
            self.draw_current();
        }
    }
}

    

