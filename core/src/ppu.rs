use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const DISPLAY_WIDTH: usize = 160;
const DISPLAY_HEIGHT: usize = 144;
const DISPLAY_SIZE: usize = DISPLAY_HEIGHT * DISPLAY_WIDTH;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Clone, Copy, Debug)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LCDControl {
    pub lcd_enable: bool,
    pub win_tile_map_area: bool,
    pub win_enable: bool,
    pub bg_win_tile_data_area: bool,
    pub bg_tile_map_area: bool,
    pub obj_size: bool,
    pub obj_enable: bool,
    pub bg_win_enable: bool,
}

impl From<u8> for LCDControl {
    fn from(value: u8) -> Self {
        Self {
            lcd_enable: value & (1 << 7) != 0,
            win_tile_map_area: value & (1 << 6) != 0,
            win_enable: value & (1 << 5) != 0,
            bg_win_tile_data_area: value & (1 << 4) != 0,
            bg_tile_map_area: value & (1 << 3) != 0,
            obj_size: value & (1 << 2) != 0,
            obj_enable: value & (1 << 1) != 0,
            bg_win_enable: value & 1 != 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LCDStatus {
    pub ly_interrupt_enable: bool,
    pub oam_interrupt_enable: bool,
    pub vblank_interrupt_enable: bool,
    pub hblank_interrupt_enable: bool,
    pub ly_compare: bool,
    pub mode: u8,
}

impl From<u8> for LCDStatus {
    fn from(value: u8) -> Self {
        Self {
            ly_interrupt_enable: (value & (1 << 6)) != 0,
            oam_interrupt_enable: (value & (1 << 5)) != 0,
            vblank_interrupt_enable: (value & (1 << 4)) != 0,
            hblank_interrupt_enable: (value & (1 << 3)) != 0,
            ly_compare: (value & (1 << 2)) != 0,
            mode: value & 0x3,
        }
    }
}

impl Into<u8> for LCDStatus {
    fn into(self) -> u8 {
        let mut value = 0;
        if self.ly_interrupt_enable {
            value |= 1 << 6;
        }
        if self.oam_interrupt_enable {
            value |= 1 << 5;
        }
        if self.vblank_interrupt_enable {
            value |= 1 << 4;
        }
        if self.hblank_interrupt_enable {
            value |= 1 << 3;
        }
        if self.ly_compare {
            value |= 1 << 2;
        }
        value |= self.mode;
        value
    }
}

pub struct PPU {
    pub memory: Rc<RefCell<Memory>>,
    pub clocks_to_finish: usize,
    pub frame_buffer: [u8; DISPLAY_SIZE * 4],
    pub obj_idx: Vec<(usize, usize)>,
    pub window_line_counter: usize,
    pub wy_cond_triggered: bool,
}

impl PPU {
    pub fn new(memory: Rc<RefCell<Memory>>) -> PPU {
        PPU {
            memory,
            clocks_to_finish: 0,
            frame_buffer: [0; DISPLAY_SIZE * 4],
            obj_idx: Vec::with_capacity(10),
            window_line_counter: 0,
            wy_cond_triggered: false,
        }
    }

    fn set_ly(&mut self, value: u8) {
        let mut stat = LCDStatus::from(self.memory.borrow().lcd_status);
        self.memory.borrow_mut().ly = value;
        stat.ly_compare = value == self.memory.borrow().lyc;
        if stat.ly_interrupt_enable && stat.ly_compare {
            self.memory.borrow_mut().interrupt_flag |= 1 << 1;
        }
        self.memory.borrow_mut().lcd_status = stat.into();
    }

    fn enter_mode_0(&mut self) {
        let mut stat = LCDStatus::from(self.memory.borrow().lcd_status);
        stat.mode = 0;
        self.memory.borrow_mut().lcd_status = stat.into();
        if stat.hblank_interrupt_enable {
            self.memory.borrow_mut().interrupt_flag |= 1 << 1;
        }
        self.clocks_to_finish = 204;
    }

    fn enter_mode_1(&mut self) {
        let mut stat = LCDStatus::from(self.memory.borrow().lcd_status);
        stat.mode = 1;
        self.memory.borrow_mut().lcd_status = stat.into();
        self.memory.borrow_mut().interrupt_flag |= 1;
        if stat.vblank_interrupt_enable {
            self.memory.borrow_mut().interrupt_flag |= 1 << 1;
        }
        self.clocks_to_finish = 456;
    }

    fn enter_mode_2(&mut self) {
        let mut stat = LCDStatus::from(self.memory.borrow().lcd_status);
        stat.mode = 2;
        self.memory.borrow_mut().lcd_status = stat.into();
        if stat.oam_interrupt_enable {
            self.memory.borrow_mut().interrupt_flag |= 1 << 1;
        }
        self.clocks_to_finish = 80;
    }

    fn enter_mode_3(&mut self) {
        let mut stat = LCDStatus::from(self.memory.borrow().lcd_status);
        stat.mode = 3;
        self.memory.borrow_mut().lcd_status = stat.into();
        self.clocks_to_finish = 172;
    }

    pub fn tick(&mut self) {
        let ly = self.memory.borrow().ly;
        let wy = self.memory.borrow().wy;
        let stat = LCDStatus::from(self.memory.borrow().lcd_status);
        if stat.mode == 2 && self.clocks_to_finish == 80 {
            // WY condition is checked at the start of Mode 2 only.
            self.wy_cond_triggered |= ly == wy;
        }
        self.clocks_to_finish -= 1;
        if self.clocks_to_finish == 0 {
            if stat.mode == 2 {
                // OAM SCAN
                self.oam_scan(ly as usize);
                self.enter_mode_3();
            } else if stat.mode == 3 {
                // DRAWING PIXELS
                self.render(ly as usize);
                self.enter_mode_0();
            } else if stat.mode == 0 {
                // HORIZONTAL BLANK
                if ly < 143 {
                    self.enter_mode_2();
                } else {
                    self.enter_mode_1();
                }
                self.set_ly(ly + 1);
            } else {
                // VERTICAL BLANK
                if ly < 153 {
                    // stay mode 1
                    self.clocks_to_finish = 456;
                    self.set_ly(ly + 1);
                } else {
                    self.enter_mode_2();
                    self.set_ly(0);
                    self.window_line_counter = 0;
                    self.wy_cond_triggered = false;
                }
            }
        }
    }

    pub fn set_pixel(&mut self, screen_y: usize, screen_x: usize, color: Color) {
        const WHITE: u8 = 255;
        const LIGHT_GRAY: u8 = 170;
        const DARK_GRAY: u8 = 85;
        const BLACK: u8 = 0;

        let index = (screen_y * DISPLAY_WIDTH + screen_x) * 4;
        match color {
            Color::White => {
                self.frame_buffer[index] = WHITE;
                self.frame_buffer[index + 1] = WHITE;
                self.frame_buffer[index + 2] = WHITE;
                self.frame_buffer[index + 3] = 255;
            }
            Color::LightGray => {
                self.frame_buffer[index] = LIGHT_GRAY;
                self.frame_buffer[index + 1] = LIGHT_GRAY;
                self.frame_buffer[index + 2] = LIGHT_GRAY;
                self.frame_buffer[index + 3] = 255;
            }
            Color::DarkGray => {
                self.frame_buffer[index] = DARK_GRAY;
                self.frame_buffer[index + 1] = DARK_GRAY;
                self.frame_buffer[index + 2] = DARK_GRAY;
                self.frame_buffer[index + 3] = 255;
            }
            Color::Black => {
                self.frame_buffer[index] = BLACK;
                self.frame_buffer[index + 1] = BLACK;
                self.frame_buffer[index + 2] = BLACK;
                self.frame_buffer[index + 3] = 255;
            }
        }
    }

    pub fn get_background_pixel(&self, screen_y: usize, screen_x: usize) -> Color {
        let lcdc: LCDControl = self.memory.borrow().lcd_control.into();
        if !lcdc.bg_win_enable {
            return Color::White;
        }
        let y = (screen_y + self.memory.borrow().scy as usize) % 256;
        let x = (screen_x + self.memory.borrow().scx as usize) % 256;
        let tile_map_base_addr = if lcdc.bg_tile_map_area {
            0x9c00u16
        } else {
            0x9800u16
        };
        let tile_map_addr = tile_map_base_addr + ((y / 8) * 32 + (x / 8)) as u16;
        let tile_idx = self.memory.borrow().get_byte(tile_map_addr);
        let tile_data_addr = if lcdc.bg_win_tile_data_area {
            0x8000u16.wrapping_add((tile_idx as u16) << 4)
        } else {
            0x9000u16.wrapping_add_signed(((tile_idx as i8) as i16) << 4)
        };
        let y = y % 8;
        let x = x % 8;
        let tile_data = self
            .memory
            .borrow()
            .get_word(tile_data_addr + (y as u16) * 2);
        let color_id = ((tile_data >> (7 - x)) & 1) + (((tile_data >> (15 - x)) & 1) << 1);
        let palette = self.memory.borrow().bg_palette;
        let color: Color = ((palette >> (color_id * 2)) & 0x3).into();
        color
    }

    pub fn get_window_pixel(&self, screen_x: usize) -> Option<Color> {
        let lcdc: LCDControl = self.memory.borrow().lcd_control.into();
        if !lcdc.bg_win_enable || !lcdc.win_enable {
            return None;
        }
        let wx = self.memory.borrow().wx as usize;
        if wx > screen_x + 7 {
            return None;
        }
        let x = screen_x + 7 - wx;
        let tile_map_base_addr = if lcdc.win_tile_map_area {
            0x9c00u16
        } else {
            0x9800u16
        };
        let tile_map_addr =
            tile_map_base_addr + ((self.window_line_counter / 8) * 32 + (x / 8)) as u16;
        let tile_idx = self.memory.borrow().get_byte(tile_map_addr);
        let tile_data_addr = if lcdc.bg_win_tile_data_area {
            0x8000u16.wrapping_add((tile_idx as u16) << 4)
        } else {
            0x9000u16.wrapping_add_signed(((tile_idx as i8) as i16) << 4)
        };
        let y = self.window_line_counter % 8;
        let x = x % 8;
        let tile_data = self
            .memory
            .borrow()
            .get_word(tile_data_addr + (y as u16) * 2);
        let color_id = ((tile_data >> (7 - x)) & 1) + (((tile_data >> (15 - x)) & 1) << 1);
        let palette = self.memory.borrow().bg_palette;
        let color: Color = ((palette >> (color_id * 2)) & 0x3).into();
        Some(color)
    }

    pub fn get_object_pixel(
        &self,
        obj_idx: usize,
        screen_y: usize,
        screen_x: usize,
    ) -> Option<Color> {
        let control = LCDControl::from(self.memory.borrow().lcd_control);
        let addr = 0xfe00 + obj_idx * 4;
        let obj_h = if control.obj_size { 16 } else { 8 };
        let obj_y = self.memory.borrow().get_byte(addr as u16) as usize;
        let obj_x = self.memory.borrow().get_byte((addr + 1) as u16) as usize;
        let mut tile_idx = self.memory.borrow().get_byte((addr + 2) as u16) as usize;
        if control.obj_size {
            tile_idx &= 0xfe;
        }
        let attr = self.memory.borrow().get_byte((addr + 3) as u16) as usize;
        let mut y = screen_y + 16 - obj_y;
        let mut x = screen_x + 8 - obj_x;
        if attr & (1 << 6) != 0 {
            y = obj_h - y - 1;
        }
        if attr & (1 << 5) != 0 {
            x = 7 - x;
        }
        if y >= 8 {
            tile_idx += 1;
            y -= 8;
        }
        let tile_data_addr = 0x8000u16.wrapping_add((tile_idx as u16) << 4);
        let tile_data = self
            .memory
            .borrow()
            .get_word(tile_data_addr + (y as u16) * 2);
        let color_id = ((tile_data >> (7 - x)) & 1) + (((tile_data >> (15 - x)) & 1) << 1);
        if color_id == 0 {
            return None;
        }
        let palette = self.memory.borrow().obj_palette[(attr >> 4) & 1];
        let color: Color = ((palette >> (color_id * 2)) & 0x3).into();
        Some(color)
    }

    pub fn oam_scan(&mut self, y: usize) {
        let control = LCDControl::from(self.memory.borrow().lcd_control);
        self.obj_idx.clear();
        for idx in 0..40 {
            let addr = 0xfe00 + idx * 4;
            let obj_h = if control.obj_size { 16 } else { 8 };
            let obj_y = self.memory.borrow().get_byte(addr) as usize;
            let obj_x = self.memory.borrow().get_byte(addr + 1) as usize;
            if obj_y + obj_h > y + 16 && obj_y <= y + 16 {
                self.obj_idx.push((obj_x, idx as usize));
                if self.obj_idx.len() >= 10 {
                    break;
                }
            }
        }
        self.obj_idx.sort_by(|(x1, _), (x2, _)| x1.cmp(x2));
    }

    pub fn clear_frame_buffer(&mut self) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.set_pixel(y, x, Color::White);
            }
        }
    }

    pub fn render(&mut self, y: usize) {
        let control = LCDControl::from(self.memory.borrow().lcd_control);
        let wx = self.memory.borrow().wx;
        let wx_cond_triggered = wx < 168;
        let is_window_visible = control.bg_win_enable
            && control.win_enable
            && self.wy_cond_triggered
            && wx_cond_triggered;
        for x in 0..DISPLAY_WIDTH {
            let mut bg_pixel = self.get_background_pixel(y, x);
            self.set_pixel(y, x, bg_pixel);
            if is_window_visible {
                if let Some(win_pixel) = self.get_window_pixel(x) {
                    bg_pixel = win_pixel;
                    self.set_pixel(y, x, win_pixel);
                }
            }
            if control.obj_enable {
                let obj_idx = self.obj_idx.clone();
                for (obj_x, idx) in obj_idx {
                    if obj_x > x + 8 || obj_x <= x {
                        continue;
                    }
                    if let Some(obj_pixel) = self.get_object_pixel(idx, y, x) {
                        let addr = 0xfe00 + idx * 4;
                        let attr = self.memory.borrow().get_byte((addr + 3) as u16) as usize;
                        if attr & (1 << 7) == 0 || matches!(bg_pixel, Color::White) {
                            self.set_pixel(y, x, obj_pixel);
                        }
                        break;
                    }
                }
            }
        }
        if is_window_visible {
            self.window_line_counter += 1;
        }
    }
}
