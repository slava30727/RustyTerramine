use std::io::Cursor;
use std::fs;

/// Texture struct.
/// Contains texture stuff.
pub struct Texture {
	pub path: String,
	pub opengl_texture: glium::texture::SrgbTexture2d,
	pub width: u32,
	pub heigth: u32
}

impl Texture {
	/// Loads texture from path.
	pub fn from(path: &str, display: &glium::Display) -> Self {
		let image_bytes = fs::read(path).unwrap();

		let image = image::load(
			Cursor::new(image_bytes),
			image::ImageFormat::Png
		).unwrap().to_rgba8();
		let image_size = image.dimensions();
		let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_size);
		
		let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();

		Texture {
			path: String::from(path),
			opengl_texture: texture,
			width: image_size.0,
			heigth: image_size.1
		}
	}
	/// Adds mips to texture uniform.
	pub fn with_mips(&self) -> glium::uniforms::Sampler<glium::texture::SrgbTexture2d> {
		glium::uniforms::Sampler::new(&self.opengl_texture)
			.magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
			.minify_filter(glium::uniforms::MinifySamplerFilter::LinearMipmapNearest)
	}
}