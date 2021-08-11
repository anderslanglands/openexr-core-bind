use crate::attr::{
    Attribute, AttributeRead, Compression, LevelMode, LineOrder, Storage,
};
use crate::chunkio::ChunkInfo;
use crate::coding::ChannelInfo;
use crate::context::*;
use crate::error::Error;
use openexr_core_sys as sys;
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::path::Path;

use imath_traits::{Bound2, Vec2};

type Result<T, E = Error> = std::result::Result<T, E>;

#[repr(transparent)]
// We have to box this because exr_decode_pipeline_t uses a small-buffer 
// optimization internally
pub struct DecodePipeline(Box<sys::exr_decode_pipeline_t>);

impl DecodePipeline {
    pub fn channels(&self) -> &[ChannelInfo] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.channels as *const ChannelInfo,
                self.0.channel_count as usize,
            )
        }
    }

    pub fn channels_mut(&mut self) -> &mut [ChannelInfo] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.0.channels as *mut ChannelInfo,
                self.0.channel_count as usize,
            )
        }
    }
}

impl Default for DecodePipeline {
    fn default() -> Self {
        let d = std::mem::MaybeUninit::<sys::exr_decode_pipeline_t>::zeroed();
        DecodePipeline(Box::new(unsafe { d.assume_init() }))
    }
}

impl ReadContext {
    /// Initialize the decoding pipeline structure with the channel info
    /// for the specified part, and the first block to be read.
    ///
    pub fn decoding_initialize(
        &self,
        part_index: usize,
        chunk_info: &ChunkInfo,
        decode_pipeline: &mut DecodePipeline,
    ) -> Result<()> {
        unsafe {
            sys::exr_decoding_initialize(
                self.inner,
                part_index.try_into().unwrap(),
                chunk_info as *const ChunkInfo as *const sys::exr_chunk_info_t,
                &mut *decode_pipeline.0,
            )
            .ok(())
        }
    }

    /// Given an initialized decode pipeline, find appropriate functions
    /// to read and shuffle / convert data into the defined channel outputs
    ///
    /// Calling this is not required if custom routines will be used, or if
    /// just the raw compressed data is desired. Although in that scenario,
    /// it is probably easier to just read the chunk directly using \ref
    /// exr_read_chunk
    ///
    pub fn decoding_choose_default_routines(
        &self,
        part_index: usize,
        decode_pipeline: &mut DecodePipeline,
    ) -> Result<()> {
        unsafe {
            sys::exr_decoding_choose_default_routines(
                self.inner,
                part_index.try_into().unwrap(),
                &mut *decode_pipeline.0,
            )
            .ok(())
        }
    }

    ///  Given a decode pipeline previously initialized, update it for the
    /// new chunk to be read.
    ///
    /// In this manner, memory buffers can be re-used to avoid continual
    /// allocations. Further, it allows the previous choices for
    /// the various functions to be quickly re-used.
    ///
    pub fn decoding_update(
        &self,
        part_index: usize,
        chunk_info: &ChunkInfo,
        decode_pipeline: &mut DecodePipeline,
    ) -> Result<()> {
        unsafe {
            sys::exr_decoding_update(
                self.inner,
                part_index.try_into().unwrap(),
                chunk_info as *const ChunkInfo as *const sys::exr_chunk_info_t,
                &mut *decode_pipeline.0,
            )
            .ok(())
        }
    }

    /// Execute the decoding pipeline
    ///
    pub unsafe fn decoding_run(
        &self,
        part_index: usize,
        decode_pipeline: &mut DecodePipeline,
    ) -> Result<()> {
        unsafe {
            sys::exr_decoding_run(
                self.inner,
                part_index.try_into().unwrap(),
                &mut *decode_pipeline.0,
            )
            .ok(())
        }
    }

    /// Free any intermediate memory in the decoding pipeline
    ///
    /// This does *not* free any pointers referred to in the channel info
    /// areas, but rather only the intermediate buffers and memory needed
    /// for the structure itself.
    ///
    pub fn decoding_destroy(
        &self,
        decode_pipeline: DecodePipeline,
    ) -> Result<()> {
        let mut decode_pipeline = decode_pipeline;
        unsafe {
            sys::exr_decoding_destroy(self.inner, &mut *decode_pipeline.0).ok(())
        }
    }
}
