pub mod context;
pub mod error;
pub use error::Error;
pub mod attr;
pub mod part;
pub mod decode;
pub mod chunkio;
pub mod coding;

use openexr_core_sys as sys;
use semver::{BuildMetadata, Prerelease, Version};

pub fn get_library_version() -> Version {
    use std::ffi::CStr;
    let mut major = 0;
    let mut minor = 0;
    let mut patch = 0;
    let mut extra = std::ptr::null();

    unsafe {
        sys::exr_get_library_version(
            &mut major, &mut minor, &mut patch, &mut extra,
        );

        Version {
            major: major as u64,
            minor: minor as u64,
            patch: patch as u64,
            pre: Prerelease::new(CStr::from_ptr(extra).to_str().unwrap())
                .unwrap(),
            build: BuildMetadata::EMPTY,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as exr;
    use semver::VersionReq;
    use std::path::Path;

    #[test]
    fn it_works() {
        let ver = exr::get_library_version();
        println!("ver: {}", ver);
        assert!(VersionReq::parse("3.1.0--dev").unwrap().matches(&ver));
    }

    #[test]
    fn read_scanline() -> Result<(), exr::Error> {
        let path_ferris = Path::new(
            &std::env::var("CARGO_MANIFEST_DIR")
                .expect("CARGO_MANIFEST_DIR not set"),
        )
        .join("images")
        .join("ferris.exr");

        exr::context::ReadContext::new(&path_ferris)?;

        Ok(())
    }
}
