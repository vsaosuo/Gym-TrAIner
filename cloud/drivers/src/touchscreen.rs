use std::{intrinsics::size_of, io};

use memmap2::{MmapOptions, MmapRaw};
use voladdress::{Safe, VolAddress};

use crate::{DevMem, LW_BRIDGE_BASE};

const TOUCHSCREEN_BASE: u64 = LW_BRIDGE_BASE + 0x1020;
const TOUCHSCREEN_SPAN: usize = size_of::<u16>() * 3;

pub struct TouchScreen {
    data: MmapRaw,
}

struct TouchRegs(usize);

impl TouchRegs {
    #[inline]
    unsafe fn new(base: *const u8) -> Self {
        Self(base as usize)
    }

    #[inline]
    fn data(&self) -> VolAddress<u32, Safe, ()> {
        unsafe { VolAddress::new(self.0) }
    }

    // #[inline]
    // fn rxdata(&self) -> VolAddress<u16, Safe, ()> {
    //     unsafe { VolAddress::new(self.0) }
    // }

    // #[inline]
    // #[allow(dead_code)]
    // fn txdata(&self) -> VolAddress<u16, (), Safe> {
    //     unsafe { VolAddress::new(self.0 + size_of::<u16>()) }
    // }

    // #[inline]
    // fn status(&self) -> VolAddress<u16, Safe, Safe> {
    //     unsafe { VolAddress::new(self.0 + size_of::<u16>() * 2) }
    // }
}

const RRDY: u32 = 1 << 15;

#[derive(Debug)]
pub struct TouchEvent {
    pub x: usize,
    pub y: usize,
    pub pen_state: PenState,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PenState {
    Up,
    Down,
}

impl TouchScreen {
    pub fn new(DevMem(mem): &DevMem) -> io::Result<Self> {
        let data = MmapOptions::new()
            .offset(TOUCHSCREEN_BASE)
            .len(TOUCHSCREEN_SPAN)
            .map_raw(mem)?;

        Ok(Self { data })
    }

    // instead of checking raw touches, provide a way to tell when a touch falls within a rectangle
    // linear search because we only have a few
    pub async fn wait_touch(&self, areas: &[TouchArea]) -> usize {
        // flush to start with, in case there were other events
        self.flush().await;

        loop {
            // wait for pen down followed by pen up
            // both have to be inside an area
            let (x_down, y_down) = loop {
                let TouchEvent { x, y, pen_state } = self.read().await;
                if pen_state == PenState::Down {
                    println!("Press at ({x}, {y})");
                    break (x, y);
                }
            };

            let (x_up, y_up) = loop {
                let TouchEvent { x, y, pen_state } = self.read().await;
                if pen_state == PenState::Up {
                    println!("Release at ({x}, {y})");
                    break (x, y);
                }
            };

            if let Some(i) = areas
                .iter()
                .position(|a| a.contains(x_down, y_down) && a.contains(x_up, y_up))
            {
                return i;
            }
        }
    }

    pub async fn flush(&self) {
        let regs = unsafe { TouchRegs::new(self.data.as_ptr()) };

        // just read until not ready
        while regs.data().read() & RRDY != 0 {
            tokio::task::yield_now().await;
        }
    }

    pub async fn read(&self) -> TouchEvent {
        let [pen, x_lo, x_hi, y_lo, y_hi] = self.read_event_bytes().await;

        // this converts the coordinates to 0-1023
        let x = (x_hi as usize) << 7 | (x_lo as usize);
        let y = (y_hi as usize) << 7 | (y_lo as usize);
        let pen_state = if pen & 1 == 1 {
            PenState::Down
        } else {
            PenState::Up
        };

        TouchEvent { x, y, pen_state }
    }

    async fn read_event_bytes(&self) -> [u8; 5] {
        // bitmasks for checking whether byte is valid
        // we also expect the lower 2 bits to be 0's for x and y
        const MASKS: [(u8, u8); 5] = [
            (0b1000_0000, 0b1000_0000),
            (0b0000_0000, 0b1000_0000),
            (0b0000_0000, 0b1110_0000),
            (0b0000_0000, 0b1000_0000),
            (0b0000_0000, 0b1110_0000),
        ];
        let mut data = [0u8; 5];

        // keep reading until the data matches the format
        'outer: loop {
            for (data, (magic, mask)) in data.iter_mut().zip(MASKS) {
                let byte = self.read_byte().await;

                if (byte ^ magic) & mask != 0 {
                    continue 'outer;
                }

                *data = byte;
            }

            break;
        }

        data
    }

    async fn read_byte(&self) -> u8 {
        let regs = unsafe { TouchRegs::new(self.data.as_ptr()) };

        // poll status
        let mut data;
        loop {
            data = regs.data().read();
            if data & RRDY != 0 {
                break;
            }
            // hope this works fine
            tokio::task::yield_now().await;
        }

        (data & 0xFF) as u8
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TouchArea {
    x1: usize,
    x2: usize,
    y1: usize,
    y2: usize,
}

pub const TOUCHSCREEN_WIDTH: usize = 4096;
pub const TOUCHSCREEN_HEIGHT: usize = 4096;

impl TouchArea {
    // since we don't need anything more complicated, this takes vga coordinates and converts them to touchscreen coordinates
    pub const fn new((x1, y1): (usize, usize), (x2, y2): (usize, usize)) -> Self {
        assert!(x1 <= x2);
        assert!(x2 < TOUCHSCREEN_WIDTH);
        assert!(y1 <= y2);
        assert!(y2 < TOUCHSCREEN_HEIGHT);

        Self { x1, x2, y1, y2 }
    }

    // internal
    fn contains(&self, x: usize, y: usize) -> bool {
        let Self { x1, x2, y1, y2 } = *self;

        (x1..=x2).contains(&x) && (y1..=y2).contains(&y)
    }
}
