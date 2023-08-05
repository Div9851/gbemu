use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct Memory {
    pub cart_rom: [u8; 8 * 1024 * 1024], // support up to 8MB rom
    pub cart_type: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub rom_bank_number: usize,
    pub ram_bank_number: usize,
    pub banking_mode: bool,
    pub video_ram: [u8; 8 * 1024],
    pub cart_ram: [u8; 32 * 1024],
    pub work_ram: [u8; 8 * 1024],
    pub obj_attr_memory: [u8; 160],
    pub joypad: u8,
    pub divider: u8,
    pub timer: u8,
    pub timer_modulo: u8,
    pub timer_control: u8,
    pub nr10: u8,
    pub nr11: u8,
    pub nr12: u8,
    pub nr13: u8,
    pub nr14: u8,
    pub nr21: u8,
    pub nr22: u8,
    pub nr23: u8,
    pub nr24: u8,
    pub nr30: u8,
    pub nr31: u8,
    pub nr32: u8,
    pub nr33: u8,
    pub nr34: u8,
    pub nr41: u8,
    pub nr42: u8,
    pub nr43: u8,
    pub nr44: u8,
    pub nr50: u8,
    pub nr52: u8,
    pub wave_ram: [u8; 16],
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
            cart_rom: [0; 8 * 1024 * 1024],
            cart_type: 0,
            rom_size: 0,
            ram_size: 0,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: false,
            video_ram: [0; 8 * 1024],
            cart_ram: [0; 32 * 1024],
            work_ram: [0; 8 * 1024],
            obj_attr_memory: [0; 160],
            joypad: 0,
            divider: 0,
            timer: 0,
            timer_modulo: 0,
            timer_control: 0,
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            nr50: 0,
            nr52: 0,
            wave_ram: [0; 16],
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
        let rom_bank_number = self.rom_bank_number;
        let ram_bank_number = self.ram_bank_number;
        match address {
            0x0000..=0x3fff => self.cart_rom[address],
            0x4000..=0x7fff => self.cart_rom[rom_bank_number * 0x4000 + address - 0x4000],
            0x8000..=0x9fff => self.video_ram[address - 0x8000],
            0xa000..=0xbfff => self.cart_ram[ram_bank_number * 0x2000 + address - 0xa000],
            0xc000..=0xdfff => self.work_ram[address - 0xc000],
            0xfe00..=0xfe9f => self.obj_attr_memory[address - 0xfe00],
            0xff00 => 0xc0 | self.joypad,
            0xff04 => self.divider,
            0xff05 => self.timer,
            0xff06 => self.timer_modulo,
            0xff07 => self.timer_control,
            0xff0f => self.interrupt_flag,
            0xff10 => self.nr10 | 0x80,
            0xff11 => self.nr11 | 0x3f,
            0xff12 => self.nr12,
            0xff13 => 0xff, // write only
            0xff14 => self.nr14 | 0xbf,
            0xff16 => self.nr21 | 0x3f,
            0xff17 => self.nr22,
            0xff18 => 0xff, // write only
            0xff19 => self.nr24 | 0xbf,
            0xff1a => self.nr30 | 0x7f,
            0xff1b => self.nr31,
            0xff1c => self.nr32 | 0x9f,
            0xff1d => 0xff, // write only
            0xff1e => self.nr34 | 0xbf,
            0xff20 => 0xff, // write only
            0xff21 => self.nr42,
            0xff22 => self.nr43,
            0xff23 => self.nr44 | 0xbf,
            0xff24 => self.nr50,
            0xff26 => self.nr52 | 0x70,
            0xff30..=0xff3f => self.wave_ram[address - 0xff30],
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
        let ram_bank_number = self.ram_bank_number;
        if address <= 0x7fff {
            if 0x1 <= self.cart_type && self.cart_type <= 0x3 {
                // MBC1
                match address {
                    0x2000..=0x3fff => {
                        let mask = match self.rom_size {
                            0x00 => 0x1,
                            0x01 => 0x3,
                            0x02 => 0x7,
                            0x03 => 0xf,
                            _ => 0x1f,
                        };
                        let prev = self.rom_bank_number;
                        self.rom_bank_number = if (value & 0x1f) == 0 {
                            1
                        } else {
                            (value & mask) as usize
                        };
                        self.rom_bank_number |= prev & 0x60;
                    }
                    0x4000..=0x5fff => {
                        if self.banking_mode {
                            self.ram_bank_number = (value & 0x3) as usize;
                        } else {
                            self.rom_bank_number |= ((value & 0x3) << 5) as usize;
                        }
                    }
                    0x6000..=0x7fff => {
                        if value == 0 {
                            // ROM bank mode (max 8KB RAM, 2MB ROM)
                            self.banking_mode = false;
                            self.ram_bank_number = 0;
                        } else {
                            // RAM bank mode (max 32KB RAM, 512KB ROM)
                            self.banking_mode = true;
                            self.rom_bank_number &= 0x1f;
                        }
                    }
                    _ => {}
                }
            } else if 0x19 <= self.cart_type && self.cart_type <= 0x1b {
                // MBC5
                match address {
                    0x2000..=0x2fff => {
                        let prev = self.rom_bank_number;
                        self.rom_bank_number = value as usize;
                        self.rom_bank_number |= prev & 0x100;
                    }
                    0x3000..=0x3fff => {
                        self.rom_bank_number |= ((value & 1) as usize) << 9;
                    }
                    0x4000..=0x5fff => {
                        self.ram_bank_number = (value as usize) & 0xf;
                    }
                    _ => {}
                }
            }
            return;
        }
        match address {
            0x8000..=0x9fff => self.video_ram[address - 0x8000] = value,
            0xa000..=0xbfff => self.cart_ram[ram_bank_number * 0x2000 + address - 0xa000] = value,
            0xc000..=0xdfff => self.work_ram[address - 0xc000] = value,
            0xfe00..=0xfe9f => self.obj_attr_memory[address - 0xfe00] = value,
            0xff00 => self.joypad = 0xc0 | (value & 0x30) | (self.joypad & 0xf),
            0xff01 => log(format!("{} '{}'", value, value as char).as_str()),
            0xff04 => self.divider = 0x00, // Writing any value to this register resets it to 0x00.
            0xff05 => self.timer = value,
            0xff06 => self.timer_modulo = value,
            0xff07 => self.timer_control = value,
            0xff0f => self.interrupt_flag = value,
            0xff10 => self.nr10 = value | 0x80,
            0xff11 => self.nr11 = value,
            0xff12 => self.nr12 = value,
            0xff13 => self.nr13 = value,
            0xff14 => self.nr14 = value | 0x38,
            0xff16 => self.nr21 = value,
            0xff17 => self.nr22 = value,
            0xff18 => self.nr23 = value,
            0xff19 => self.nr24 = value | 0x38,
            0xff1a => self.nr30 = value | 0x7f,
            0xff1b => self.nr31 = value,
            0xff1c => self.nr32 = value | 0x9f,
            0xff1d => self.nr33 = value,
            0xff1e => self.nr34 = value | 0x38,
            0xff20 => self.nr41 = value | 0xc0,
            0xff21 => self.nr42 = value,
            0xff22 => self.nr43 = value,
            0xff23 => self.nr44 = value | 0x3f,
            0xff24 => self.nr50 = value,
            0xff26 => {
                let prev = self.nr52;
                self.nr52 = (value & (1 << 7)) | 0x70 | (prev & 0xf);
            }
            0xff30..=0xff3f => self.wave_ram[address - 0xff30] = value,
            0xff40 => self.lcd_control = value,
            0xff41 => self.lcd_status = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            // since ly is read only, it is omitted
            0xff45 => self.lyc = value,
            0xff46 => {
                // writing to this register starts a DMA transfer from ROM or RAM to OAM.
                for i in 0..=0x9f {
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
