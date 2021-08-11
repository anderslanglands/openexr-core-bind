use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use std::convert::TryInto;
use std::ops::Deref;

use crate::context::{Context, ContextState, WriteHeaderContext};

use imath_traits::Bound2;

use crate::error::Error;
type Result<T, E = Error> = std::result::Result<T, E>;

use openexr_core_sys as sys;

pub use sys::exr_attr_chromaticities_t as AttrChromaticities;
pub use sys::exr_attr_keycode_t as AttrKeycode;

pub use sys::exr_attr_m33d_t as AttrM33d;
pub use sys::exr_attr_m33f_t as AttrM33f;

pub use sys::exr_attr_m44d_t as AttrM44d;
pub use sys::exr_attr_m44f_t as AttrM44f;

pub use sys::exr_attr_rational_t as AttrRational;
pub use sys::exr_attr_timecode_t as AttrTimecode;

pub use sys::exr_attr_v2d_t as AttrV2d;
pub use sys::exr_attr_v2f_t as AttrV2f;
pub use sys::exr_attr_v2i_t as AttrV2i;

pub use sys::exr_attr_v3d_t as AttrV3d;
pub use sys::exr_attr_v3f_t as AttrV3f;
pub use sys::exr_attr_v3i_t as AttrV3i;

pub use sys::exr_attr_box2f_t as AttrBox2f;
pub use sys::exr_attr_box2i_t as AttrBox2i;

pub use sys::exr_attr_tiledesc_t as AttrTiledesc;

pub struct Attribute(pub(crate) sys::exr_attribute_t);

impl Attribute {
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.name)
                .to_str()
                .expect("Could not convert name string")
        }
    }

    pub fn set_name(&mut self, name: &CStr) {
        self.0.name = name.as_ptr();
    }
}

pub enum AttrString<'a> {
    Ref(&'a CStr),
    Owned(CString),
}

impl<'a> AttrString<'a> {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        AttrString::Owned(
            CString::new(s.as_ref())
                .expect("Bad bytes in AttrString constructor"),
        )
    }
}

impl<'a> From<&'a AttrString<'a>> for sys::exr_attr_string_t {
    fn from(a: &'a AttrString) -> Self {
        match a {
            AttrString::Ref(s) => sys::exr_attr_string_t {
                length: s.to_bytes_with_nul().len() as i32,
                alloc_size: 0,
                str_: s.as_ptr(),
            },
            AttrString::Owned(s) => sys::exr_attr_string_t {
                length: s.to_bytes_with_nul().len() as i32,
                alloc_size: 0,
                str_: s.as_ptr(),
            },
        }
    }
}

impl<'a> From<&'a sys::exr_attr_string_t> for AttrString<'a> {
    fn from(a: &'a sys::exr_attr_string_t) -> AttrString<'a> {
        // # Safety
        // This is safe as long as sys::exr_attr_string_t is null-terminated
        unsafe { AttrString::Ref(CStr::from_ptr(a.str_)) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Compression {
    None,
    Rle,
    Zips,
    Zip,
    Piz,
    Pxr24,
    B44,
    B44a,
    Dwaa,
    Dwab,
}

impl From<Compression> for sys::exr_compression_t {
    fn from(c: Compression) -> sys::exr_compression_t {
        match c {
            Compression::None => sys::exr_compression_t::EXR_COMPRESSION_NONE,
            Compression::Rle => sys::exr_compression_t::EXR_COMPRESSION_RLE,
            Compression::Zips => sys::exr_compression_t::EXR_COMPRESSION_ZIPS,
            Compression::Zip => sys::exr_compression_t::EXR_COMPRESSION_ZIP,
            Compression::Piz => sys::exr_compression_t::EXR_COMPRESSION_PIZ,
            Compression::Pxr24 => sys::exr_compression_t::EXR_COMPRESSION_PXR24,
            Compression::B44 => sys::exr_compression_t::EXR_COMPRESSION_B44,
            Compression::B44a => sys::exr_compression_t::EXR_COMPRESSION_B44A,
            Compression::Dwaa => sys::exr_compression_t::EXR_COMPRESSION_DWAA,
            Compression::Dwab => sys::exr_compression_t::EXR_COMPRESSION_DWAB,
        }
    }
}

impl From<sys::exr_compression_t> for Compression {
    fn from(c: sys::exr_compression_t) -> Compression {
        match c {
            sys::exr_compression_t::EXR_COMPRESSION_NONE => Compression::None,
            sys::exr_compression_t::EXR_COMPRESSION_RLE => Compression::Rle,
            sys::exr_compression_t::EXR_COMPRESSION_ZIPS => Compression::Zips,
            sys::exr_compression_t::EXR_COMPRESSION_ZIP => Compression::Zip,
            sys::exr_compression_t::EXR_COMPRESSION_PIZ => Compression::Piz,
            sys::exr_compression_t::EXR_COMPRESSION_PXR24 => Compression::Pxr24,
            sys::exr_compression_t::EXR_COMPRESSION_B44 => Compression::B44,
            sys::exr_compression_t::EXR_COMPRESSION_B44A => Compression::B44a,
            sys::exr_compression_t::EXR_COMPRESSION_DWAA => Compression::Dwaa,
            sys::exr_compression_t::EXR_COMPRESSION_DWAB => Compression::Dwab,
            _ => panic!("Unhandled compression variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Envmap {
    Latlong,
    Cube,
}

impl From<Envmap> for sys::exr_envmap_t {
    fn from(e: Envmap) -> Self {
        match e {
            Envmap::Latlong => sys::exr_envmap_t::EXR_ENVMAP_LATLONG,
            Envmap::Cube => sys::exr_envmap_t::EXR_ENVMAP_CUBE,
        }
    }
}

impl From<sys::exr_envmap_t> for Envmap {
    fn from(e: sys::exr_envmap_t) -> Self {
        match e {
            sys::exr_envmap_t::EXR_ENVMAP_LATLONG => Envmap::Latlong,
            sys::exr_envmap_t::EXR_ENVMAP_CUBE => Envmap::Cube,
            _ => panic!("Unhandled envmap variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LineOrder {
    IncreasingY,
    DecreasingY,
    RandomY,
}

impl From<LineOrder> for sys::exr_lineorder_t {
    fn from(e: LineOrder) -> Self {
        match e {
            LineOrder::IncreasingY => {
                sys::exr_lineorder_t::EXR_LINEORDER_INCREASING_Y
            }
            LineOrder::DecreasingY => {
                sys::exr_lineorder_t::EXR_LINEORDER_DECREASING_Y
            }
            LineOrder::RandomY => sys::exr_lineorder_t::EXR_LINEORDER_RANDOM_Y,
        }
    }
}

impl From<sys::exr_lineorder_t> for LineOrder {
    fn from(e: sys::exr_lineorder_t) -> Self {
        match e {
            sys::exr_lineorder_t::EXR_LINEORDER_INCREASING_Y => {
                LineOrder::IncreasingY
            }
            sys::exr_lineorder_t::EXR_LINEORDER_DECREASING_Y => {
                LineOrder::DecreasingY
            }
            sys::exr_lineorder_t::EXR_LINEORDER_RANDOM_Y => LineOrder::RandomY,
            _ => panic!("Unhandled lineorder variant"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Storage {
    Scanline,
    Tiled,
    DeepScanline,
    DeepTiled,
}

impl From<Storage> for sys::exr_storage_t {
    fn from(s: Storage) -> sys::exr_storage_t {
        match s {
            Storage::Scanline => sys::exr_storage_t::EXR_STORAGE_SCANLINE,
            Storage::Tiled => sys::exr_storage_t::EXR_STORAGE_TILED,
            Storage::DeepScanline => {
                sys::exr_storage_t::EXR_STORAGE_DEEP_SCANLINE
            }
            Storage::DeepTiled => sys::exr_storage_t::EXR_STORAGE_DEEP_TILED,
        }
    }
}

impl From<sys::exr_storage_t> for Storage {
    fn from(s: sys::exr_storage_t) -> Storage {
        match s {
            sys::exr_storage_t::EXR_STORAGE_SCANLINE => Storage::Scanline,
            sys::exr_storage_t::EXR_STORAGE_TILED => Storage::Tiled,
            sys::exr_storage_t::EXR_STORAGE_DEEP_SCANLINE => {
                Storage::DeepScanline
            }
            sys::exr_storage_t::EXR_STORAGE_DEEP_TILED => Storage::DeepTiled,
            _ => {
                panic!("unhandled exr_storage_t value")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LevelMode {
    OneLevel,
    MipmapLevels,
    RipmapLevels,
}

impl From<LevelMode> for sys::exr_tile_level_mode_t {
    fn from(s: LevelMode) -> sys::exr_tile_level_mode_t {
        match s {
            LevelMode::OneLevel => {
                sys::exr_tile_level_mode_t::EXR_TILE_ONE_LEVEL
            }
            LevelMode::MipmapLevels => {
                sys::exr_tile_level_mode_t::EXR_TILE_MIPMAP_LEVELS
            }
            LevelMode::RipmapLevels => {
                sys::exr_tile_level_mode_t::EXR_TILE_RIPMAP_LEVELS
            }
        }
    }
}

impl From<sys::exr_tile_level_mode_t> for LevelMode {
    fn from(s: sys::exr_tile_level_mode_t) -> LevelMode {
        match s {
            sys::exr_tile_level_mode_t::EXR_TILE_ONE_LEVEL => {
                LevelMode::OneLevel
            }
            sys::exr_tile_level_mode_t::EXR_TILE_MIPMAP_LEVELS => {
                LevelMode::MipmapLevels
            }
            sys::exr_tile_level_mode_t::EXR_TILE_RIPMAP_LEVELS => {
                LevelMode::RipmapLevels
            }
            _ => {
                panic!("unhandled exr_tile_level_mode_t value")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TileRoundMode {
    RoundDown,
    RoundUp,
}

impl From<TileRoundMode> for sys::exr_tile_round_mode_t {
    fn from(e: TileRoundMode) -> Self {
        match e {
            TileRoundMode::RoundDown => {
                sys::exr_tile_round_mode_t::EXR_TILE_ROUND_DOWN
            }
            TileRoundMode::RoundUp => {
                sys::exr_tile_round_mode_t::EXR_TILE_ROUND_UP
            }
        }
    }
}

impl From<sys::exr_tile_round_mode_t> for TileRoundMode {
    fn from(e: sys::exr_tile_round_mode_t) -> Self {
        match e {
            sys::exr_tile_round_mode_t::EXR_TILE_ROUND_DOWN => {
                TileRoundMode::RoundDown
            }
            sys::exr_tile_round_mode_t::EXR_TILE_ROUND_UP => {
                TileRoundMode::RoundUp
            }
            _ => {
                panic!("unhandled exr_tile_round_mode_t value")
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PixelType {
    Uint,
    Half,
    Float,
}

impl From<PixelType> for sys::exr_pixel_type_t {
    fn from(e: PixelType) -> Self {
        match e {
            PixelType::Uint => sys::exr_pixel_type_t::EXR_PIXEL_UINT,
            PixelType::Half => sys::exr_pixel_type_t::EXR_PIXEL_HALF,
            PixelType::Float => sys::exr_pixel_type_t::EXR_PIXEL_FLOAT,
        }
    }
}

impl From<sys::exr_pixel_type_t> for PixelType {
    fn from(e: sys::exr_pixel_type_t) -> Self {
        match e {
            sys::exr_pixel_type_t::EXR_PIXEL_UINT => PixelType::Uint,
            sys::exr_pixel_type_t::EXR_PIXEL_HALF => PixelType::Half,
            sys::exr_pixel_type_t::EXR_PIXEL_FLOAT => PixelType::Float,
            _ => panic!("Unhandled exr_pixel_type_t value"),
        }
    }
}

#[repr(transparent)]
pub struct Channel(sys::exr_attr_chlist_entry_t);

impl Channel {
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.0.name.str_)
                .to_str()
                .expect("Failed to convert channel name")
        }
    }

    pub fn pixel_type(&self) -> PixelType {
        self.0.pixel_type.into()
    }

    pub fn p_linear(&self) -> bool {
        self.0.p_linear != 0
    }

    pub fn x_sampling(&self) -> i32 {
        self.0.x_sampling
    }

    pub fn y_sampling(&self) -> i32 {
        self.0.y_sampling
    }
}

#[repr(transparent)]
pub struct ChannelList(sys::exr_attr_chlist_t);

impl ChannelList {
    pub fn as_slice(&self) -> &[Channel] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.entries as *const Channel,
                self.0.num_channels.try_into().unwrap(),
            )
        }
    }
}

impl AsRef<[Channel]> for ChannelList {
    fn as_ref(&self) -> &[Channel] {
        self.as_slice()
    }
}

impl Deref for ChannelList {
    type Target = [Channel];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

pub trait AttributeRead: Sized {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<Self>;
}

pub trait AttributeWrite: Sized {
    fn set(
        ctx: &WriteHeaderContext,
        part_index: usize,
        name: &str,
        value: &Self,
    ) -> Result<()>;
}

impl AttributeRead for f32 {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<Self> {
        let mut result = Default::default();
        unsafe {
            let c_name = CString::new(name).unwrap();
            sys::exr_attr_get_float(
                ctx.inner,
                part_index.try_into().unwrap(),
                c_name.as_ptr(),
                &mut result,
            )
            .ok(result)
        }
    }
}

impl AttributeRead for i32 {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<Self> {
        let mut result = Default::default();
        unsafe {
            let c_name = CString::new(name).unwrap();
            sys::exr_attr_get_int(
                ctx.inner,
                part_index.try_into().unwrap(),
                c_name.as_ptr(),
                &mut result,
            )
            .ok(result)
        }
    }
}

impl AttributeRead for &[f32] {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<Self> {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let mut sz = 0;
            let mut ptr = std::ptr::null();
            sys::exr_attr_get_float_vector(
                ctx.inner,
                part_index.try_into().unwrap(),
                c_name.as_ptr(),
                &mut sz,
                &mut ptr,
            )
            .ok(std::slice::from_raw_parts(ptr, sz as usize))
        }
    }
}

impl AttributeRead for Compression {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<Self> {
        let mut result = sys::exr_compression_t::EXR_COMPRESSION_LAST_TYPE;
        unsafe {
            let c_name = CString::new(name).unwrap();
            sys::exr_attr_get_compression(
                ctx.inner,
                part_index.try_into().unwrap(),
                c_name.as_ptr(),
                &mut result,
            )
            .ok(result.into())
        }
    }
}

impl AttributeRead for [i32; 4] {
    fn get<S: ContextState>(
        ctx: &Context<S>,
        part_index: usize,
        name: &str,
    ) -> Result<[i32; 4]> {
        let mut result = [0i32; 4];
        unsafe {
            let c_name = CString::new(name).unwrap();
            sys::exr_attr_get_box2i(
                ctx.inner,
                part_index.try_into().unwrap(),
                c_name.as_ptr(),
                result.as_mut_ptr() as *mut sys::exr_attr_box2i_t,
            )
            .ok(result.into())
        }
    }
}
