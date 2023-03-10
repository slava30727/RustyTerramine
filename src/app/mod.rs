pub mod utils;

use {
    /* Other files */
    crate::app::utils::{
        cfg,
        concurrency::loading,
        user_io::InputManager,
        graphics::{
            self,
            Graphics,
            camera::Camera,
            texture::Texture,
            debug_visuals::{self, DebugVisualizedStatic},
        },
        terrain::chunk::{
            chunk_array::ChunkArray,
            ChunkDrawBundle,
        },
        time::timer::Timer,
        profiler,
        runtime::RUNTIME,
    },

    /* Glium includes */
    glium::{
        Surface,
        glutin::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        },
        uniform,
    },

    math_linear::prelude::*,
    std::sync::Mutex,
};

/// Struct that handles everything.
pub struct App {
    input_manager: InputManager,
    graphics: Graphics,
    camera: DebugVisualizedStatic<Camera>,
    timer: Timer,

    chunk_arr: DebugVisualizedStatic<ChunkArray>,
    chunk_draw_bundle: ChunkDrawBundle<'static>,

    texture_atlas: Texture,
}

impl App where Self: 'static {
    /// Constructs app struct.
    pub fn new() -> Self {
        let graphics = Graphics::new()
            .expect("failed to create graphics");

        let camera = DebugVisualizedStatic::new_camera(
            Camera::new()
                .with_position(0.0, 16.0, 2.0)
                .with_rotation(0.0, 0.0, std::f64::consts::PI),
            graphics.display.as_ref().get_ref(),
        );
    
        let texture_atlas = Texture::from_path("src/image/texture_atlas.png", graphics.display.as_ref().get_ref())
            .expect("path should be valid and file is readable");

        let chunk_draw_bundle = ChunkDrawBundle::new(graphics.display.as_ref().get_ref());
        let chunk_arr = DebugVisualizedStatic::new_chunk_array(
            ChunkArray::new_empty(),
            graphics.display.as_ref().get_ref(),
        );

        App {
            chunk_arr,
            chunk_draw_bundle,
            graphics,
            camera,
            texture_atlas,
            timer: Timer::new(),
            input_manager: InputManager::new(),
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) {
        // TODO: rewrite `loop { async {} }` into `async { loop {} }`.
        self.graphics.take_event_loop().run(move |event, _, control_flow| {
            RUNTIME.block_on(async {
                self.run_frame_loop(event, control_flow).await
            })
        });
    }

    /// Event loop run function.
    pub async fn run_frame_loop(&mut self, event: Event<'_, ()>, control_flow: &mut ControlFlow) {
        self.graphics.imguiw.handle_event(
            self.graphics.imguic.io_mut(),
            self.graphics.display.gl_window().window(),
            &event
        );
        self.input_manager.handle_event(&event, &self.graphics);

        match event {
            /* Window events */
            Event::WindowEvent { event, .. } => match event {
                /* Close event */
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    self.chunk_arr.drop_tasks();
                },

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    self.camera.aspect_ratio = height as f32
                                             / width  as f32;
                    self.graphics.on_window_resize(UInt2::new(width, height));
                },

                _ => (),
            },

            Event::MainEventsCleared =>
                self.main_events_cleared(control_flow).await,

            Event::RedrawRequested(_) =>
                self.redraw_requested().await,

            Event::NewEvents(_) =>
                self.new_events().await,

            _ => ()
        }
    }

    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        use glium::glutin::event::VirtualKeyCode as Key;
        
        /* Close window is `escape` pressed */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
            self.chunk_arr.drop_tasks();
        }

        if self.input_manager.keyboard.just_pressed(Key::Y) {
            self.chunk_arr.drop_tasks();
        }

        /* Control camera by user input */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.grabbes_cursor {
                self.input_manager.mouse.release_cursor(&self.graphics);
            } else {
                self.input_manager.mouse.grab_cursor(&self.graphics);
            }

            self.camera.grabbes_cursor = !self.camera.grabbes_cursor;
        }

        if self.input_manager.keyboard.just_pressed(Key::H) {
            self.chunk_draw_bundle = ChunkDrawBundle::new(self.graphics.display.as_ref().get_ref());
            self.graphics.refresh_postprocessing_shaders();
        }

        /* Update save/load tasks of `ChunkArray` */
        self.chunk_arr.update(&mut self.input_manager).await
            .expect("failed to update chunk array");

        let window = self.graphics.display.gl_window();
        let window = window.window();

        /* Display FPS */
        window.set_title(&format!("Terramine: {0:.0} FPS", self.timer.fps()));

        /* Update ImGui stuff */
        self.graphics.imguiw
            .prepare_frame(self.graphics.imguic.io_mut(), window)
            .expect("failed to prepare frame");

        /* Moves to `RedrawRequested` stage */
        window.request_redraw();
    }

    /// Prepares the frame.
    async fn redraw_requested(&mut self) {
        static LIGHT_DIRECTION: Mutex<vec3> = Mutex::new(vecf!(1.0, -1.0, 1.0));

        /* InGui draw data */
        let draw_data = {
            /* Get UI frame renderer */
            let ui = self.graphics.imguic.new_frame();

            /* Camera window */
            self.camera.spawn_control_window(ui, &mut self.input_manager);

            /* Profiler window */
            profiler::update_and_build_window(ui, &self.timer, &mut self.input_manager);

            /* Render UI */
            self.graphics.imguiw.prepare_render(ui, self.graphics.display.gl_window().window());

            /* Chunk array control window */
            self.chunk_arr.spawn_control_window(ui);

            /* Spawns loadings window */
            loading::spawn_info_window(ui);

            {
                // FIXME:
                let mut light = LIGHT_DIRECTION
                    .lock()
                    .expect("mutex should be not poisoned");

                ui.window("Light")
                    .always_auto_resize(true)
                    .movable(true)
                    .build(|| {
                        ui.slider("x", -1.0, 1.0, &mut light.x);
                        ui.slider("y", -1.0, 1.0, &mut light.y);
                        ui.slider("z", -1.0, 1.0, &mut light.z);
                    });

                *light = light.normalized();
            }

            self.graphics.imguic.render()
        };

        /* Uniforms set */
        let uniforms = uniform! {
            tex:  self.texture_atlas.with_mips(),
            time: self.timer.time(),
            proj: self.camera.get_proj(),
            view: self.camera.get_view(),
            light_dir: LIGHT_DIRECTION
                .lock()
                .expect("mutex should be not poisoned")
                .as_array(),
        };

        graphics::draw! {
            self.graphics,
            self.graphics.display.draw(),

            |mut frame_buffer| {
                self.chunk_arr.render_chunk_array(
                    &mut frame_buffer, &self.chunk_draw_bundle,
                    &uniforms, self.graphics.display.as_ref().get_ref(), &self.camera
                ).await
                    .expect("failed to render chunk array");
        
                self.camera.render_camera(self.graphics.display.as_ref().get_ref(), &mut frame_buffer, &uniforms)
                    .expect("failed to render camera");
            },

            |mut target| {
                self.graphics.imguir.render(&mut target, draw_data)
                    .expect("failed to render imgui");
            },

            uniform! {
                light_dir: LIGHT_DIRECTION
                    .lock()
                    .expect("mutex should be not poisoned")
                    .as_array(),
                time: self.timer.time(),
                cam_pos: self.camera.pos.as_array(),
            },
        };

        self.timer.update();
        self.graphics.imguic
            .io_mut()
            .update_delta_time(self.timer.duration());
    }

    /// Updates things.
    async fn new_events(&mut self) {
        /* Rotating camera */
        self.camera.update(&mut self.input_manager, self.timer.dt_as_f64());

        /* Debug visuals switcher */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
            debug_visuals::switch_enable();
        }

        /* Input update */
        self.input_manager.update(&self.graphics);		

        /* Loading recieve */
        loading::recv_all()
            .expect("failed to receive all loadings");
    }
}
