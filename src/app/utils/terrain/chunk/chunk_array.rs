use super::{Chunk, ChunkEnvironment as ChunkEnv};
use crate::app::utils::{
	graphics::Graphics,
	math::prelude::{*, rges::*},
	graphics::camera::Camera,
	reinterpreter::{
		StaticSize,
		ReinterpretAsBytes,
		ReinterpretFromBytes
	},
	saves::*,
};
use glium::{
	uniforms::Uniforms,
	DrawError,
	Frame
};
use std::{fs::File, os::windows::prelude::FileExt};

/// Represents self-controlling chunk array.
/// * Width is bigger if you go to x+ direction
/// * Height is bigger if you go to y+ direction
/// * Depth is bigger if you go to z+ direction
#[allow(dead_code)]
pub struct ChunkArray<'a> {
	/* Size */
	width:	usize,
	height:	usize,
	depth:	usize,

	/* Chunk array itself */
	chunks: Vec<Chunk<'a>>,
}

#[derive(Clone, Copy)]
enum SaveType {
	Width,
	Height,
	Depth,
	_ChunkArray,
}

impl Into<Offset> for SaveType {
	fn into(self) -> Offset { self as Offset }
}

impl<'a> ChunkArray<'a> {
	pub fn new(graphics: &Graphics, width: usize, height: usize, depth: usize) -> Self {
		/* Amount of voxels in chunks */
		let volume = width * height * depth;

		/* Initialize vector */
		let mut chunks = Vec::<Chunk>::with_capacity(volume);

		/* Name of world file */
		let filename = "src/world.chunks";
		let (path, name) = ("src/world", "world.chunks");

		let _save = Save::new(name)
			.create(path)
			.write(&width, SaveType::Width)
			.write(&height, SaveType::Height)
			.write(&depth, SaveType::Depth)
			.save().unwrap();

		{
		/* Offset of world bytes */
		let chunk_table_offset = usize::static_size() * 3;
		let chunk_heap_offset = chunk_table_offset + u64::static_size() * volume;

		let mut generate_file = || {
			/* World file */
			let file = File::create(filename).expect(format!("Failed to create file {filename} in write-only mode!").as_str());
			file.set_len((Chunk::static_size() * volume + chunk_heap_offset) as u64).expect("Failed to set file size!");

			/* Write width, height and depth to file */
			file.seek_write(&width.reinterpret_as_bytes(), 0).unwrap();
			file.seek_write(&height.reinterpret_as_bytes(), usize::static_size() as u64).unwrap();
			file.seek_write(&depth.reinterpret_as_bytes(), (usize::static_size() * 2) as u64).unwrap();

			/* Fill vector with chunks with no mesh attached */
			for x in (0..width).center() {
			for y in (0..height).center() {
			for z in (0..depth).center() {
				/* Local index function */
				let index = |mut x: isize, mut y: isize, mut z: isize| -> usize {
					/* Conversion to [0; dim(p) - 1] */
					x -= (0..width) .center().start;
					y -= (0..height).center().start;
					z -= (0..depth) .center().start;

					/* Index */
					sdex::get_index(&[x as usize, y as usize, z as usize], &[width, height, depth])
				};

				/* Generate chunk */
				let chunk = Chunk::new(None, Int3::new(x as i32, y as i32, z as i32), false);

				/* Write it to file */
				let current_offset = (index(x, y, z) * Chunk::static_size() + chunk_heap_offset) as u64;
				file.seek_write(&chunk.reinterpret_as_bytes(), current_offset).unwrap();
				file.seek_write(&current_offset.reinterpret_as_bytes(), (index(x, y, z) * u64::static_size() + chunk_table_offset) as u64).unwrap();

				/* Push it to chunk array */
				chunks.push(chunk);
			}}}
		};

		if std::path::Path::new(filename).exists() {
			/* World file */
			let file = File::open(filename).expect(format!("Failed to open file {filename} in read-only mode!").as_str());

			/* Read dimensions of world */
			let mut bytes = vec![0; usize::static_size()];

			/* Width */
			file.seek_read(&mut bytes, 0).unwrap();
			let read_width = usize::reinterpret_from_bytes(&bytes);

			/* Height */
			file.seek_read(&mut bytes, (usize::static_size()) as u64).unwrap();
			let read_height = usize::reinterpret_from_bytes(&bytes);

			/* Depth */
			file.seek_read(&mut bytes, (usize::static_size() * 2) as u64).unwrap();
			let read_depth = usize::reinterpret_from_bytes(&bytes);

			/* Size changed => regenerate world */
			if read_width == width && read_height == height && read_depth == depth {
				/* Current byte pointer */
				let mut current = chunk_table_offset as u64;
	
				/* Bytes buffer */
				let mut buffer = vec![0; Chunk::static_size()];
				let mut chunk_offset_buffer = vec![0; u64::static_size()];
	
				while current <= ((volume - 1) * u64::static_size() + chunk_table_offset) as u64 {
					/* Read chunk offset from table */
					file.seek_read(&mut chunk_offset_buffer, current).unwrap();
					let chunk_offset = u64::reinterpret_from_bytes(&chunk_offset_buffer);

					/* Read exact bytes for one chunk */
					file.seek_read(&mut buffer, chunk_offset).unwrap();
	
					/* Push chunk to array */
					chunks.push(Chunk::reinterpret_from_bytes(&buffer));
	
					/* Increment current pointer */
					current += u64::static_size() as u64;
				}
			} else {
				generate_file();
			}
		} else {
			generate_file();
		}
		}

		/* Make environments with references to chunk array */
		let env = Self::make_environment(&chunks, width, height, depth);

		/* Create mesh for each chunk */
		chunks.iter().zip(env.iter())
			.for_each(|(chunk, env)| chunk.update_mesh(&graphics, env));

		ChunkArray { width, height, depth, chunks }
	}

	/// Creates environment for ChunkArray
	fn make_environment<'v, 'c>(chunks: &'v Vec<Chunk<'c>>, width: usize, height: usize, depth: usize) -> Vec<ChunkEnv<'c>> {
		let mut env = vec![ChunkEnv::none(); width * height * depth];

		for x in 0..width {
		for y in 0..height {
		for z in 0..depth {
			/* Local index function */
			let index = |x, y, z| sdex::get_index(&[x, y, z], &[width, height, depth]);

			/* Reference to current environment variable */
			let env = &mut env[index(x, y, z)];

			/* For `front` side */
			if x as isize - 1 >= 0 {
				env.front	= Some(&chunks[index(x - 1, y, z)]);
			}

			/* For `back` side */
			if x + 1 < width {
				env.back	= Some(&chunks[index(x + 1, y, z)]);
			}

			/* For `bottom` side */
			if y as isize - 1 >= 0 {
				env.bottom	= Some(&chunks[index(x, y - 1, z)]);
			}
		
			/* For `top` side */
			if y + 1 < height {
				env.top		= Some(&chunks[index(x, y + 1, z)]);
			}

			/* For `left` side */
			if z as isize - 1 >= 0 {
				env.left	= Some(&chunks[index(x, y, z - 1)]);
			}

			/* For `right` side */
			if z + 1 < depth {
				env.right	= Some(&chunks[index(x, y, z + 1)]);
			}
		}}}

		return env;
	}

	/// Renders chunks.
	pub fn render<U: Uniforms>(&mut self, target: &mut Frame, uniforms: &U, camera: &Camera) -> Result<(), DrawError> {
		/* Iterating through array */
		for chunk in self.chunks.iter_mut() {
			chunk.render(target, uniforms, camera)?
		}
		Ok(())
	}
}