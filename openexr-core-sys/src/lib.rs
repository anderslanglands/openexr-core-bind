#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
type size_t = usize;

use std::marker::PhantomData;

include!(concat!(env!("OUT_DIR"), "/openexr_wrapper.rs"));

#[repr(transparent)]
pub struct exr_result_t(i32);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Unable to allocate memory")]
    OutOfMemory,
    #[error("Context argument to function is not valid")]
    MissingContextArg,
    #[error("Invalid argument to function")]
    InvalidArgument,
    #[error("Argument to function out of valid range")]
    ArgumentOutOfRange,
    #[error("Unable to open file (path does not exist or permission denied)")]
    FileAccess,
    #[error("File is not an OpenEXR file or has a bad header value")]
    FileBadHeader,
    #[error("File not opened for read")]
    NotOpenRead,
    #[error("File not opened for write")]
    NotOpenWrite,
    #[error("File opened for write, but header not yet written")]
    HeaderNotWritten,
    #[error("Error reading from stream")]
    ReadIo,
    #[error("Error writing to stream")]
    WriteIo,
    #[error("Text too long for file flags")]
    NameTooLong,
    #[error("Missing required attribute in part header")]
    MissingReqAttr,
    #[error("Invalid attribute in part header")]
    InvalidAttr,
    #[error("No attribute by that name in part header")]
    NoAttrByName,
    #[error("Attribute type mismatch")]
    AttrTypeMismatch,
    #[error("Attribute type vs. size mismatch")]
    AttrSizeMismatch,
    #[error("Attempt to use a scanline accessor function for a tiled image")]
    ScanTileMixedApi,
    #[error("Attempt to use a tiled accessor function for a scanline image")]
    TileScanMixedApi,
    #[error(
        "Attempt to modify a value when in update mode with different size"
    )]
    ModifySizeChange,
    #[error("File in write mode, but header already written, can no longer edit attributes")]
    AlreadyWroteAttrs,
    #[error(
        "Unexpected or corrupt values in data block leader vs computed value"
    )]
    BadChunkLeader,
    #[error("Corrupt data block data, unable to decode")]
    CorruptChunk,
    #[error("Previous part not yet finished writing")]
    IncorrectPart,
    #[error("Invalid data block to write at this point")]
    IncorrectChunk,
    #[error("Use deep scanline write with the sample count table arguments")]
    UseScanDeepWrite,
    #[error("Use deep tile write with the sample count table arguments")]
    UseTileDeepWrite,
    #[error("Use non-deep scanline write (sample count table invalid for this part type)")]
    UseScanNonDeepWrite,
    #[error("Use non-deep tile write (sample count table invalid for this part type)")]
    UseTileNonDeepWrite,
    #[error("Invalid sample data table value")]
    InvalidSampleData,
    #[error("Feature not yet implemented, please use C++ library")]
    FeatureNotImplemented,
    #[error("Unknown error code")]
    Unknown,
}

impl exr_result_t {
    pub fn ok<T>(&self, val: T) -> Result<T, Error> {
        match self.0 as u32 {
            exr_error_code_t::EXR_ERR_SUCCESS => Ok(val),
            exr_error_code_t::EXR_ERR_OUT_OF_MEMORY => Err(Error::OutOfMemory),
            exr_error_code_t::EXR_ERR_MISSING_CONTEXT_ARG => {
                Err(Error::MissingContextArg)
            }
            exr_error_code_t::EXR_ERR_INVALID_ARGUMENT => {
                Err(Error::InvalidArgument)
            }
            exr_error_code_t::EXR_ERR_ARGUMENT_OUT_OF_RANGE => {
                Err(Error::ArgumentOutOfRange)
            }
            exr_error_code_t::EXR_ERR_FILE_ACCESS => Err(Error::FileAccess),
            exr_error_code_t::EXR_ERR_FILE_BAD_HEADER => {
                Err(Error::FileBadHeader)
            }
            exr_error_code_t::EXR_ERR_NOT_OPEN_READ => Err(Error::NotOpenRead),
            exr_error_code_t::EXR_ERR_NOT_OPEN_WRITE => {
                Err(Error::NotOpenWrite)
            }
            exr_error_code_t::EXR_ERR_HEADER_NOT_WRITTEN => {
                Err(Error::HeaderNotWritten)
            }
            exr_error_code_t::EXR_ERR_READ_IO => Err(Error::ReadIo),
            exr_error_code_t::EXR_ERR_WRITE_IO => Err(Error::WriteIo),
            exr_error_code_t::EXR_ERR_NAME_TOO_LONG => Err(Error::NameTooLong),
            exr_error_code_t::EXR_ERR_MISSING_REQ_ATTR => {
                Err(Error::MissingReqAttr)
            }
            exr_error_code_t::EXR_ERR_INVALID_ATTR => Err(Error::InvalidAttr),
            exr_error_code_t::EXR_ERR_NO_ATTR_BY_NAME => {
                Err(Error::NoAttrByName)
            }
            exr_error_code_t::EXR_ERR_ATTR_TYPE_MISMATCH => {
                Err(Error::AttrTypeMismatch)
            }
            exr_error_code_t::EXR_ERR_ATTR_SIZE_MISMATCH => {
                Err(Error::AttrSizeMismatch)
            }
            exr_error_code_t::EXR_ERR_SCAN_TILE_MIXEDAPI => {
                Err(Error::ScanTileMixedApi)
            }
            exr_error_code_t::EXR_ERR_TILE_SCAN_MIXEDAPI => {
                Err(Error::TileScanMixedApi)
            }
            exr_error_code_t::EXR_ERR_MODIFY_SIZE_CHANGE => {
                Err(Error::ModifySizeChange)
            }
            exr_error_code_t::EXR_ERR_ALREADY_WROTE_ATTRS => {
                Err(Error::AlreadyWroteAttrs)
            }
            exr_error_code_t::EXR_ERR_BAD_CHUNK_LEADER => {
                Err(Error::BadChunkLeader)
            }
            exr_error_code_t::EXR_ERR_CORRUPT_CHUNK => Err(Error::CorruptChunk),
            exr_error_code_t::EXR_ERR_INCORRECT_PART => {
                Err(Error::IncorrectPart)
            }
            exr_error_code_t::EXR_ERR_INCORRECT_CHUNK => {
                Err(Error::IncorrectChunk)
            }
            exr_error_code_t::EXR_ERR_USE_SCAN_DEEP_WRITE => {
                Err(Error::UseScanDeepWrite)
            }
            exr_error_code_t::EXR_ERR_USE_TILE_DEEP_WRITE => {
                Err(Error::UseTileDeepWrite)
            }
            exr_error_code_t::EXR_ERR_USE_SCAN_NONDEEP_WRITE => {
                Err(Error::UseScanNonDeepWrite)
            }
            exr_error_code_t::EXR_ERR_USE_TILE_NONDEEP_WRITE => {
                Err(Error::UseTileNonDeepWrite)
            }
            exr_error_code_t::EXR_ERR_INVALID_SAMPLE_DATA => {
                Err(Error::InvalidSampleData)
            }
            exr_error_code_t::EXR_ERR_FEATURE_NOT_IMPLEMENTED => {
                Err(Error::FeatureNotImplemented)
            }
            exr_error_code_t::EXR_ERR_UNKNOWN => Err(Error::Unknown),
            _ => panic!(
                "{}",
                format!("Unhandled value for exr_error_code_t: {}", self.0)
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v2i_t {
    x: i32,
    y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v2f_t {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v2d_t {
    x: f64,
    y: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v3i_t {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v3f_t {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(C, packed)]
pub struct exr_attr_v3d_t {
    x: f64,
    y: f64,
    z: f64,
}

#[cfg(test)]
mod tests {
    use crate as sys;

    #[test]
    fn it_works() {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        let mut extra = std::ptr::null();

        unsafe {
            sys::exr_get_library_version(
                &mut major, &mut minor, &mut patch, &mut extra,
            );
        }

        assert_eq!(major, 3);
        assert_eq!(minor, 1);
        assert_eq!(patch, 0);
    }
}
