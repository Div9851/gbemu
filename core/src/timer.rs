use crate::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

// TIMA is incremented at the clock frequency specified by the TAC register.
// DIV is incremented at a rate of 16384 Hz (= CPU Clock / 256)
pub struct Timer {
    pub memory: Rc<RefCell<Memory>>,
    pub timer_counter: usize,
    pub divider_counter: usize,
}

impl Timer {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Timer {
        Timer {
            memory,
            timer_counter: 0,
            divider_counter: 0,
        }
    }
    pub fn tick(&mut self) {
        let mut memory = self.memory.borrow_mut();
        self.divider_counter += 1;
        if self.divider_counter == 256 {
            let value = memory.divider.wrapping_add(1);
            memory.divider = value;
            self.divider_counter = 0;
        }
        if memory.timer_control & (1 << 2) != 0 {
            let mode = memory.timer_control & 3;
            let target = match mode {
                0 => 1024, // 4096 Hz (= CPU Clock / 1024)
                1 => 16,   // 262144 Hz (= CPU Clock / 16)
                2 => 64,   // 65536 Hz (= CPU Clock / 64)
                3 => 256,  // 16384 Hz (=CPU Clock / 256)
                _ => unreachable!(),
            };
            self.timer_counter += 1;
            if self.timer_counter == target {
                let (value, overflow) = memory.timer.overflowing_add(1);
                if overflow {
                    memory.timer = memory.timer_modulo;
                    memory.interrupt_flag |= 1 << 2;
                } else {
                    memory.timer = value;
                }
                self.timer_counter = 0;
            }
        }
    }
}
