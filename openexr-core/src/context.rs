use crate::error::Error;
use openexr_core_sys as sys;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;

type Result<T, E = Error> = std::result::Result<T, E>;

/// A context is a single instance of an OpenEXR file or stream.
///
/// Beyond a particular file or stream handle, it also has separate controls
/// for error handling and memory allocation. This is done to enable encoding or
/// decoding on mixed hardware
///
// pub struct ReadContext(pub(crate) *mut sys::_priv_exr_context_t);
// pub struct WriteContext(pub(crate) *mut sys::_priv_exr_context_t);
// pub struct WriteHeaderContext(pub(crate) *mut sys::_priv_exr_context_t);
// pub struct InplaceHeaderUpdateContext(pub(crate) *mut sys::_priv_exr_context_t);
#[repr(transparent)]
pub struct Context<S: ContextState> {
    pub(crate) inner: *mut sys::_priv_exr_context_t,
    marker: PhantomData<S>,
}

pub enum ReadState {}
pub enum WriteState {}
pub enum WriteHeaderState {}
pub enum InplaceHeaderUpdateState {}

pub trait ContextState {}
impl ContextState for ReadState {}
impl ContextState for WriteState {}
impl ContextState for WriteHeaderState {}
impl ContextState for InplaceHeaderUpdateState {}

pub type ReadContext = Context<ReadState>;
pub type WriteContext = Context<WriteState>;
pub type WriteHeaderContext = Context<WriteHeaderState>;
pub type InplaceHeaderUpdateContext = Context<InplaceHeaderUpdateState>;

impl Context<ReadState> {
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<ReadContext> {
        let c_filename = CString::new(
            filename
                .as_ref()
                .to_str()
                .expect("Invalid bytes in filename"),
        )
        .expect("Internal null bytes in filename");

        let mut inner = std::ptr::null_mut();
        unsafe {
            sys::exr_start_read(
                &mut inner,
                c_filename.as_ptr(),
                std::ptr::null(),
            )
            .ok(ReadContext {
                inner,
                marker: PhantomData,
            })
        }
    }

    pub fn file_name(&self) -> Result<&str> {
        let mut ptr = std::ptr::null();
        unsafe {
            sys::exr_get_file_name(self.inner, &mut ptr)
                .ok(())
                .map(|_| CStr::from_ptr(ptr).to_str().unwrap())
        }
    }
}

pub enum DefaultWriteMode {
    WriteFileDirectly,
    IntermediateTempFile,
}

impl From<DefaultWriteMode> for sys::exr_default_write_mode {
    fn from(m: DefaultWriteMode) -> sys::exr_default_write_mode {
        match m {
            DefaultWriteMode::WriteFileDirectly => {
                sys::exr_default_write_mode::EXR_WRITE_FILE_DIRECTLY
            }
            DefaultWriteMode::IntermediateTempFile => {
                sys::exr_default_write_mode::EXR_INTERMEDIATE_TEMP_FILE
            }
        }
    }
}

impl WriteHeaderContext {
    pub fn new<P: AsRef<Path>>(
        filename: P,
        default_write_mode: DefaultWriteMode,
    ) -> Result<WriteHeaderContext> {
        let c_filename = CString::new(
            filename
                .as_ref()
                .to_str()
                .expect("Invalid bytes in filename"),
        )
        .expect("Internal null bytes in filename");

        let mut inner = std::ptr::null_mut();
        unsafe {
            sys::exr_start_write(
                &mut inner,
                c_filename.as_ptr(),
                default_write_mode.into(),
                std::ptr::null(),
            )
            .ok(WriteHeaderContext {
                inner,
                marker: PhantomData,
            })
        }
    }

    pub fn set_longname_support(&mut self, enabled: bool) -> Result<()> {
        unsafe {
            sys::exr_set_longname_support(
                self.inner,
                if enabled { 1 } else { 0 },
            )
            .ok(())
        }
    }

    pub fn write_header(self) -> Result<WriteContext> {
        unsafe {
            sys::exr_write_header(self.inner).ok(WriteContext {
                inner: self.inner,
                marker: PhantomData,
            })
        }
    }
}

impl InplaceHeaderUpdateContext {
    pub fn new<P: AsRef<Path>>(
        filename: P,
    ) -> Result<InplaceHeaderUpdateContext> {
        let c_filename = CString::new(
            filename
                .as_ref()
                .to_str()
                .expect("Invalid bytes in filename"),
        )
        .expect("Internal null bytes in filename");

        let mut inner = std::ptr::null_mut();
        unsafe {
            sys::exr_start_read(
                &mut inner,
                c_filename.as_ptr(),
                std::ptr::null(),
            )
            .ok(InplaceHeaderUpdateContext {
                inner,
                marker: PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as exr;
    use semver::VersionReq;
    use std::path::Path;

    use imath_traits::f16;
    use imath_traits::Bound2;

    #[test]
    fn read_scanline() -> Result<(), Box<dyn std::error::Error>> {
        let path_ferris = Path::new(
            &std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR not set"),
        )
        .join("images")
        .join("ferris.exr");

        let ctx = exr::context::ReadContext::new(&path_ferris)?;
        assert_eq!(ctx.file_name()?, path_ferris.to_str().unwrap());

        assert_eq!(ctx.count()?, 1);
        assert_eq!(ctx.name(0)?, None);
        assert_eq!(ctx.storage(0)?, exr::attr::Storage::Scanline);
        assert_eq!(ctx.tile_levels(0), Err(exr::Error::TileScanMixedApi));
        assert_eq!(ctx.tile_sizes(0, 0, 0), Err(exr::Error::TileScanMixedApi));

        let chunk_count = ctx.chunk_count(0)?;
        let scanlines_per_chunk = ctx.scanlines_per_chunk(0)?;
        let chunk_unpacked_size = ctx.chunk_unpacked_size(0)?;

        let attr_count = ctx.attribute_count(0)?;
        assert_eq!(attr_count, 8);

        for i in 0..attr_count {
            let attr = ctx.get_attribute_by_index(
                0,
                exr::part::AttrListAccessMode::FileOrder,
                i,
            )?;
            println!("Attribute {} - {}", i, attr.name());
        }

        let attr_channels = ctx.get_attribute_by_name(0, "channels")?;

        assert_eq!(ctx.get_attribute::<f32>(0, "screenWindowWidth")?, 1.0f32);
        assert_eq!(
            ctx.get_attribute::<exr::attr::Compression>(0, "compression")?,
            exr::attr::Compression::Piz
        );

        let dw = [0, 0, 1199, 799];
        assert_eq!(ctx.get_attribute::<[i32; 4]>(0, "dataWindow")?, dw);
        assert_eq!(ctx.get_attribute::<[i32; 4]>(0, "displayWindow")?, dw);

        assert_eq!(ctx.data_window::<[i32; 4]>(0)?, dw);
        assert_eq!(ctx.display_window::<[i32; 4]>(0)?, dw);

        assert_eq!(ctx.lineorder(0)?, exr::attr::LineOrder::IncreasingY);

        assert_eq!(ctx.pixel_aspect_ratio(0)?, 1.0f32);
        assert_eq!(ctx.screen_window_width(0)?, 1.0f32);
        assert_eq!(ctx.screen_window_center::<[f32; 2]>(0)?, [0.0, 0.0]);

        let width = dw.width() as usize + 1;
        let height = dw.height() as usize + 1;

        let channels_to_read = ["R", "G", "B", "A"];
        let nchan = channels_to_read.len();
        let channel_bytes = 2;
        let pixel_bytes = channels_to_read.len() * channel_bytes;
        let scanline_bytes = pixel_bytes * width;

        let num_chunk_lines = scanlines_per_chunk * chunk_count;

        // we need to make the storage big enough to hold all chunks, then we'll
        // truncate at the end. Alternative would be to allocate a chunk-sized
        // buffer and copy into the result vec as each chunk is decoded
        let mut pixel_data =
            vec![f16::from_f32(0.5); width * num_chunk_lines * nchan];

        println!("width: {}, height: {}", width, height);

        let mut chunk_scanline_start = 0;
        let mut chunk_scanline_end = scanlines_per_chunk;

        let chunk_info =
            ctx.read_scanline_chunk_info(0, chunk_scanline_start as i32)?;
        let mut decoder = exr::decode::DecodePipeline::default();

        ctx.decoding_initialize(0, &chunk_info, &mut decoder)?;

        while chunk_scanline_end <= num_chunk_lines {
            let pixel_ptr = pixel_data[chunk_scanline_start * width * nchan
                ..chunk_scanline_end * width * nchan]
                .as_mut_ptr();

            let chunk_info =
                ctx.read_scanline_chunk_info(0, chunk_scanline_start as i32)?;

            ctx.decoding_update(0, &chunk_info, &mut decoder)?;

            let mut chan_offset = 0;
            for req_chan_name in &channels_to_read {
                for decode_channel in decoder.channels_mut() {
                    if decode_channel.name() == *req_chan_name {
                        unsafe {
                            decode_channel.set_decode_to(
                                pixel_ptr.offset(chan_offset) as *mut u8,
                            );
                        }

                        decode_channel.set_user_bytes_per_element(2);
                        decode_channel.set_user_pixel_stride(pixel_bytes);
                        decode_channel.set_user_line_stride(scanline_bytes);
                        chan_offset += 1;
                    }
                }
            }

            ctx.decoding_choose_default_routines(0, &mut decoder)?;
            unsafe { ctx.decoding_run(0, &mut decoder)? };

            chunk_scanline_start += scanlines_per_chunk;
            chunk_scanline_end += scanlines_per_chunk;
        }

        // finished with the decoder, clean up
        ctx.decoding_destroy(decoder)?;

        // now truncate the pixels to the correct length and convert to u8
        // to write out a png for comparison
        let png_data = pixel_data
            .into_iter()
            .take(width * height * nchan)
            .map(|c| (f32::from(c) * 255.0).floor() as u8)
            .collect::<Vec<_>>();

        let f = std::fs::File::create("read_scanline.png")?;
        let ref mut w = std::io::BufWriter::new(f);
        let mut encoder = png::Encoder::new(w, width as u32, height as u32);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&png_data)?;

        Ok(())
    }
}
