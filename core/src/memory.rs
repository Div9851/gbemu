use core::panic;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct Memory {
    pub cart_rom: [u8; 32 * 1024],
    pub video_ram: [u8; 8 * 1024],
    pub cart_ram: [u8; 8 * 1024],
    pub work_ram: [u8; 8 * 1024],
    pub obj_attr_memory: [u8; 160],
    pub joypad: u8,
    pub divider: u8,
    pub timer: u8,
    pub timer_modulo: u8,
    pub timer_control: u8,
    pub lcd_control: u8,
    pub lcd_status: u8,
    pub scy: u8,
    pub scx: u8,
    pub ly: u8,
    pub lyc: u8,
    pub bg_palette: u8,
    pub obj_palette: [u8; 2],
    pub wy: u8,
    pub wx: u8,
    pub high_ram: [u8; 127],
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,
    pub interrupt_master_enable: bool,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            cart_rom: [0; 32 * 1024],
            video_ram: [0; 8 * 1024],
            cart_ram: [0; 8 * 1024],
            work_ram: [0; 8 * 1024],
            obj_attr_memory: [0; 160],
            joypad: 0,
            divider: 0,
            timer: 0,
            timer_modulo: 0,
            timer_control: 0,
            lcd_control: 0,
            lcd_status: 0,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            ly: 0,
            lyc: 0,
            bg_palette: 0,
            obj_palette: [0; 2],
            high_ram: [0; 127],
            interrupt_flag: 0,
            interrupt_enable: 0,
            interrupt_master_enable: false,
        }
    }

    pub fn get_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            0..=0x7fff => self.cart_rom[address],
            0x8000..=0x9fff => self.video_ram[address - 0x8000],
            0xa000..=0xbfff => self.cart_ram[address - 0xa000],
            0xc000..=0xdfff => self.work_ram[address - 0xc000],
            0xfe00..=0xfe9f => self.obj_attr_memory[address - 0xfe00],
            0xff00 => self.joypad,
            0xff04 => self.divider,
            0xff05 => self.timer,
            0xff06 => self.timer_modulo,
            0xff07 => self.timer_control,
            0xff0f => self.interrupt_flag,
            0xff40 => self.lcd_control,
            0xff41 => self.lcd_status,
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bg_palette,
            0xff48..=0xff49 => self.obj_palette[address - 0xff48],
            0xff4a => self.wy,
            0xff4b => self.wx,
            0xff80..=0xfffe => self.high_ram[address - 0xff80],
            0xffff => self.interrupt_enable,
            _ => 0xff, // other areas are not supported yet
        }
    }

    pub fn get_word(&self, address: u16) -> u16 {
        let lower = self.get_byte(address) as u16;
        let upper = self.get_byte(address + 1) as u16;
        (upper << 8) | lower
    }

    pub fn set_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            0..=0x7fff => self.cart_rom[address] = value,
            0x8000..=0x9fff => self.video_ram[address - 0x8000] = value,
            0xa000..=0xbfff => self.cart_ram[address - 0xa000] = value,
            0xc000..=0xdfff => self.work_ram[address - 0xc000] = value,
            0xfe00..=0xfe9f => self.obj_attr_memory[address - 0xfe00] = value,
            0xff00 => self.joypad = (value & 0x30) | (self.joypad & 0xf),
            0xff01 => log(format!("{} '{}'", value, value as char).as_str()),
            0xff04 => self.divider = 0x00, // Writing any value to this register resets it to 0x00.
            0xff05 => self.timer = value,
            0xff06 => self.timer_modulo = value,
            0xff07 => self.timer_control = value,
            0xff0f => self.interrupt_flag = value,
            0xff40 => self.lcd_control = value,
            0xff41 => self.lcd_status = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            // since ly is read only, it is omitted
            0xff45 => self.lyc = value,
            0xff46 => {
                // writing to this register starts a DMA transfer from ROM or RAM to OAM.
                for i in 0..0x9f {
                    let src = ((value as u16) << 8) | i;
                    let dst = 0xfe00 | i;
                    let byte = self.get_byte(src);
                    self.set_byte(dst, byte);
                }
            }
            0xff47 => self.bg_palette = value,
            0xff48..=0xff49 => self.obj_palette[address - 0xff48] = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            0xff80..=0xfffe => self.high_ram[address - 0xff80] = value,
            0xffff => self.interrupt_enable = value,
            _ => (), // other areas are not supported yet
        }
    }

    pub fn set_word(&mut self, address: u16, value: u16) {
        let lower = (value & 0xff) as u8;
        let upper = (value >> 8) as u8;
        self.set_byte(address, lower);
        self.set_byte(address + 1, upper);
    }
}
