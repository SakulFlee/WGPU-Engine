use wgpu::SurfaceError;
use winit::error::OsError;

#[derive(Debug)]
pub enum EngineError {
    NoAdapters,
    RequestDeviceError,
    CreateSurfaceError,
    NoMatch,
    SurfaceError(SurfaceError),
    ResourceMissing,
    IOError(std::io::Error),
    ImageError(image::ImageError),
    WinitOSError(OsError),
}
