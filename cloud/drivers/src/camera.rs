use std::{intrinsics, io, mem::size_of};

use common_types::{Frame, IMAGE_SIZE, IMAGE_WIDTH};
use memmap2::{MmapOptions, MmapRaw};

use crate::{devmem::DevMem, LW_BRIDGE_BASE};

const VIDEO_IN_CTRL_BASE: u64 = LW_BRIDGE_BASE + 0x0000306C;
const VIDEO_IN_CTRL_SPAN: usize = size_of::<u32>();

const BUFFER_BASE: u64 = 0xC8000000;
const BUFFER_SPAN: u64 = 0x0003FFFF;

const CAMERA_ENABLE: u32 = 1 << 2;

pub struct Camera {
    control: MmapRaw,
    buffer: MmapRaw,
}

pub struct CameraGuard<'a>(&'a mut Camera);

impl CameraGuard<'_> {
    pub fn capture_frame(&self) -> Frame {
        self.0.capture_frame()
    }
}

impl Drop for CameraGuard<'_> {
    fn drop(&mut self) {
        // disable the camera
        unsafe {
            println!("Disabled camera");
            let control_ptr = self.0.control.as_mut_ptr() as *mut u32;
            control_ptr.write_volatile(control_ptr.read_volatile() & !CAMERA_ENABLE);
        }
    }
}

impl Camera {
    pub fn new(DevMem(mem): &DevMem) -> io::Result<Self> {
        let control = MmapOptions::new()
            .offset(VIDEO_IN_CTRL_BASE)
            .len(VIDEO_IN_CTRL_SPAN)
            .map_raw(mem)?;

        let buffer = MmapOptions::new()
            .offset(BUFFER_BASE)
            .len(BUFFER_SPAN as usize)
            .map_raw(mem)?;

        unsafe {
            // make sure camera is disabled to begin with, regardless of previous state
            let control_ptr = control.as_mut_ptr() as *mut u32;
            let front_buf = control.as_mut_ptr().sub(2 * size_of::<u32>()) as *mut u32;

            println!("Camera buffer is {:x}", front_buf.read_volatile());

            control_ptr.write_volatile(control_ptr.read_volatile() & !CAMERA_ENABLE);
        }

        Ok(Self { control, buffer })
    }

    pub fn enable(&mut self) -> CameraGuard {
        unsafe {
            println!("Enabled camera");
            let control_ptr = self.control.as_mut_ptr() as *mut u32;
            control_ptr.write_volatile(control_ptr.read_volatile() | CAMERA_ENABLE);
        }

        CameraGuard(self)
    }

    fn capture_frame(&self) -> Frame {
        let mut data = vec![0u8; IMAGE_SIZE];

        for (i, data_slice) in data
            .chunks_exact_mut(IMAGE_WIDTH * size_of::<u16>())
            .enumerate()
        {
            unsafe {
                intrinsics::volatile_copy_nonoverlapping_memory(
                    data_slice.as_mut_ptr(),
                    self.buffer.as_ptr().add(i << 10),
                    IMAGE_WIDTH * size_of::<u16>(),
                );
            }
        }

        Frame(data)
    }
}
