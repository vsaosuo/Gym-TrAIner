use std::{io, mem::size_of};

use memmap2::{MmapOptions, MmapRaw};

use crate::{DevMem, LW_BRIDGE_BASE};

const KEY_BASE: u64 = 0x00000050;

pub type KeysPressed = [bool; 4];

pub struct Keys {
    keys: MmapRaw,
    prev: u8,
}

impl Keys {
    pub fn new(DevMem(mem): &DevMem) -> io::Result<Self> {
        let keys = MmapOptions::new()
            .offset(LW_BRIDGE_BASE + KEY_BASE)
            .len(size_of::<u8>())
            .map_raw(mem)?;

        Ok(Self { keys, prev: 0u8 })
    }

    pub fn read(&mut self) -> KeysPressed {
        let cur = unsafe { self.keys.as_ptr().read_volatile() };
        let delta = !self.prev & cur;
        self.prev = cur;

        [
            (delta & (1 << 0)) != 0,
            (delta & (1 << 1)) != 0,
            (delta & (1 << 2)) != 0,
            (delta & (1 << 3)) != 0,
        ]
    }
}
