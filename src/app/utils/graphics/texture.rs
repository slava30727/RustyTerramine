use {
    crate::app::utils::logger,
    std::{io::{Cursor, self}, fs, path::{Path, PathBuf}},
    glium::{
        uniforms::SamplerWrapFunction,
        texture::{RawImage2d, Texture2d, MipmapsOption},
        uniforms::{Sampler, MagnifySamplerFilter, MinifySamplerFilter},
        backend::Facade
    },
};

/// Texture struct.
/// Contains texture stuff.
#[derive(Debug)]
pub struct Texture {
    pub path: PathBuf,
    pub inner: Texture2d,
}

impl Texture {
    /// Loads texture from path.
    pub fn from_path(path: impl AsRef<Path>, display: &dyn Facade) -> Result<Self, io::Error> {
        let _log_guard = logger::work!(from = "texture loader", "from {path:?}", path = path.as_ref());

        let path_buf = path.as_ref().to_owned();
        let image_bytes = fs::read(path)?;

        let image = image::load(Cursor::new(image_bytes), image::ImageFormat::Png)
            .expect("failed to load image")
            .to_rgba8();
        let image_size = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_size);
        
        let texture = Texture2d::with_mipmaps(
            display,
            image,
            MipmapsOption::AutoGeneratedMipmapsMax(4)
        ).expect("failed to add mipmaps to texture");

        Ok(Self {
            path: path_buf,
            inner: texture,
        })
    }

    /// Adds mips to texture uniform.
    pub fn with_mips(&self) -> Sampler<Texture2d> {
        Sampler::new(&self.inner)
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::NearestMipmapNearest)
            .wrap_function(SamplerWrapFunction::Clamp)
            .anisotropy(4)
    }
}