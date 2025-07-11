use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read from zip file")]
    Zip(#[from] zip::result::ZipError),
    #[error("Failed to read from zip file")]
    AsyncZip(#[from] async_zip::error::ZipError),
    #[error("I/O operation failed during extraction")]
    Io(#[from] std::io::Error),
    #[error(
        "The top-level of the archive must only contain a list directory, but it contains: {0:?}"
    )]
    NonSingularArchive(Vec<OsString>),
    #[error("The top-level of the archive must only contain a list directory, but it's empty")]
    EmptyArchive,
    #[error("Bad CRC (got {computed:08x}, expected {expected:08x}) for file: {}", path.display())]
    BadCrc32 {
        path: PathBuf,
        computed: u32,
        expected: u32,
    },
}

impl Error {
    /// Returns `true` if the error is due to the server not supporting HTTP streaming. Most
    /// commonly, this is due to serving ZIP files with features that are incompatible with
    /// streaming, like data descriptors.
    pub fn is_http_streaming_unsupported(&self) -> bool {
        matches!(
            self,
            Self::AsyncZip(async_zip::error::ZipError::FeatureNotSupported(_))
        )
    }

    /// Returns `true` if the error is due to HTTP streaming request failed.
    pub fn is_http_streaming_failed(&self) -> bool {
        match self {
            Self::AsyncZip(async_zip::error::ZipError::UpstreamReadError(_)) => true,
            Self::Io(err) => {
                if let Some(inner) = err.get_ref() {
                    inner.downcast_ref::<reqwest::Error>().is_some()
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
