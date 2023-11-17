use std::{
    io,
    mem::{self, size_of},
    time::Duration,
};

use common_types::{IMAGE_HEIGHT, IMAGE_WIDTH};
use itertools::Itertools;
use memmap2::{MmapOptions, MmapRaw};
use voladdress::{Safe, VolAddress};

use crate::{DevMem, Texture, LW_BRIDGE_BASE};

pub struct VgaDisplay {
    control: MmapRaw,
    buffer1: MmapRaw,
    buffer2: MmapRaw,
    char_buf: MmapRaw,
    is_first: bool,
}

const PIXEL_BUF_CTRL_BASE: u64 = LW_BRIDGE_BASE + 0x00003020;
const PIXEL_BUF_CTRL_SPAN: u64 = 0x00000010;

const PIXEL_BUF1_BASE: u64 = 0xC8000000;
const PIXEL_BUF2_BASE: u64 = 0xC0000000;
const PIXEL_BUF_SPAN: u64 = 0x0003FFFF;

const CHAR_BUF_BASE: u64 = 0xC9000000;
const CHAR_BUF_SPAN: u64 = 0x00001FFF;

const PIXEL_BUF_WIDTH: usize = IMAGE_WIDTH;
const PIXEL_BUF_HEIGHT: usize = IMAGE_HEIGHT;

const CHAR_BUF_WIDTH: usize = 80;
const CHAR_BUF_HEIGHT: usize = 60;

const DISPLAY_ENABLE: u32 = 1 << 2;
const STATUS_FLAG: u32 = 1;

type Color = u16;

struct VideoRegisters(usize);

impl VideoRegisters {
    #[inline]
    unsafe fn new(base: *const u8) -> Self {
        Self(base as usize)
    }

    #[inline]
    fn buffer(&self) -> VolAddress<u32, Safe, Safe> {
        unsafe { VolAddress::new(self.0) }
    }

    #[inline]
    fn back_buffer(&self) -> VolAddress<u32, Safe, Safe> {
        unsafe { VolAddress::new(self.0 + size_of::<u32>()) }
    }

    #[inline]
    fn status(&self) -> VolAddress<u32, Safe, ()> {
        unsafe { VolAddress::new(self.0 + size_of::<u32>() * 3) }
    }

    #[inline]
    fn control(&self) -> VolAddress<u32, (), Safe> {
        unsafe { VolAddress::new(self.0 + size_of::<u32>() * 3) }
    }
}

impl VgaDisplay {
    pub fn new(DevMem(mem): &DevMem) -> io::Result<Self> {
        let control = MmapOptions::new()
            .offset(PIXEL_BUF_CTRL_BASE)
            .len(PIXEL_BUF_CTRL_SPAN as usize)
            .map_raw(mem)?;
        let buffer1 = MmapOptions::new()
            .offset(PIXEL_BUF1_BASE)
            .len(PIXEL_BUF_SPAN as usize)
            .map_raw(mem)?;
        let buffer2 = MmapOptions::new()
            .offset(PIXEL_BUF2_BASE)
            .len(PIXEL_BUF_SPAN as usize)
            .map_raw(mem)?;
        let char_buf = MmapOptions::new()
            .offset(CHAR_BUF_BASE)
            .len(CHAR_BUF_SPAN as usize)
            .map_raw(mem)?;

        let is_first;

        // set up the addresses here
        unsafe {
            let regs = VideoRegisters::new(control.as_ptr());

            // enable first
            regs.control().write(regs.status().read() | DISPLAY_ENABLE);

            let front_buf = regs.buffer().read();
            is_first = front_buf == PIXEL_BUF2_BASE as u32;

            // let's keep this in, just in case
            if front_buf == regs.back_buffer().read() {
                regs.back_buffer().write(PIXEL_BUF2_BASE as u32);
            }

            println!("Front buffer is {front_buf:x}");
            println!(
                "Back buffer is {back_buf:x}",
                back_buf = regs.back_buffer().read()
            );
        }

        Ok(Self {
            control,
            buffer1,
            buffer2,
            char_buf,
            is_first,
        })
    }

    // check if the front buffer is active
    pub fn is_front(&self) -> bool {
        !self.is_first
    }

    fn buf_ptr(&mut self) -> *mut u8 {
        (if self.is_first {
            &self.buffer1
        } else {
            &self.buffer2
        })
        .as_mut_ptr()
    }

    pub fn draw_texture(&mut self, x: usize, y: usize, texture: &Texture) {
        // assertions to make sure we don't go out of bounds
        assert!(x + texture.width() <= PIXEL_BUF_WIDTH);
        assert!(y + texture.height() <= PIXEL_BUF_HEIGHT);

        let buf_ptr = self.buf_ptr();

        for ((y, x), color) in (y..y + texture.height())
            .cartesian_product(x..x + texture.width())
            .zip(texture.data().iter().copied())
        {
            Self::plot_on_buf(x, y, color, buf_ptr);
        }
    }

    // TODO: remove unused methods
    pub fn plot_pixel(&mut self, x: usize, y: usize, color: Color) {
        // clamp inputs
        let x = x.min(PIXEL_BUF_WIDTH - 1);
        let y = y.min(PIXEL_BUF_HEIGHT - 1);

        Self::plot_on_buf(x, y, color, self.buf_ptr());
    }

    // line drawing algorithm taken from DE1-SoC VGA driver
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        // clamp inputs
        let mut x0 = x0.min(PIXEL_BUF_WIDTH - 1) as isize;
        let mut y0 = y0.min(PIXEL_BUF_HEIGHT - 1) as isize;
        let mut x1 = x1.min(PIXEL_BUF_WIDTH - 1) as isize;
        let mut y1 = y1.min(PIXEL_BUF_HEIGHT - 1) as isize;

        let is_steep = (y1 - y0).abs() > x1 - x0;
        if is_steep {
            mem::swap(&mut x0, &mut y0);
            mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
            mem::swap(&mut y0, &mut y1);
        }

        let deltax = x1 - x0;
        let deltay = (y0 - y1).abs();
        let mut error = -(deltax / 2);
        let mut y = y0;

        let y_step = if y0 < y1 { 1 } else { -1 };

        let buf_ptr = self.buf_ptr();

        for x in x0..=x1 {
            if is_steep {
                Self::plot_on_buf(y as usize, x as usize, color, buf_ptr);
            } else {
                Self::plot_on_buf(x as usize, y as usize, color, buf_ptr);
            }

            error += deltay;

            if error > 0 {
                y += y_step;
                error -= deltax;
            }
        }
    }

    pub fn draw_box(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        // clamp inputs
        let mut x0 = x0.min(PIXEL_BUF_WIDTH - 1);
        let mut y0 = y0.min(PIXEL_BUF_HEIGHT - 1);
        let mut x1 = x1.min(PIXEL_BUF_WIDTH - 1);
        let mut y1 = y1.min(PIXEL_BUF_HEIGHT - 1);

        // reverse if out of order
        if x0 > x1 {
            mem::swap(&mut x0, &mut x1);
        }
        if y0 > y1 {
            mem::swap(&mut y0, &mut y1);
        }

        let buf_ptr = self.buf_ptr();

        for x in x0..=x1 {
            for y in y0..=y1 {
                Self::plot_on_buf(x, y, color, buf_ptr);
            }
        }
    }

    pub fn clear_screen(&mut self) {
        let buf_ptr = self.buf_ptr();

        for x in 0..PIXEL_BUF_WIDTH {
            for y in 0..PIXEL_BUF_HEIGHT {
                Self::plot_on_buf(x, y, 0x0, buf_ptr);
            }
        }
    }

    pub fn write_text(&mut self, x: usize, y: usize, text: &str) {
        // clamp inputs
        let mut x = x.min(CHAR_BUF_WIDTH - 1);
        let mut y = y.min(CHAR_BUF_HEIGHT - 1);

        for c in text.bytes() {
            self.put_char(x, y, c);
            x += 1;

            if x == CHAR_BUF_WIDTH {
                x = 0;
                y += 1;
            }

            if y == CHAR_BUF_HEIGHT {
                y = 0;
            }
        }
    }

    pub fn erase_text(&mut self) {
        for x in 0..CHAR_BUF_WIDTH {
            for y in 0..CHAR_BUF_HEIGHT {
                self.put_char(x, y, b' ');
            }
        }
    }

    fn put_char(&mut self, x: usize, y: usize, c: u8) {
        unsafe {
            self.char_buf
                .as_mut_ptr()
                .add((x & 0x7F) | (y & 0x3F) << 7)
                .write_volatile(c);
        }
    }

    pub async fn sync_screen(&mut self) {
        unsafe {
            let regs = VideoRegisters::new(self.control.as_ptr());

            regs.buffer().write(1);

            // busy loop
            while regs.status().read() & STATUS_FLAG != 0 {
                // let's add a small sleep here
                // TODO: determine if this works better or not
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }

        self.is_first = !self.is_first;
    }

    fn plot_on_buf(x: usize, y: usize, color: Color, buf_ptr: *mut u8) {
        unsafe {
            (buf_ptr.add((x & 0x1FF) << 1 | (y & 0xFF) << 10) as *mut Color).write_volatile(color);
        }
    }
}

impl Drop for VgaDisplay {
    fn drop(&mut self) {
        unsafe {
            let regs = VideoRegisters::new(self.control.as_ptr());
            regs.control().write(regs.status().read() & !DISPLAY_ENABLE);
        }
    }
}
