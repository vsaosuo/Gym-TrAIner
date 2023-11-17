use std::io;

use memmap2::{MmapOptions, MmapRaw};

use crate::{DevMem, LW_BRIDGE_BASE};

const HEX_BASE: u64 = LW_BRIDGE_BASE + 0x20;
const HEX_SPAN: usize = 0x20;
const HEX_HIGH_OFFSET: usize = 0x10;

pub struct HexDisplay {
    hex: MmapRaw,
}

impl HexDisplay {
    pub fn new(DevMem(mem): &DevMem) -> io::Result<Self> {
        let hex = MmapOptions::new()
            .offset(HEX_BASE)
            .len(HEX_SPAN)
            .map_raw(mem)?;

        let mut res = Self { hex };

        // clear the display upon opening
        res.clear();

        Ok(res)
    }

    pub fn write(&mut self, data: [u8; 6]) {
        unsafe {
            (self.hex.as_mut_ptr() as *mut u32)
                .write_volatile(u32::from_be_bytes([data[2], data[3], data[4], data[5]]));
            (self.hex.as_mut_ptr().add(HEX_HIGH_OFFSET) as *mut u32)
                .write_volatile(u32::from_be_bytes([0, 0, data[0], data[1]]));
        }
    }

    pub fn clear(&mut self) {
        self.write([0; 6]);
    }

    pub fn digit_to_hex(digit: u8) -> u8 {
        match digit {
            0 => 0x3f,
            1 => 0x06,
            2 => 0x5b,
            3 => 0x4f,
            4 => 0x66,
            5 => 0x6d,
            6 => 0x7d,
            7 => 0x07,
            8 => 0x7f,
            9 => 0x6f,
            _ => panic!("Invalid digit"),
        }
    }
}

impl Drop for HexDisplay {
    fn drop(&mut self) {
        // also clear the display when dropping
        self.write([0; 6]);
    }
}
