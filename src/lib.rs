#![no_std]

use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::{print, vga_buffer::{
    clear_screen, is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
}};

use core::{
    clone::{self, Clone}, cmp::{min, Eq, PartialEq}, error, iter::Iterator, marker::Copy, ops::Range, prelude::rust_2024::derive
};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LetterMover {
    letters: [char; BUFFER_WIDTH],
    num_letters: usize,
    next_letter: usize,
    col: usize,
    row: usize,
    dx: usize,
    dy: usize,
}

pub struct PlayerObj {
    pos_x: usize,
    pos_y: usize,
    characters: [char; 1],
}

#[derive(Copy, Clone)]
pub struct EnemyObj {
    id : usize,
    alive: bool,
    pos_x: usize,
    pos_y: usize,
    characters: [char; 3],
}

#[derive(Copy, Clone)]
pub struct Projectile {
    id : usize,
    char: char,
    active: bool,
    pos_x: usize,
    pos_y: usize,
}

pub struct GamePlayer {
    enemies: [EnemyObj; 10],
    player: PlayerObj,
    projectiles: [Projectile; BUFFER_HEIGHT],
    projectile_count : usize,
    active_enemies : usize,
    health : usize,
    rng : oorandom::Rand32,
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

impl Default for PlayerObj {
    fn default() -> Self {
        Self {
            pos_x: BUFFER_WIDTH / 2,
            pos_y: BUFFER_HEIGHT - 2,
            characters: ['*'],
        }
    }
}
impl Default for EnemyObj {
    fn default() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            characters: ['<', '.', '>'],
            alive: false,
        }
    }
}

impl Default for GamePlayer {
    fn default() -> Self {
        Self {
            enemies: [EnemyObj::default(); 10],
            player: PlayerObj::default(),
            projectiles: [Projectile {
                id : 0,
                char: '*',
                active: false,
                pos_x: 0,
                pos_y: 0,
            }; BUFFER_HEIGHT],
            health : 100,
            projectile_count : 0,
            active_enemies : 0,
            rng : oorandom::Rand32::new(50)
        }
    }
}

impl Default for LetterMover {
    fn default() -> Self {
        Self {
            letters: ['A'; BUFFER_WIDTH],
            num_letters: 1,
            next_letter: 1,
            col: BUFFER_WIDTH / 2,
            row: BUFFER_HEIGHT / 2,
            dx: 0,
            dy: 0,
        }
    }
}
impl GamePlayer {
    pub fn tick(&mut self) {
        if self.health > 0
        {
            if self.active_enemies < 10 {
                self.active_enemies += 1;
                self.add_enemy(self.active_enemies - 1);
            }
            self.clear_screen();
            self.move_items();
            self.draw();
        } else {
            self.death_screen();
        }
    }
    pub fn input(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn death_screen(&self) {
        let death_message = "You Died!";
        let mut i = 0;
        for char in death_message.chars() {
            plot(char, BUFFER_HEIGHT/2, BUFFER_WIDTH + i, ColorCode::new(Color::Black, Color::Black));
            i += 1;
        }
    }
    fn clear_screen(&mut self) {
        for enemy in self.enemies {
                if enemy.characters.len() % 2 == 1 {
                    if enemy.characters.len() > 1 {
                        let offset = enemy.characters.len()/2;
                        let mut i = 0;
                        for _char in enemy.characters {
                            if enemy.pos_x >= offset {
                                plot(' ', enemy.pos_x - offset + i, enemy.pos_y, ColorCode::new(Color::Black, Color::Black));
                                i += 1;
                            }
                        }
                    } else {
                        plot(' ', enemy.pos_x, enemy.pos_y, ColorCode::new(Color::Black, Color::Black));
                    }
                } else {
                    print!("Error: Non-Odd enemy character count")
                }
        }
        if self.player.characters.len() % 2 == 1 {
            if self.player.characters.len() > 1 {
                let player_offset = self.player.characters.len()/2;
                for _char in self.player.characters {
                    plot(' ', self.player.pos_x - player_offset, self.player.pos_y, ColorCode::new(Color::Black, Color::Black));
                }
            } else {
                plot(' ', self.player.pos_x, self.player.pos_y, ColorCode::new(Color::Black, Color::Black));
            }
        } else {
            print!("Error: Non-Odd player character count")
        }
        for projectile in self.projectiles {
            if projectile.active {
                plot(' ', projectile.pos_x, projectile.pos_y, ColorCode::new(Color::Black, Color::Black));
            }
        }
    }
    fn move_items(&mut self) {
        for mut enemy in self.enemies {
            if enemy.alive == true {
                if enemy.pos_y < BUFFER_HEIGHT {
                    enemy.pos_y += 1;
                } else {
                    self.health -= 1;
                    enemy.alive = false;
                }
            }
        }
        for mut projectile in self.projectiles {
            if projectile.active {
                if projectile.pos_x > 0 {
                    projectile.pos_x -= 1;
                } else {
                    projectile.active = false;
                    self.projectile_count -= 1;
                }
            }
        }
    }
    fn draw(&mut self) {
        for enemy in self.enemies {
            if enemy.alive == true {
                if enemy.characters.len() % 2 == 1 {
                    if enemy.characters.len() > 1 {
                        let offset = enemy.characters.len()/2;
                        let mut i = 0;
                        for char in enemy.characters {
                            if enemy.pos_x >= offset {
                                plot(char, enemy.pos_x - offset + i, enemy.pos_y, ColorCode::new(Color::Red, Color::Black));
                                i += 1;
                            }
                        }
                    } else {
                        plot(enemy.characters[0], enemy.pos_x, enemy.pos_y, ColorCode::new(Color::Red, Color::Black));
                    }
                } else {
                    print!("Error: Non-Odd enemy character count")
                }
            }
        }
        if self.player.characters.len() % 2 == 1 {
            if self.player.characters.len() > 1 {
                let player_offset = self.player.characters.len()/2;
                for char in self.player.characters {
                    plot(char, self.player.pos_x - player_offset, self.player.pos_y, ColorCode::new(Color::Blue, Color::Black));
                }
            } else {
                plot(self.player.characters[0], self.player.pos_x, self.player.pos_y, ColorCode::new(Color::Blue, Color::Black));
            }
        } else {
            print!("Error: Non-Odd player character count")
        }
        for projectile in self.projectiles {
            if projectile.active {
                plot(projectile.char, projectile.pos_x, projectile.pos_y, ColorCode::new(Color::Green, Color::Black));
            }
        }
    }

    fn add_enemy(&mut self) {
        let mut id = 0;
        for enemy in self.enemies {
            if enemy.alive == false {
                id = enemy.id
            }
        }
        let new_x_pos = self.rng.rand_range(Range{start : 0, end : BUFFER_WIDTH as u32});
        self.enemies[id].alive = true;
        self.enemies[id].pos_x = new_x_pos as usize;
        self.enemies[id].pos_y = 0;
    }
    fn handle_unicode(&mut self, c: char) {
        match c {
            'a' => {
                if self.player.pos_x > 0 {
                    self.player.pos_x = self.player.pos_x - 1;
                }
            }
            'd' => {
                self.player.pos_x = safe_add::<BUFFER_WIDTH>(self.player.pos_x,1); 
            }
            _ => {}
        }
    }
    fn handle_raw(&mut self, code : KeyCode) {
        match code {
            KeyCode::Spacebar => {
                self.projectile_count += 1;
                self.init_proj(self.projectile_count - 1)
            }
            _ => {

            }
        }
    }
    fn init_proj(&mut self, id : usize) {
        let new_x_pos = self.player.pos_x;
        let new_y_pos = self.player.pos_y - 1;
        self.projectiles[id].pos_x = new_x_pos;
        self.projectiles[id].pos_y = new_y_pos;
    }
}
impl LetterMover {
    fn letter_columns(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.num_letters).map(|n| safe_add::<BUFFER_WIDTH>(n, self.col))
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        for x in self.letter_columns() {
            plot(' ', x, self.row, ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn update_location(&mut self) {
        self.col = safe_add::<BUFFER_WIDTH>(self.col, self.dx);
        self.row = safe_add::<BUFFER_HEIGHT>(self.row, self.dy);
    }

    fn draw_current(&self) {
        for (i, x) in self.letter_columns().enumerate() {
            plot(
                self.letters[i],
                x,
                self.row,
                ColorCode::new(Color::Cyan, Color::Black),
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
            KeyCode::ArrowLeft => {
                self.dx = sub1::<BUFFER_WIDTH>(self.dx);
            }
            KeyCode::ArrowRight => {
                self.dx = add1::<BUFFER_WIDTH>(self.dx);
            }
            KeyCode::ArrowUp => {
                self.dy = sub1::<BUFFER_HEIGHT>(self.dy);
            }
            KeyCode::ArrowDown => {
                self.dy = add1::<BUFFER_HEIGHT>(self.dy);
            }
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if is_drawable(key) {
            self.letters[self.next_letter] = key;
            self.next_letter = add1::<BUFFER_WIDTH>(self.next_letter);
            self.num_letters = min(self.num_letters + 1, BUFFER_WIDTH);
        }
    }
}
