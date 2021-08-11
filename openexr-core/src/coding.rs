use crate::attr::{
    Attribute, AttributeRead, Compression, LevelMode, LineOrder, PixelType,
    Storage,
};
use crate::chunkio::ChunkInfo;
use crate::context::*;
use crate::error::Error;
use openexr_core_sys as sys;
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::path::Path;

use imath_traits::{Bound2, Vec2};

type Result<T, E = Error> = std::result::Result<T, E>;

#[repr(transparent)]
pub struct ChannelInfo(sys::exr_coding_channel_info_t);

impl ChannelInfo {
    /// Name of the channel
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.0.channel_name).to_str().unwrap() }
    }

    /// Number of lines for this channel in this chunk.
    ///
    /// May be 0 or less than overall image height when in 4:2:0-type sampling
    ///
    pub fn height(&self) -> usize {
        self.0.height as usize
    }

    /// Width in pixel count
    ///
    /// May be 0 or less than overall image width when in 4:2:0-type sampling
    ///
    pub fn width(&self) -> usize {
        self.0.width as usize
    }

    pub fn x_samples(&self) -> usize {
        self.0.x_samples as usize
    }

    pub fn y_samples(&self) -> usize {
        self.0.y_samples as usize
    }

    /// Is the channel perceptually linear?
    ///
    pub fn p_linear(&self) -> bool {
        self.0.p_linear != 0
    }

    /// How many bytes each element consumes, i.e. 2 for f16, 4 for f32
    ///
    pub fn bytes_per_element(&self) -> usize {
        self.0.bytes_per_element as usize
    }

    pub fn data_type(&self) -> PixelType {
        match self.0.data_type {
            0 => PixelType::Uint,
            1 => PixelType::Half,
            2 => PixelType::Float,
            _ => panic!("Unhandled value for pixel type"),
        }
    }

    /// How many bytes per pixel the input is or output should be
    /// (i.e. 2 for float16, 4 for float32 / uint32). Defaults to same
    /// size as input
    ///
    pub fn user_bytes_per_element(&self) -> usize {
        self.0.user_bytes_per_element as usize
    }

    /// How many bytes per pixel the input is or output should be
    /// (i.e. 2 for float16, 4 for float32 / uint32). Defaults to same
    /// size as input
    ///
    pub fn set_user_bytes_per_element(&mut self, value: usize) {
        self.0.user_bytes_per_element =
            value.try_into().expect("value is not representable");
    }

    pub fn user_data_type(&self) -> PixelType {
        match self.0.user_data_type {
            0 => PixelType::Uint,
            1 => PixelType::Half,
            2 => PixelType::Float,
            _ => panic!("Unhandled value for pixel type"),
        }
    }

    pub fn set_user_data_type(&mut self, value: PixelType) {
        self.0.user_data_type =
            match value {
                PixelType::Uint => 0,
                PixelType::Half => 1,
                PixelType::Float => 2,
            }
    }

    /// Increment to next pixel in bytes
    ///
    pub fn user_pixel_stride(&self) -> usize {
        self.0.user_pixel_stride as usize
    }

    /// Increment to next pixel in bytes
    ///
    pub fn set_user_pixel_stride(&mut self, value: usize) {
        self.0.user_pixel_stride =
            value.try_into().expect("value is not representable");
    }

    /// Increment to next line in bytes
    ///
    pub fn user_line_stride(&self) -> usize {
        self.0.user_line_stride as usize
    }

    /// Increment to next line in bytes
    ///
    pub fn set_user_line_stride(&mut self, value: usize) {
        self.0.user_line_stride =
            value.try_into().expect("value is not representable");
    }

    pub unsafe fn set_decode_to(&mut self, ptr: *mut u8) {
        self.0.__bindgen_anon_1.decode_to_ptr = ptr;
    }

}
