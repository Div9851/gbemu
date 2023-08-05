use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

const WAVEFORM: [usize; 4 * 8] = [
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
];

const DIVISOR: [usize; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct APU {
    pub memory: Rc<RefCell<Memory>>,
    pub audio_buffer: Vec<f32>,
    pub sampling_timer: usize,
    pub frame_sequencer_counter: usize,
    pub frame_sequencer_clock_counter: usize,
    pub frequency_timer_1: usize,
    pub wave_duty_position_1: usize,
    pub period_timer_1: usize,
    pub length_timer_1: usize,
    pub sweep_enable_1: bool,
    pub shadow_frequency_1: usize,
    pub sweep_timer_1: usize,
    pub current_volume_1: usize,
    pub frequency_timer_2: usize,
    pub wave_duty_position_2: usize,
    pub period_timer_2: usize,
    pub length_timer_2: usize,
    pub current_volume_2: usize,
    pub sample_index_3: usize,
    pub length_timer_3: usize,
    pub frequency_timer_3: usize,
    pub period_timer_4: usize,
    pub length_timer_4: usize,
    pub current_volume_4: usize,
    pub frequency_timer_4: usize,
    pub lfsr: usize,
}

impl APU {
    pub fn new(memory: Rc<RefCell<Memory>>) -> APU {
        APU {
            memory,
            audio_buffer: Vec::new(),
            sampling_timer: 0,
            frame_sequencer_counter: 0,
            frame_sequencer_clock_counter: 0,
            frequency_timer_1: 0,
            wave_duty_position_1: 0,
            period_timer_1: 0,
            length_timer_1: 0,
            sweep_enable_1: false,
            shadow_frequency_1: 0,
            sweep_timer_1: 0,
            current_volume_1: 0,
            frequency_timer_2: 0,
            wave_duty_position_2: 0,
            period_timer_2: 0,
            length_timer_2: 0,
            current_volume_2: 0,
            sample_index_3: 0,
            length_timer_3: 0,
            frequency_timer_3: 0,
            period_timer_4: 0,
            length_timer_4: 0,
            current_volume_4: 0,
            frequency_timer_4: 0,
            lfsr: 0,
        }
    }

    fn calculate_frequency(&mut self) -> usize {
        let sweep_shift = (self.memory.borrow().nr10 & 0x7) as usize;
        let is_decrementing = (self.memory.borrow().nr10 & (1 << 3)) != 0;
        let mut new_frequency = self.shadow_frequency_1 >> sweep_shift;
        if is_decrementing {
            new_frequency = self.shadow_frequency_1 - new_frequency;
        } else {
            new_frequency = self.shadow_frequency_1 + new_frequency;
        }
        /* overflow check */
        if new_frequency > 2047 {
            self.memory.borrow_mut().nr52 &= 0xfe;
        }
        new_frequency
    }

    pub fn tick(&mut self) {
        if self.memory.borrow().nr14 & (1 << 7) != 0 {
            // restart channel 1
            self.memory.borrow_mut().nr14 ^= 1 << 7;
            self.memory.borrow_mut().nr52 |= 1;
            let nr13 = self.memory.borrow().nr13 as usize;
            let nr14 = self.memory.borrow().nr14 as usize;
            let frequency_1 = nr13 | ((nr14 & 0x7) << 8);
            self.frequency_timer_1 = (2048 - frequency_1) * 4;
            let initial_length_timer_1 = (self.memory.borrow().nr11 & 0x3f) as usize;
            self.length_timer_1 = 64 - initial_length_timer_1;
            let initial_volume_1 = (self.memory.borrow().nr12 >> 4) as usize;
            self.current_volume_1 = initial_volume_1;
            let period_1 = (self.memory.borrow().nr12 & 0x7) as usize;
            self.period_timer_1 = period_1;
            self.shadow_frequency_1 = frequency_1;
            self.sweep_timer_1 = 8;
            let sweep_period = ((self.memory.borrow().nr10 >> 4) & 0x7) as usize;
            let sweep_shift = (self.memory.borrow().nr10 & 0x7) as usize;
            self.sweep_enable_1 = sweep_period != 0 || sweep_shift != 0;
            /* for overflow check */
            if sweep_shift != 0 {
                self.calculate_frequency();
            }
        }
        if self.memory.borrow().nr24 & (1 << 7) != 0 {
            // restart channel 2
            self.memory.borrow_mut().nr24 ^= 1 << 7;
            self.memory.borrow_mut().nr52 |= 1 << 1;
            let nr23 = self.memory.borrow().nr23 as usize;
            let nr24 = self.memory.borrow().nr24 as usize;
            let frequency_2 = nr23 | ((nr24 & 0x7) << 8);
            self.frequency_timer_2 = (2048 - frequency_2) * 4;
            let initial_length_timer_2 = (self.memory.borrow().nr21 & 0x3f) as usize;
            self.length_timer_2 = 64 - initial_length_timer_2;
            let initial_volume_2 = (self.memory.borrow().nr22 >> 4) as usize;
            self.current_volume_2 = initial_volume_2;
            let period_2 = (self.memory.borrow().nr22 & 0x7) as usize;
            self.period_timer_2 = period_2;
        }
        if self.memory.borrow().nr34 & (1 << 7) != 0 {
            // restart channel 3
            self.memory.borrow_mut().nr34 ^= 1 << 7;
            self.memory.borrow_mut().nr52 |= 1 << 2;
            let nr33 = self.memory.borrow().nr33 as usize;
            let nr34 = self.memory.borrow().nr34 as usize;
            let frequency_3 = nr33 | ((nr34 & 0x7) << 8);
            self.frequency_timer_3 = (2048 - frequency_3) * 4;
            let initial_length_timer_3 = self.memory.borrow().nr31 as usize;
            self.length_timer_3 = 256 - initial_length_timer_3;
            self.sample_index_3 = 0;
        }
        if self.memory.borrow().nr44 & (1 << 7) != 0 {
            // restart channel 4
            self.memory.borrow_mut().nr44 ^= 1 << 7;
            self.memory.borrow_mut().nr52 |= 1 << 3;
            let nr43 = self.memory.borrow().nr43 as usize;
            let divisor = DIVISOR[nr43 & 0x7];
            let shift_amount = nr43 >> 4;
            self.frequency_timer_4 = divisor << shift_amount;
            let initial_length_timer_4 = (self.memory.borrow().nr41 & 0x3f) as usize;
            self.length_timer_4 = 64 - initial_length_timer_4;
            let initial_volume_4 = (self.memory.borrow().nr42 >> 4) as usize;
            self.current_volume_4 = initial_volume_4;
            let period_4 = (self.memory.borrow().nr42 & 0x7) as usize;
            self.period_timer_4 = period_4;
            self.lfsr = 0x7fff;
        }
        self.frequency_timer_1 -= 1;
        if self.frequency_timer_1 == 0 {
            self.wave_duty_position_1 = (self.wave_duty_position_1 + 1) % 8;
            let nr13 = self.memory.borrow().nr13 as usize;
            let nr14 = self.memory.borrow().nr14 as usize;
            let frequency_1 = nr13 | ((nr14 & 0x7) << 8);
            self.frequency_timer_1 = (2048 - frequency_1) * 4;
        }
        self.frequency_timer_2 -= 1;
        if self.frequency_timer_2 == 0 {
            self.wave_duty_position_2 = (self.wave_duty_position_2 + 1) % 8;
            let nr23 = self.memory.borrow().nr23 as usize;
            let nr24 = self.memory.borrow().nr24 as usize;
            let frequency_2 = nr23 | ((nr24 & 0x7) << 8);
            self.frequency_timer_2 = (2048 - frequency_2) * 4;
        }
        self.frequency_timer_3 -= 1;
        if self.frequency_timer_3 == 0 {
            self.sample_index_3 = (self.sample_index_3 + 1) % 32;
            let nr33 = self.memory.borrow().nr33 as usize;
            let nr34 = self.memory.borrow().nr34 as usize;
            let frequency_3 = nr33 | ((nr34 & 0x7) << 8);
            self.frequency_timer_3 = (2048 - frequency_3) * 4;
        }
        self.frequency_timer_4 -= 1;
        if self.frequency_timer_4 == 0 {
            let divisor_code = (self.memory.borrow().nr43 & 0x7) as usize;
            let shift_amount = (self.memory.borrow().nr43 >> 4) as usize;
            let width_mode = (self.memory.borrow().nr43 >> 3) & 1;
            self.frequency_timer_4 = (if divisor_code > 0 {
                divisor_code << 4
            } else {
                8
            }) << shift_amount;
            let xor_result = (self.lfsr & 1) ^ ((self.lfsr & 2) >> 1);
            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);
            if width_mode == 1 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }
        }
        self.frame_sequencer_counter += 1;
        if self.frame_sequencer_counter == 8192 {
            self.frame_sequencer_counter = 0;
            if self.frame_sequencer_clock_counter == 7 {
                let period_1 = (self.memory.borrow().nr12 & 0x7) as usize;
                if period_1 != 0 {
                    if self.period_timer_1 > 0 {
                        self.period_timer_1 -= 1;
                    }
                    if self.period_timer_1 == 0 {
                        self.period_timer_1 = period_1;
                        let is_upwards = (self.memory.borrow().nr12 & (1 << 3)) != 0;
                        if self.current_volume_1 < 0xf && is_upwards {
                            self.current_volume_1 += 1;
                        } else if self.current_volume_1 > 0 && !is_upwards {
                            self.current_volume_1 -= 1;
                        }
                    }
                }
                let period_2 = (self.memory.borrow().nr22 & 0x7) as usize;
                if period_2 != 0 {
                    if self.period_timer_2 > 0 {
                        self.period_timer_2 -= 1;
                    }
                    if self.period_timer_2 == 0 {
                        self.period_timer_2 = period_2;
                        let is_upwards = (self.memory.borrow().nr22 & (1 << 3)) != 0;
                        if self.current_volume_2 < 0xf && is_upwards {
                            self.current_volume_2 += 1;
                        } else if self.current_volume_2 > 0 && !is_upwards {
                            self.current_volume_2 -= 1;
                        }
                    }
                }
                let period_4 = (self.memory.borrow().nr42 & 0x7) as usize;
                if period_4 != 0 {
                    if self.period_timer_4 > 0 {
                        self.period_timer_4 -= 1;
                    }
                    if self.period_timer_4 == 0 {
                        self.period_timer_4 = period_4;
                        let is_upwards = (self.memory.borrow().nr42 & (1 << 3)) != 0;
                        if self.current_volume_4 < 0xf && is_upwards {
                            self.current_volume_4 += 1;
                        } else if self.current_volume_4 > 0 && !is_upwards {
                            self.current_volume_4 -= 1;
                        }
                    }
                }
            }
            if self.frame_sequencer_clock_counter & 1 == 0 {
                if self.memory.borrow().nr14 & (1 << 6) != 0 {
                    self.length_timer_1 -= 1;
                    if self.length_timer_1 == 0 {
                        self.memory.borrow_mut().nr52 &= 0xfe;
                    }
                }
                if self.memory.borrow().nr24 & (1 << 6) != 0 {
                    self.length_timer_2 -= 1;
                    if self.length_timer_2 == 0 {
                        self.memory.borrow_mut().nr52 &= 0xfd;
                    }
                }
                if self.memory.borrow().nr34 & (1 << 6) != 0 {
                    self.length_timer_3 -= 1;
                    if self.length_timer_3 == 0 {
                        self.memory.borrow_mut().nr52 &= 0xfb;
                    }
                }
                if self.memory.borrow().nr44 & (1 << 6) != 0 {
                    self.length_timer_4 -= 1;
                    if self.length_timer_4 == 0 {
                        self.memory.borrow_mut().nr52 &= 0xf7;
                    }
                }
            }
            if self.frame_sequencer_clock_counter == 2 || self.frame_sequencer_clock_counter == 6 {
                let sweep_period = ((self.memory.borrow().nr10 >> 4) & 0x7) as usize;
                let sweep_shift = (self.memory.borrow().nr10 & 0x7) as usize;
                if self.sweep_timer_1 > 0 {
                    self.sweep_timer_1 -= 1;
                }
                if self.sweep_timer_1 == 0 {
                    if sweep_period > 0 {
                        self.sweep_timer_1 = sweep_period;
                    } else {
                        self.sweep_timer_1 = 8;
                    }
                    if self.sweep_enable_1 && sweep_period > 0 {
                        let new_frequency = self.calculate_frequency();
                        if new_frequency <= 2047 && sweep_shift > 0 {
                            self.memory.borrow_mut().nr13 = (new_frequency & 0xff) as u8;
                            let nr14 = self.memory.borrow().nr14 as usize;
                            self.memory.borrow_mut().nr14 =
                                ((nr14 & 0xf8) | (new_frequency >> 8)) as u8;
                            self.shadow_frequency_1 = new_frequency;
                            self.calculate_frequency();
                        }
                    }
                }
            }
            self.frame_sequencer_clock_counter = (self.frame_sequencer_clock_counter + 1) & 0x7;
        }
        self.sampling_timer += 1;
        if self.sampling_timer == 87 {
            self.sampling_timer = 0;
            let mut amplitude = 0.0;
            if self.memory.borrow().nr52 & 1 != 0 {
                let wave_duty_pattern = (self.memory.borrow().nr11 >> 6) as usize;
                let dac_input = WAVEFORM[wave_duty_pattern * 8 + self.wave_duty_position_1]
                    * self.current_volume_1;
                amplitude += (dac_input as f32) / 7.5 - 1.0;
            }
            if self.memory.borrow().nr52 & (1 << 1) != 0 {
                let wave_duty_pattern = (self.memory.borrow().nr21 >> 6) as usize;
                let dac_input = WAVEFORM[wave_duty_pattern * 8 + self.wave_duty_position_2]
                    * self.current_volume_2;
                amplitude += (dac_input as f32) / 7.5 - 1.0;
            }
            if self.memory.borrow().nr52 & (1 << 2) != 0 {
                let sample_index = self.sample_index_3;
                let mut dac_input = self
                    .memory
                    .borrow()
                    .get_byte((0xff30 + sample_index / 2) as u16);
                if sample_index & 1 == 0 {
                    dac_input = (dac_input >> 4) & 0xf;
                } else {
                    dac_input = dac_input & 0xf;
                }
                let volume = (self.memory.borrow().nr32 >> 5) & 0x3;
                if volume == 0 {
                    dac_input >>= 4;
                } else if volume == 2 {
                    dac_input >>= 1;
                } else if volume == 3 {
                    dac_input >>= 2;
                }
                amplitude += (dac_input as f32) / 7.5 - 1.0;
            }
            if self.memory.borrow().nr52 & (1 << 3) != 0 {
                let dac_input = ((self.lfsr ^ 0x7fff) & 0x1) * self.current_volume_4;
                amplitude += (dac_input as f32) / 7.5 - 1.0;
            }
            amplitude /= 4.0;
            self.audio_buffer.push(amplitude);
        }
    }

    pub fn clear_audio_buffer(&mut self) {
        self.audio_buffer.clear()
    }
}
