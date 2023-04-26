pub mod glium_shader;
pub mod glium_texture;
pub mod camera;
pub mod glium_mesh;
pub mod debug_visuals;
pub mod ui;
pub mod light;
pub mod surface;
pub mod failed_mesh;
pub mod failed_shader;
pub mod failed_texture;
pub mod mesh;
pub mod render_resource;
pub mod pipeline;
pub mod pass;
pub mod gpu_conversions;
pub mod material;

use {
    crate::{
        prelude::*,
        window::Window,
    },
    failed_mesh::{Mesh, Bufferizable, MeshDescriptor, Renderable},
    failed_shader::Shader, failed_texture::Texture,
    wgpu::{*, util::DeviceExt},
    winit::event_loop::EventLoop,
    std::path::PathBuf,
};

pub use {
    material::*, gpu_conversions::*, pass::{*, RenderPass},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default, Pod, Zeroable)]
pub struct TestVertex {
    position: vec2,
    tex_coords: vec2,
}
assert_impl_all!(TestVertex: Send, Sync);

impl Bufferizable for TestVertex {
    const ATTRS: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: Self::ATTRS,
    };
}

const TEST_VERTICES: &[TestVertex] = &[
    TestVertex { position: vecf![-0.5, -0.5], tex_coords: vecf![0.0, 1.0] },
    TestVertex { position: vecf![ 0.5, -0.5], tex_coords: vecf![1.0, 1.0] },
    TestVertex { position: vecf![ 0.5,  0.5], tex_coords: vecf![1.0, 0.0] },
    TestVertex { position: vecf![-0.5, -0.5], tex_coords: vecf![0.0, 1.0] },
    TestVertex { position: vecf![ 0.5,  0.5], tex_coords: vecf![1.0, 0.0] },
    TestVertex { position: vecf![-0.5,  0.5], tex_coords: vecf![0.0, 0.0] },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CommonUniforms {
    pub screen_resolution: vec2,
    pub time: f32,
    pub _pad: u32,
}
assert_impl_all!(CommonUniforms: Send, Sync);

#[derive(Debug)]
pub struct CommonUniformsBuffer {
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub bind_group: BindGroup,
    pub buffer: Buffer,
}
assert_impl_all!(CommonUniformsBuffer: Send, Sync);

impl CommonUniformsBuffer {
    pub fn new(device: &Device, initial_value: &CommonUniforms) -> Self {
        let buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some("common_uniforms_buffer"),
                contents: bytemuck::bytes_of(initial_value),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );

        let layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("common_uniforms_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );

        let bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("common_uniforms_bind_group"),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            },
        );

        Self { bind_group_layout: Arc::new(layout), bind_group, buffer }
    }

    pub fn update(&self, queue: &Queue, uniforms: CommonUniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]));
        queue.submit(std::iter::empty());
    }
}

#[derive(Debug)]
pub struct RenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Adapter,
    pub surface: Surface,
}

/// Graphics handler.
#[derive(Debug)]
pub struct Graphics {
    pub window: Window,
    pub context: RenderContext,
    pub config: SurfaceConfiguration,

    pub common_uniforms: CommonUniformsBuffer,
    
    pub test_texture: Texture,
    pub test_mesh: Mesh<TestVertex>,

    pub imgui: ImGui,
}
assert_impl_all!(Graphics: Send, Sync);

impl Graphics {
    /// Creates new [`Graphics`] that holds some renderer stuff.
    pub async fn new(event_loop: &EventLoop<()>) -> Result<Self, winit::error::OsError> {
        let _log_guard = logger::work("graphics", "initialization");

        const DEFAULT_SIZES: USize2 = cfg::window::default::SIZES;
        
        // Window creation
        let window = Window::from(event_loop, DEFAULT_SIZES)?;

        // ------------ WGPU initialization ------------

        let wgpu_instance = Instance::new(
            InstanceDescriptor {
                backends: Backends::DX12 | Backends::VULKAN,
                dx12_shader_compiler: default(),
            }
        );

        // # Safety
        //
        // `Graphics` owns both the `window` and the `surface` so it
        // lives as long as wgpu's `Surface`.
        let surface = unsafe {
            wgpu_instance.create_surface(&*window)
                .expect("context should be not WebGL2")
        };

        let adapter = wgpu_instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            })
            .await
            .expect("failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: default(),
            }, None)
            .await
            .expect("failed to create device");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *swapchain_capabilities.formats.get(0)
            .expect("failed to get swap chain format 0: the surface is incompatible with the adapter");
        
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: DEFAULT_SIZES.x as u32,
            height: DEFAULT_SIZES.y as u32,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // ------------ Renderng tests stuff ------------

        let test_texture = Texture::load_from_file(
            Arc::clone(&device),
            Arc::clone(&queue),
            "TerramineIcon32p.png",
            "test_texture",
            0, 1,
        ).await
            .expect("failed to load an image");

        let common_uniforms = CommonUniformsBuffer::new(
            &device,
            &CommonUniforms { time: 0.0, screen_resolution: vec2::from(DEFAULT_SIZES), _pad: 0 },
        );

        let shader = Shader::load_from_file(Arc::clone(&device), "triangle shader", "shader.wgsl")
            .await
            .expect("failed to load shader from file");

        let mesh = Mesh::new(
            MeshDescriptor {
                device: Arc::clone(&device),
                shader: Arc::new(shader),
                label: Arc::new(String::from("test mesh")),

                fragment_targets: Arc::new([Some(ColorTargetState {
                    // TODO: think about how transfer config.format everywhere.
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })]),

                primitive_topology: PrimitiveTopology::TriangleList,
                polygon_mode: PolygonMode::Fill,

                // TODO: remind this ->
                bind_group_layouts: Arc::new([
                    Arc::clone(&common_uniforms.bind_group_layout),
                    Arc::clone(&test_texture.bind_group_layout),
                ]),
            },
            TEST_VERTICES,
        );

        // ------------ Dear ImGui initialization ------------

        // Create ImGui context and set `.ini` file name.
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(Some(PathBuf::from("src/imgui_settings.ini")));

        // Bind ImGui to winit.
        let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(imgui_context.io_mut(), &window, imgui_winit_support::HiDpiMode::Rounded);

        // Style configuration.
        imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
        imgui_context.style_mut().window_rounding = 16.0;

        // Create ImGui renderer.
        let imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui_context,
            &device,
            &queue,
            imgui_wgpu::RendererConfig {
                texture_format: config.format,
                ..default()
            },
        );

        Ok(Self {
            test_mesh: mesh,
            window,
            context: RenderContext {
                device,
                queue,
                adapter,
                surface,
            },
            config,
            common_uniforms,
            test_texture,
            imgui: ImGui {
                context: imgui_context,
                platform: winit_platform,
                renderer: ImGuiRendererWrapper(imgui_renderer),
            },
        })
    }

    pub async fn refresh_test_shader(&mut self) {
        let shader = Shader::load_from_file(
            Arc::clone(&self.context.device),
            "test_shader",
            "shader.wgsl",
        ).await;

        match shader {
            Ok(shader) => self.test_mesh.reload_shader(Arc::new(shader)),
            Err(err) => logger::log!(Error, from = "graphics", "failed to reload test shader: {err}"),
        }
    }

    pub fn render<UseUi: FnOnce(&mut imgui::Ui)>(
        &mut self, desc: RenderDescriptor<UseUi>,
    ) -> Result<(), SurfaceError> {
        let size = self.window.inner_size();
        self.common_uniforms.update(&self.context.queue, CommonUniforms {
            time: desc.time,
            screen_resolution: (size.width as f32, size.height as f32).into(),
            _pad: 0,
        });

        let output = self.context.surface.get_current_texture()?;
        let view = output.texture.create_view(&default());
        let mut encoder = self.context.device.create_command_encoder(&default());

        let _ = ClearPass::new(&mut encoder, [&view]);

        {
            let mut pass = RenderPass::new(&mut encoder, "logo_draw_pass", [&view]);

            pass.set_bind_group(0, &self.common_uniforms.bind_group, &[]);
            pass.set_bind_group(1, &self.test_texture.bind_group, &[]);
            
            let Ok(()) = self.test_mesh.render(&mut pass);
        }

        {
            let mut render_pass = RenderPass::new(&mut encoder, "imgui_draw_pass", [&view]);

            let ui = self.imgui.context.new_frame();
            (desc.use_imgui_ui)(ui);

            self.imgui.platform.prepare_render(ui, &self.window);

            let draw_data = self.imgui.context.render();
            self.imgui.renderer.render(draw_data, &self.context.queue, &self.context.device, &mut render_pass)
                .expect("failed to render imgui");
        }
    
        self.context.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }

    pub fn on_window_resize(&mut self, new_size: UInt2) {
        if new_size.x > 0 && new_size.y > 0 {
            (self.config.width, self.config.height) = (new_size.x, new_size.y);
            self.context.surface.configure(&self.context.device, &self.config);
        }
    }
}

/// imgui-wgpu uses only wgpu stuff that are Send and Sync
/// but imgui context is not Send nor Sync. Use carefully.
#[derive(Debug)]
pub struct ImGui {
    // ImGui context.
    pub context: imgui::Context,

    // ImGui winit support.
    pub platform: imgui_winit_support::WinitPlatform,

    // ImGui WGPU renderer.
    pub renderer: ImGuiRendererWrapper,
}

// imgui-wgpu uses only wgpu stuff that are Send and Sync
// but imgui context is not Send nor Sync. Use carefully.
unsafe impl Send for ImGui { }
unsafe impl Sync for ImGui { }

#[derive(Deref)]
pub struct ImGuiRendererWrapper(imgui_wgpu::Renderer);

impl std::fmt::Debug for ImGuiRendererWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "imgui_Renderer {{ ... }}")
    }
}

#[derive(Debug)]
pub struct RenderDescriptor<UseImguiUi> {
    pub use_imgui_ui: UseImguiUi,
    pub time: f32,
}
