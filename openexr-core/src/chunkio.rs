use crate::attr::{
    Attribute, AttributeRead, Compression, LevelMode, LineOrder, Storage,
};
use crate::context::*;
use crate::error::Error;
use openexr_core_sys as sys;
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::path::Path;

use imath_traits::{Bound2, Vec2};

type Result<T, E = Error> = std::result::Result<T, E>;

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct ChunkInfo {
    pub idx: i32,
    pub start_x: i32,
    pub start_y: i32,
    /// Height of this chunk
    pub height: i32,
    /// Width of this chunk
    pub width: i32,
    pub level_x: u8, //< For tiled files
    pub level_y: u8, //< For tiled files

    pub data_type: u8,
    pub compression: u8,

    pub data_offset: u64,
    pub packed_size: u64,
    pub unpacked_size: u64,

    pub sample_count_data_offset: u64,
    pub sample_count_table_size: u64,
}

impl ReadContext {
    pub fn read_scanline_chunk_info(
        &self,
        part_index: usize,
        y: i32,
    ) -> Result<ChunkInfo> {
        let mut result = ChunkInfo::default();
        unsafe {
            sys::exr_read_scanline_chunk_info(
                self.inner,
                part_index.try_into().unwrap(),
                y,
                &mut result as *mut ChunkInfo as *mut sys::exr_chunk_info_t,
            )
            .ok(result)
        }
    }

    pub fn read_tile_chunk_info(
        &self,
        part_index: usize,
        tile_x: i32,
        tile_y: i32,
        level_x: i32,
        level_y: i32,
    ) -> Result<ChunkInfo> {
        let mut result = ChunkInfo::default();
        unsafe {
            sys::exr_read_tile_chunk_info(
                self.inner,
                part_index.try_into().unwrap(),
                tile_x,
                tile_y,
                level_x,
                level_y,
                &mut result as *mut ChunkInfo as *mut sys::exr_chunk_info_t,
            )
            .ok(result)
        }
    }

    /// Read the packed data block for the given chunk
    ///
    /// # Safety
    /// `packed_data` must be big enough to hold `chunk_info.packed_size` bytes
    ///
    pub unsafe fn read_chunk(
        &self,
        part_index: usize,
        chunk_info: &ChunkInfo,
        packed_data: &mut [u8],
    ) -> Result<()> {
        sys::exr_read_chunk(
            self.inner,
            part_index.try_into().unwrap(),
            chunk_info as *const ChunkInfo as *const sys::exr_chunk_info_t,
            packed_data.as_mut_ptr() as *mut c_void,
        )
        .ok(())
    }
}
