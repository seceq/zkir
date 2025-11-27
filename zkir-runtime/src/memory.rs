//! Memory subsystem

use std::collections::HashMap;
use crate::state::HaltReason;

#[derive(Debug, Clone)]
pub struct Memory {
    data: HashMap<u32, u32>,
    tracing: bool,
}

impl Memory {
    pub fn new(_stack_size: u32, _heap_size: u32) -> Self {
        Memory {
            data: HashMap::new(),
            tracing: false,
        }
    }

    pub fn enable_tracing(&mut self, enabled: bool) {
        self.tracing = enabled;
    }

    pub fn load_word(&mut self, addr: u32, _cycle: u64) -> Result<u32, HaltReason> {
        if addr & 3 != 0 {
            return Err(HaltReason::MemoryError {
                address: addr,
                msg: "Unaligned word access".into(),
            });
        }
        Ok(self.data.get(&addr).copied().unwrap_or(0))
    }

    pub fn store_word(&mut self, addr: u32, value: u32, _cycle: u64) -> Result<(), HaltReason> {
        if addr & 3 != 0 {
            return Err(HaltReason::MemoryError {
                address: addr,
                msg: "Unaligned word access".into(),
            });
        }
        if value == 0 {
            self.data.remove(&addr);
        } else {
            self.data.insert(addr, value);
        }
        Ok(())
    }

    pub fn load_half(&mut self, addr: u32, _cycle: u64) -> Result<u32, HaltReason> {
        if addr & 1 != 0 {
            return Err(HaltReason::MemoryError {
                address: addr,
                msg: "Unaligned half access".into(),
            });
        }
        let word_addr = addr & !3;
        let word = self.data.get(&word_addr).copied().unwrap_or(0);
        let shift = ((addr >> 1) & 1) * 16;
        Ok((word >> shift) & 0xFFFF)
    }

    pub fn load_half_signed(&mut self, addr: u32, cycle: u64) -> Result<u32, HaltReason> {
        let value = self.load_half(addr, cycle)?;
        Ok(((value as i16) as i32) as u32)
    }

    pub fn store_half(&mut self, addr: u32, value: u32, _cycle: u64) -> Result<(), HaltReason> {
        if addr & 1 != 0 {
            return Err(HaltReason::MemoryError {
                address: addr,
                msg: "Unaligned half access".into(),
            });
        }
        let word_addr = addr & !3;
        let shift = ((addr >> 1) & 1) * 16;
        let mask = !(0xFFFFu32 << shift);
        let old = self.data.get(&word_addr).copied().unwrap_or(0);
        let new = (old & mask) | ((value & 0xFFFF) << shift);
        if new == 0 {
            self.data.remove(&word_addr);
        } else {
            self.data.insert(word_addr, new);
        }
        Ok(())
    }

    pub fn load_byte(&mut self, addr: u32, _cycle: u64) -> Result<u32, HaltReason> {
        let word_addr = addr & !3;
        let word = self.data.get(&word_addr).copied().unwrap_or(0);
        let shift = (addr & 3) * 8;
        Ok((word >> shift) & 0xFF)
    }

    pub fn load_byte_signed(&mut self, addr: u32, cycle: u64) -> Result<u32, HaltReason> {
        let value = self.load_byte(addr, cycle)?;
        Ok(((value as i8) as i32) as u32)
    }

    pub fn store_byte(&mut self, addr: u32, value: u32, _cycle: u64) -> Result<(), HaltReason> {
        let word_addr = addr & !3;
        let shift = (addr & 3) * 8;
        let mask = !(0xFFu32 << shift);
        let old = self.data.get(&word_addr).copied().unwrap_or(0);
        let new = (old & mask) | ((value & 0xFF) << shift);
        if new == 0 {
            self.data.remove(&word_addr);
        } else {
            self.data.insert(word_addr, new);
        }
        Ok(())
    }

    pub fn load_code(&mut self, code: &[u32]) {
        for (i, &word) in code.iter().enumerate() {
            let addr = zkir_spec::CODE_BASE + (i as u32 * 4);
            if word != 0 {
                self.data.insert(addr, word);
            }
        }
    }

    pub fn load_data(&mut self, data: &[u8]) {
        for (i, chunk) in data.chunks(4).enumerate() {
            let addr = zkir_spec::DATA_BASE + (i as u32 * 4);
            let mut word = 0u32;
            for (j, &byte) in chunk.iter().enumerate() {
                word |= (byte as u32) << (j * 8);
            }
            if word != 0 {
                self.data.insert(addr, word);
            }
        }
    }
}
