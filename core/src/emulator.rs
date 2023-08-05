extern crate console_error_panic_hook;
use crate::apu::APU;
use crate::cpu::{Flags, CPU};
use crate::memory::Memory;
use crate::ppu::PPU;
use crate::timer::Timer;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const CLOCKS_PER_FRAME: usize = 70224;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default)]
pub struct JoypadInput {
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

#[wasm_bindgen]
impl JoypadInput {
    pub fn new(
        start: bool,
        select: bool,
        a: bool,
        b: bool,
        down: bool,
        up: bool,
        left: bool,
        right: bool,
    ) -> JoypadInput {
        JoypadInput {
            start,
            select,
            a,
            b,
            down,
            up,
            left,
            right,
        }
    }
}

#[wasm_bindgen]
pub struct Emulator {
    cpu: CPU,
    ppu: PPU,
    apu: APU,
    timer: Timer,
    memory: Rc<RefCell<Memory>>,
    joypad_input: JoypadInput,
    pub running: bool,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Emulator {
        let memory = Rc::new(RefCell::new(Memory::new()));
        Emulator {
            cpu: CPU::new(Rc::clone(&memory)),
            ppu: PPU::new(Rc::clone(&memory)),
            apu: APU::new(Rc::clone(&memory)),
            timer: Timer::new(Rc::clone(&memory)),
            memory,
            joypad_input: JoypadInput::default(),
            running: false,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    // set the emulator state as if the boot ROM has been executed.
    // see https://gbdev.io/pandocs/Power_Up_Sequence.html#cpu-registers
    pub fn init(&mut self) {
        self.cpu.registers.a = 0x00;
        self.cpu.registers.f = Flags {
            z: true,
            n: false,
            h: false,
            c: false,
        }
        .into();
        self.cpu.registers.b = 0x00;
        self.cpu.registers.c = 0x13;
        self.cpu.registers.d = 0x00;
        self.cpu.registers.e = 0xd8;
        self.cpu.registers.h = 0x01;
        self.cpu.registers.l = 0x4d;
        self.cpu.registers.pc = 0x100;
        self.cpu.registers.sp = 0xfffe;

        self.ppu.clocks_to_finish = 456;

        let mut memory = self.memory.borrow_mut();

        memory.rom_bank_number = 1;
        memory.joypad = 0xcf;
        memory.divider = 0xab;
        memory.timer = 0x00;
        memory.timer_modulo = 0x00;
        memory.timer_control = 0xf8;
        memory.interrupt_flag = 0xe1;
        memory.nr21 = 0x3f;
        memory.nr22 = 0x00;
        memory.nr23 = 0xff;
        memory.nr24 = 0xbf;
        memory.nr52 = 0xf1;
        memory.lcd_control = 0x91;
        memory.lcd_status = 0x85;
        memory.scy = 0x00;
        memory.scx = 0x00;
        memory.ly = 0x00;
        memory.lyc = 0x00;
        memory.bg_palette = 0xfc;
        // obj palettes are left entirely uninitialized.
        memory.wy = 0x00;
        memory.wx = 0x00;
    }

    pub fn load_rom(&mut self, rom_data: &[u8]) {
        console_error_panic_hook::set_once();
        let n = rom_data.len();
        self.memory.borrow_mut().cart_rom[0..n].copy_from_slice(rom_data);
        self.memory.borrow_mut().cart_type = rom_data[0x147];
        self.memory.borrow_mut().rom_size = rom_data[0x148];
        self.memory.borrow_mut().ram_size = rom_data[0x149];
    }

    pub fn load_savedata(&mut self, savedata: &[u8]) {
        console_error_panic_hook::set_once();
        let n = savedata.len();
        self.memory.borrow_mut().cart_ram[0..n].copy_from_slice(savedata);
    }

    pub fn get_savedata(&self) -> Vec<u8> {
        self.memory.borrow().cart_ram[..].into()
    }

    pub fn tick(&mut self) {
        self.update_joypad();
        self.timer.tick();
        self.ppu.tick();
        self.apu.tick();
        self.cpu.tick();
    }

    pub fn next_frame(&mut self) {
        console_error_panic_hook::set_once();
        self.ppu.clear_frame_buffer();
        self.apu.clear_audio_buffer();
        for _ in 0..CLOCKS_PER_FRAME {
            self.tick();
        }
    }

    pub fn get_frame_buffer(&self) -> Vec<u8> {
        self.ppu.frame_buffer.into()
    }

    pub fn get_audio_buffer(&self) -> Vec<f32> {
        self.apu.audio_buffer.clone()
    }

    pub fn update_joypad_input(&mut self, joypad_input: JoypadInput) {
        self.joypad_input = joypad_input;
    }

    pub fn update_joypad(&mut self) {
        let joypad = self.memory.borrow().joypad;
        let mut next_joypad = joypad | 0xf;
        if joypad & (1 << 5) == 0 {
            // button keys
            if self.joypad_input.start {
                next_joypad ^= 1 << 3;
            }
            if self.joypad_input.select {
                next_joypad ^= 1 << 2;
            }
            if self.joypad_input.b {
                next_joypad ^= 1 << 1;
            }
            if self.joypad_input.a {
                next_joypad ^= 1;
            }
        } else if joypad & (1 << 4) == 0 {
            // direction keys
            if self.joypad_input.down {
                next_joypad ^= 1 << 3;
            }
            if self.joypad_input.up {
                next_joypad ^= 1 << 2;
            }
            if self.joypad_input.left {
                next_joypad ^= 1 << 1;
            }
            if self.joypad_input.right {
                next_joypad ^= 1;
            }
        }
        for i in 0..4 {
            if ((joypad >> i) & 1 != 0) && ((next_joypad >> i) & 1 == 0) {
                self.memory.borrow_mut().interrupt_flag |= 1 << 4;
            }
        }
        self.memory.borrow_mut().joypad = next_joypad;
    }
}
