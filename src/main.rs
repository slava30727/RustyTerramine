#[macro_use]
extern crate glium;
extern crate image;
mod shader;
mod texture;
mod window;
mod graphics;
mod vertex_buffer;
mod camera;
mod user_io;

/* Glium includes */
use glium::Surface;

/* Other files */
use user_io::{Mouse, Keyboard, KeyCode};
use camera::Camera;
use shader::Shader;
use texture::Texture;
use window::Window;
use graphics::Graphics;
use vertex_buffer::VertexBuffer;

fn main() {
	/* Keyboard init */
	let mut keyboard = Keyboard::new();
	let mut mouse = Mouse::new();

	/* Graphics initialization */
	let mut graphics = Graphics::initialize().unwrap();

	/* Camera handle */
	let mut camera = Camera::new();

	/* Texture loading */
	let texture = Texture::from("src/image/testSprite.png", &graphics.display).unwrap();

	/* Vertex buffer loading */
	let vertex_buffer = VertexBuffer::default(&graphics);
	vertex_buffer.bind(&mut graphics);

	/* Shader program */
	let shaders = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
	graphics.upload_shaders(shaders);

	/* Temporary moves */
	let vertex_buffer = graphics.take_vertex_buffer();
	let indices = graphics.take_privitive_type();
	let shaders = graphics.take_shaders();

	/* Time stuff */
	let time_start = std::time::Instant::now();
	let mut _time = time_start.elapsed().as_secs_f32();
	
	/* Camera rotation */
	let mut roll: f32 = Default::default();
	let mut pitch: f32 = Default::default();

	camera.set_position(0.0, 0.0, 2.0);

	/* Event loop run */
	graphics.take_event_loop().run(move |event, _, control_flow| {
		/* Exit if window have that message */
		match Window::process_events(&event, &mut keyboard, &mut mouse) {
			window::Exit::Exit => {
				Window::exit(control_flow);
				return;
			},
			_ => ()
		}

		/* Close window is `escape` pressed */
		if keyboard.just_pressed(KeyCode::Escape) {
			Window::exit(control_flow);
		}

		/* Control camera by user input */
		if keyboard.is_pressed(KeyCode::W)		{ camera.move_pos( 0.001,  0.0,    0.0); }
		if keyboard.is_pressed(KeyCode::S)		{ camera.move_pos(-0.001,  0.0,    0.0); }
		if keyboard.is_pressed(KeyCode::D)		{ camera.move_pos( 0.0,    0.0,   -0.001); }
		if keyboard.is_pressed(KeyCode::A)		{ camera.move_pos( 0.0,    0.0,    0.001); }
		if keyboard.is_pressed(KeyCode::LShift)	{ camera.move_pos( 0.0,   -0.001,  0.0); }
		if keyboard.is_pressed(KeyCode::Space)	{ camera.move_pos( 0.0,    0.001,  0.0); }
		if mouse.just_left_pressed() {
			camera.set_position(0.0, 0.0, 2.0);
			camera.reset_rotation();
			roll = 0.0;
			pitch = 0.0;
		}

		pitch += mouse.dx / 100.0;
		roll -= mouse.dy / 100.0;

		/* Time refresh */
		_time = time_start.elapsed().as_secs_f32();

		/* Rotating camera */
		camera.set_rotation(roll, pitch, 0.0);
		mouse.update();

		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: texture.with_mips(),
			time: _time,
			proj: camera.get_proj(),
			view: camera.get_view()
		};

		/* Drawing process */
		let mut target = graphics.display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}
