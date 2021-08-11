use crate::attr::{
    Attribute, AttributeRead, ChannelList, Compression, LevelMode, LineOrder,
    Storage,
};
use crate::context::*;
use crate::error::Error;
use openexr_core_sys as sys;
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::path::Path;

use imath_traits::{Bound2, Vec2};

type Result<T, E = Error> = std::result::Result<T, E>;

impl<S: ContextState> Context<S> {
    //! Part-related methods
    //!
    //! A part is a separate entity in the OpenEXR file. This was
    //! formalized in the OpenEXR 2.0 timeframe to allow there to be a
    //! clear set of eyes for stereo, or just a simple list of AOVs within
    //! a single OpenEXR file. Prior, it was managed by name convention,
    //! but with a multi-part file, they are clearly separate types, and
    //! can have separate behavior.
    //!
    //! This is a set of functions to query, or set up when writing, that
    //! set of parts within a file. This remains backward compatible to
    //! OpenEXR files from before this change, in that a file with a single
    //! part is a subset of a multi-part file. As a special case, creating
    //! a file with a single part will write out as if it is a file which
    //! is not multi-part aware, so as to be compatible with those old
    //! libraries.

    /// Get the number of parts in the file.
    ///
    pub fn count(&self) -> Result<usize> {
        let mut count = 0;
        unsafe { sys::exr_get_count(self.inner, &mut count).ok(count as usize) }
    }

    /// Get the name of the given part
    ///
    /// # Returns
    /// * `Ok(Some(&str))` - If `part_index` refers to a valid part with a valid
    /// name attribute
    /// * `Ok(None)` - If the file is a single-part file without a name attribute
    /// name attribute
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part
    /// * `Err(Error::FileBadHeader)` - If the name attribute is present but is
    /// not a string
    ///
    pub fn name(&self, part_index: usize) -> Result<Option<&str>> {
        let mut ptr = std::ptr::null();
        unsafe {
            match sys::exr_get_name(self.inner, part_index as i32, &mut ptr)
                .ok(())
            {
                Ok(_) => (),
                Err(Error::NoAttrByName) => (),
                Err(e) => return Err(e),
            }
            if ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(CStr::from_ptr(ptr).to_str().unwrap()))
            }
        }
    }

    /// Get the storage type for the given part
    ///
    /// # Returns
    /// * `Ok(Storage)` - if `part_index` refers to a valid part
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part
    ///
    pub fn storage(&self, part_index: usize) -> Result<Storage> {
        let mut storage = sys::exr_storage_t::EXR_STORAGE_LAST_TYPE;
        unsafe {
            sys::exr_get_storage(self.inner, part_index as i32, &mut storage)
                .ok(storage.into())
        }
    }

    /// Get the number of levels in the specified part
    ///
    /// # Returns
    /// * `Ok(usize, usize)` - the number of levels in the x and y dimensions, respectively, on
    /// success.
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part
    /// * `Err(Error::TileScanMixedApi)` - if the file is not tiled
    /// * `Err(Error::MissingReqAttr)` - if the tile data is missing or corrupt
    ///
    pub fn tile_levels(&self, part_index: usize) -> Result<(usize, usize)> {
        let mut x = 0;
        let mut y = 0;
        unsafe {
            sys::exr_get_tile_levels(
                self.inner,
                part_index as i32,
                &mut x,
                &mut y,
            )
            .ok(())
            .map(|_| (x as usize, y as usize))
        }
    }

    /// Get the size of tiles in the given level in the given part
    ///
    /// # Returns
    /// * `Ok(usize, usize)` - the width and height of the tiles at the specified level
    /// success.
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part, or `level_x` or `level_y` are not valid level indices
    /// * `Err(Error::TileScanMixedApi)` - if the file is not tiled
    /// * `Err(Error::MissingReqAttr)` - if the tile data is missing or corrupt
    ///
    pub fn tile_sizes(
        &self,
        part_index: usize,
        level_x: usize,
        level_y: usize,
    ) -> Result<(usize, usize)> {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::exr_get_tile_sizes(
                self.inner,
                part_index as i32,
                level_x as i32,
                level_y as i32,
                &mut w,
                &mut h,
            )
            .ok(())
            .map(|_| (w as usize, h as usize))
        }
    }

    /// Get the size of the given level in the given part
    ///
    /// # Returns
    /// * `Ok(usize, usize)` - the width and height of the given level in the specified part
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part, or `level_x` or `level_y` are not valid level indices
    /// * `Err(Error::TileScanMixedApi)` - if the file is not tiled
    /// * `Err(Error::MissingReqAttr)` - if the tile data is missing or corrupt
    ///
    pub fn level_sizes(
        &self,
        part_index: usize,
        level_x: usize,
        level_y: usize,
    ) -> Result<(usize, usize)> {
        let mut w = 0;
        let mut h = 0;
        unsafe {
            sys::exr_get_level_sizes(
                self.inner,
                part_index as i32,
                level_x as i32,
                level_y as i32,
                &mut w,
                &mut h,
            )
            .ok(())
            .map(|_| (w as usize, h as usize))
        }
    }

    /// Get the number of chunks in this part of the file.
    ///
    /// As in the technical documentation for OpenEXR, the chunk is the
    /// generic term for a pixel data block. This is the atomic unit that
    /// this library uses to negotiate data to and from a context.
    ///
    /// This should be used as a basis for splitting up how a file is
    /// processed. Depending on the compression, a different number of
    /// scanlines are encoded in each chunk, and since those need to be
    /// encoded / decoded as a block, the chunk should be the basis for I/O
    /// as well.
    ///
    /// # Returns
    /// * `Ok(usize)` - the number of chunks in the part on success
    /// * `Err(Error::ArgumentOutOfRange)` - If `part_index` does not refer to
    /// a valid part
    ///
    pub fn chunk_count(&self, part_index: usize) -> Result<usize> {
        let mut count = 0;
        unsafe {
            sys::exr_get_chunk_count(self.inner, part_index as i32, &mut count)
                .ok(count as usize)
        }
    }

    /// Return the number of scanlines chunks for this file part
    ///
    /// When iterating over a scanline file, this may be an easier metric
    /// for multi-threading or other access than only negotiating chunk
    /// counts, and so is provided as a utility.
    ///
    pub fn scanlines_per_chunk(&self, part_index: usize) -> Result<usize> {
        let mut count = 0;
        unsafe {
            sys::exr_get_scanlines_per_chunk(
                self.inner,
                part_index as i32,
                &mut count,
            )
            .ok(count as usize)
        }
    }

    /// Return the maximum unpacked size of a chunk for the file part
    ///
    /// This may be used ahead of any actual reading of data, so can be
    /// used to pre-allocate buffers for multiple threads in one block or
    /// whatever your application may require.
    ///
    ///
    pub fn chunk_unpacked_size(&self, part_index: usize) -> Result<usize> {
        let mut count = 0;
        unsafe {
            sys::exr_get_chunk_unpacked_size(
                self.inner,
                part_index as i32,
                &mut count,
            )
            .ok(count as usize)
        }
    }

    /// Get the compression method used for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn compression(&self, part_index: usize) -> Result<Compression> {
        let mut result = sys::exr_compression_t::EXR_COMPRESSION_LAST_TYPE;
        unsafe {
            sys::exr_get_compression(
                self.inner,
                part_index.try_into().unwrap(),
                &mut result,
            )
            .ok(result.into())
        }
    }

    /// Get the data window for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn data_window<B: Bound2<i32>>(&self, part_index: usize) -> Result<B> {
        let mut result = [0i32; 4];
        unsafe {
            sys::exr_get_data_window(
                self.inner,
                part_index.try_into().unwrap(),
                result.as_mut_ptr() as *mut sys::exr_attr_box2i_t,
            )
            .ok(B::from_slice(&result))
        }
    }

    /// Get the display window for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn display_window<B: Bound2<i32>>(
        &self,
        part_index: usize,
    ) -> Result<B> {
        let mut result = [0i32; 4];
        unsafe {
            sys::exr_get_display_window(
                self.inner,
                part_index.try_into().unwrap(),
                result.as_mut_ptr() as *mut sys::exr_attr_box2i_t,
            )
            .ok(B::from_slice(&result))
        }
    }

    /// Get the lineorder method used for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn lineorder(&self, part_index: usize) -> Result<LineOrder> {
        let mut result = sys::exr_lineorder_t::EXR_LINEORDER_LAST_TYPE;
        unsafe {
            sys::exr_get_lineorder(
                self.inner,
                part_index.try_into().unwrap(),
                &mut result,
            )
            .ok(result.into())
        }
    }

    /// Get the pixel aspect ratio for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn pixel_aspect_ratio(&self, part_index: usize) -> Result<f32> {
        let mut result = 0.0f32;
        unsafe {
            sys::exr_get_pixel_aspect_ratio(
                self.inner,
                part_index.try_into().unwrap(),
                &mut result,
            )
            .ok(result.into())
        }
    }

    /// Get the screen window center for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn screen_window_center<V: Vec2<f32>>(
        &self,
        part_index: usize,
    ) -> Result<V> {
        let mut result = [0.0f32; 2];
        unsafe {
            sys::exr_get_screen_window_center(
                self.inner,
                part_index.try_into().unwrap(),
                result.as_mut_ptr() as *mut sys::exr_attr_v2f_t,
            )
            .ok(V::from_slice(&result))
        }
    }

    /// Get the screen window width for the specified part
    ///
    /// # Panics
    /// If `part_index` is outside the range of an i32
    ///
    /// # Errors
    /// * `[Error::FileBadHeader]` - If the header could not be read
    /// * `[Error::NoAttrByName]` - If the attribute could not be found
    ///
    pub fn screen_window_width(&self, part_index: usize) -> Result<f32> {
        let mut result = 0.0f32;
        unsafe {
            sys::exr_get_screen_window_width(
                self.inner,
                part_index.try_into().unwrap(),
                &mut result,
            )
            .ok(result.into())
        }
    }

    /// Get the list of channels
    ///
    pub fn channels(&self, part_index: usize) -> Result<&ChannelList> {
        let mut ptr = std::ptr::null();
        unsafe {
            sys::exr_get_channels(
                self.inner,
                part_index.try_into().unwrap(),
                &mut ptr as *mut *const ChannelList
                    as *mut *const sys::exr_attr_chlist_t,
            )
            .ok(&*ptr)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AttrListAccessMode {
    FileOrder,
    SortedOrder,
}

impl From<AttrListAccessMode> for sys::exr_attr_list_access_mode {
    fn from(s: AttrListAccessMode) -> sys::exr_attr_list_access_mode {
        match s {
            AttrListAccessMode::FileOrder => {
                sys::exr_attr_list_access_mode::EXR_ATTR_LIST_FILE_ORDER
            }
            AttrListAccessMode::SortedOrder => {
                sys::exr_attr_list_access_mode::EXR_ATTR_LIST_SORTED_ORDER
            }
        }
    }
}

impl From<sys::exr_attr_list_access_mode> for AttrListAccessMode {
    fn from(s: sys::exr_attr_list_access_mode) -> AttrListAccessMode {
        match s {
            sys::exr_attr_list_access_mode::EXR_ATTR_LIST_FILE_ORDER => {
                AttrListAccessMode::FileOrder
            }
            sys::exr_attr_list_access_mode::EXR_ATTR_LIST_SORTED_ORDER => {
                AttrListAccessMode::SortedOrder
            }
            _ => panic!("Unhandled exr_attr_list_access_mode variant"),
        }
    }
}

impl<S: ContextState> Context<S> {
    //! Part metadata functions

    /// Get the number of attributes in the part
    ///
    pub fn attribute_count(&self, part_index: usize) -> Result<usize> {
        let mut count = 0;
        unsafe {
            sys::exr_get_attribute_count(
                self.inner,
                part_index as i32,
                &mut count,
            )
            .ok(count as usize)
        }
    }

    /// Get an attribute by its index
    ///
    pub fn get_attribute_by_index(
        &self,
        part_index: usize,
        mode: AttrListAccessMode,
        index: usize,
    ) -> Result<&Attribute> {
        let mut attr = std::ptr::null();
        unsafe {
            sys::exr_get_attribute_by_index(
                self.inner,
                part_index as i32,
                mode.into(),
                index as i32,
                &mut attr,
            )
            .ok(&*(attr as *const Attribute))
        }
    }

    /// Get an attribute by its name
    ///
    pub fn get_attribute_by_name(
        &self,
        part_index: usize,
        name: &str,
    ) -> Result<&Attribute> {
        let c_name = CString::new(name).expect("Invalid bytes in name");
        let mut attr = std::ptr::null();
        unsafe {
            sys::exr_get_attribute_by_name(
                self.inner,
                part_index as i32,
                c_name.as_ptr(),
                &mut attr,
            )
            .ok(&*(attr as *const Attribute))
        }
    }

    pub fn get_attribute<Attr: AttributeRead>(
        &self,
        part_index: usize,
        name: &str,
    ) -> Result<Attr> {
        <Attr as AttributeRead>::get(self, part_index, name)
    }
}

impl WriteContext {
    /// Add a new part in the file with name `part_name`
    ///
    /// # Returns
    /// * `Ok(part_index)` - the index of the new part on success
    /// * `Err(Error)`  - otherwise
    ///
    pub fn add_part(
        &mut self,
        part_name: &str,
        storage_type: Storage,
    ) -> Result<usize> {
        let c_part_name =
            CString::new(part_name).expect("invalid bytes in part_name");
        let mut part_index = 0;
        unsafe {
            sys::exr_add_part(
                self.inner,
                c_part_name.as_ptr(),
                storage_type.into(),
                &mut part_index,
            )
            .ok(part_index as usize)
        }
    }
}
