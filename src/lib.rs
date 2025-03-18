#![no_std]

use core::arch::x86_64::_rdtsc;
use itoa::Buffer;
use log::{info, warn};
use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::{serial_print, print, vga_buffer::{
    clear_screen, is_drawable, plot, plot_num, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
}};

use core::{
    clone::Clone, cmp::{min, Eq, PartialEq}, iter::Iterator, marker::Copy, ops::Range, prelude::rust_2024::derive
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
    characters: [char; 3],
}

#[derive(Copy, Clone)]
pub struct EnemyObj {
    id : usize,
    move_delay : usize,
    max_delay : usize,
    alive: bool,
    pos_x: usize,
    pos_y: usize,
    characters: [char; 5],
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
    tick_count : usize,
    tick_delay : usize,
    projectile_count : usize,
    active_enemies : usize,
    health : usize,
    rng : oorandom::Rand32,
    init : bool,
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
            characters: ['/','|','\\'],
        }
    }
}
impl Default for EnemyObj {
    fn default() -> Self {
        Self {
            id : 0,
            max_delay : 10,
            move_delay : 10,
            pos_x: 0,
            pos_y: 0,
            characters: ['<', '#', '.', '#', '>'],
            alive: false,
        }
    }
}

impl Default for GamePlayer {
    fn default() -> Self {
        Self {
            enemies: [EnemyObj::default(); 10],
            player: PlayerObj::default(),
            projectiles : [Projectile {
                id : 0,
                char: '*',
                active: false,
                pos_x: 0,
                pos_y: 0,
            }; BUFFER_HEIGHT],
            init : false,
            health : 10,
            tick_count : 0,
            tick_delay : 1,
            projectile_count : 0,
            active_enemies : 0,
            rng : unsafe {
                oorandom::Rand32::new(_rdtsc())
            },
        }
    }
}

// fn generate_enemies() -> [EnemyObj; 10] {
//     let final_enemies = [EnemyObj::default(); 10];
//     let mut i = 0;
//     for mut enemy in final_enemies {
//         enemy.id = i;
//         i += 1;
//     }
//     return final_enemies;
// }

// fn generate_proj() -> [Projectile; BUFFER_HEIGHT] {
//     let final_projectiles = [Projectile {
//         id : 0,
//         char: '*',
//         active: false,
//         pos_x: 0,
//         pos_y: 0,
//     }; BUFFER_HEIGHT];
//     let mut i = 0;
//     for mut proj in final_projectiles {
//         proj.change_id(i);
//         serial_print!("{}",proj.id);
//         i += 1;
//     }
//     return final_projectiles
// }

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

impl Projectile {
    pub fn change_id(&mut self, new_id: usize) {
        self.id = new_id;
    }
    pub fn update_pos_y(&mut self, new_pos: usize) {
        self.pos_x = new_pos;
    }
}
impl GamePlayer {
    pub fn tick(&mut self) {
        if self.init == false {
            self.initialize();
        }
        if self.health > 0
        {
            if self.tick_count >= self.tick_delay {
                if self.active_enemies < 10 {
                    self.active_enemies += 1;
                    self.add_enemy();
                }
                clear_screen();
                self.move_items();
                self.draw();
                self.tick_count = 0;
            } else {
                self.tick_count += 1
            }
        } else {
            serial_print!("Dead");
            clear_screen();
            self.death_screen();
        }
    }
    pub fn input(&mut self, key: DecodedKey) {
        if self.init == false {
            self.initialize();
        }
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }
    fn initialize(&mut self) {
        let mut i = 0;
        while i < self.projectiles.len() {
            self.projectiles[i].id = i;
            i += 1
        }
        i = 0;
        while i < self.enemies.len() {
            let move_delay = self.rng.rand_range(Range{ start: 3, end : 10});
            self.enemies[i].id = i;
            self.enemies[i].max_delay = move_delay as usize;
            i += 1;
        }
        self.init = true
    }
    fn death_screen(&mut self) {
        let death_message = "You Died!";
        let mut i = 0;
        for character in death_message.chars() {
            plot(character, BUFFER_HEIGHT/2 + i, BUFFER_WIDTH/2, ColorCode::new(Color::Red, Color::Black));
            serial_print!("{}", character);
            i += 1
        }
    }

    fn move_items(&mut self) {
        let mut i = 0;
        while i < self.enemies.len() {
            let enemy = self.enemies[i];
            if enemy.alive == true {
                if enemy.move_delay <= 0 {
                    if enemy.pos_y < (BUFFER_HEIGHT-1) {
                        serial_print!("{}", enemy.id);
                        self.enemies[i].pos_y += 1;
                    } else {
                        serial_print!("Enemy Id Reached: {}", enemy.id);
                        self.health -= 1;
                        self.enemies[i].alive = false;
                        self.active_enemies -= 1;
                    }
                    self.enemies[i].move_delay = enemy.max_delay;
                } else {
                    self.enemies[i].move_delay -= 1;
                }
            }
            i += 1;
        }
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles[i];
            // serial_print!("{}",projectile.id);
            if projectile.active {
                if projectile.pos_y > 1 {
                    self.projectiles[i].pos_y -= 1;
                    let mut x = 0;
                    while x < self.enemies.len() {
                        let enemy = self.enemies[x];
                        if enemy.alive {
                            if enemy.pos_y == self.projectiles[i].pos_y {
                                let size = enemy.characters.len()/2;
                                if self.projectiles[i].pos_x.abs_diff(enemy.pos_x) <= size {
                                    if self.enemies[x].alive == true {
                                        self.enemies[x].alive = false;
                                        self.active_enemies -= 1;
                                        self.projectiles[i].active = false;
                                    }
                                }
                            }
                        }
                        x+=1
                    }
                } else {
                    self.projectiles[i].active = false;
                }
            }
            i += 1
        }
    }
    fn draw(&mut self) {
        plot_num(self.health as isize, 0, 0, ColorCode::new(Color::Green, Color::Black));
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
                let mut i = 0;
                for char in self.player.characters {
                    if self.player.pos_x >= player_offset {
                    let draw_pos = self.player.pos_x - player_offset + i;
                    if draw_pos > 0 && draw_pos < BUFFER_WIDTH {
                        plot(char, draw_pos, self.player.pos_y, ColorCode::new(Color::Blue, Color::Black));
                    }
                }
                    i += 1;
                }
            } else {
                plot(self.player.characters[0], self.player.pos_x, self.player.pos_y, ColorCode::new(Color::Blue, Color::Black));
            }
        } else {
            print!("Error: Non-Odd player character count")
        }
        let mut i = 0;
        while i < self.projectiles.len() {
            let projectile = self.projectiles[i];
            if projectile.active {
                plot(projectile.char, projectile.pos_x, projectile.pos_y, ColorCode::new(Color::Green, Color::Black));
            }
            i += 1;
        }
        
    }

    fn add_enemy(&mut self) {
        let mut id = 0;
        for enemy in self.enemies {
            serial_print!("{}", enemy.id);
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
                    self.player.pos_x -= 1;
                } else  {
                    self.player.pos_x = BUFFER_WIDTH;
                }
                self.draw();
            }
            'd' => {
                self.player.pos_x = safe_add::<BUFFER_WIDTH>(self.player.pos_x, 1); 
                self.draw();
            }
            'w' => {
                self.init_proj();
            }
            _ => {}
        }
    }
    fn handle_raw(&mut self, code : KeyCode) {
        match code {
            KeyCode::Spacebar => {
                self.projectile_count += 1;
                self.init_proj();
            }
            _ => {

            }
        }
    }
    fn init_proj(&mut self) {
        let mut id = 0;
        for proj in self.projectiles {
            if proj.active == false {
                id = proj.id
            }
        }
        let new_x_pos = self.player.pos_x;
        let new_y_pos = self.player.pos_y - 1;
        self.projectiles[id].active = true;
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
