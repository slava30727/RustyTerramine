use {
    crate::app::utils::{
        cfg,
        graphics::{
            camera::Camera,
            vertex_buffer::VertexBuffer,
            mesh::UnindexedMesh,
        },
    },
    super::*,
    glium::{
        Depth, DepthTest, BackfaceCullingMode, DrawError,
        index::PrimitiveType,
        uniforms::Uniforms,
    },
    std::sync::atomic::Ordering,
    lazy_static::lazy_static,
};

pub mod data {
    use super::*;

    static mut SHADER: Option<ShaderWrapper> = None;

    lazy_static! {
        static ref DRAW_PARAMS: DrawParametersWrapper<'static> = DrawParametersWrapper(
            DrawParameters {
                polygon_mode: glium::PolygonMode::Line,
                line_width: Some(2.0),
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    .. Default::default()
                },
                backface_culling: BackfaceCullingMode::CullingDisabled,
                .. Default::default()
            }
        );
    }

    pub fn get<'s>(facade: &dyn glium::backend::Facade) -> DebugVisualsStatics<'s, Camera> {
        cond_init(facade);
        get_unchecked()
    }

    pub fn get_unchecked<'s>() -> DebugVisualsStatics<'s, Camera> {
        unsafe {
            DebugVisualsStatics {
                shader: &SHADER.as_ref().expect("shader should be initialized").0,
                draw_params: &DRAW_PARAMS.0,
                _phantom: PhantomData
            }
        }
    }

    pub fn cond_init(facade: &dyn glium::backend::Facade) {
        unsafe {
            /* Check if uninitialized */
            if SHADER.is_none() {
                let shader = Shader::new("debug_lines", "debug_lines", facade);
                SHADER.replace(ShaderWrapper(shader));
            }
        }
    }

    pub fn construct_mesh(camera: &Camera, facade: &dyn glium::backend::Facade) -> UnindexedMesh<Vertex> {
        let color = [0.5; 4];
        let rays = camera.get_frustum().courner_rays;
        const LEN: f32 = cfg::camera::FRUSTUM_EDGE_LINE_LENGTH;
        let vertices = [
            Vertex { pos: [rays[0].origin.x, rays[0].origin.y, rays[0].origin.z], color },
            Vertex { pos: [
                rays[0].origin.x + LEN * rays[0].direction.x,
                rays[0].origin.y + LEN * rays[0].direction.y,
                rays[0].origin.z + LEN * rays[0].direction.z
            ], color },
            
            Vertex { pos: [rays[1].origin.x, rays[1].origin.y, rays[1].origin.z], color },
            Vertex { pos: [
                rays[1].origin.x + LEN * rays[1].direction.x,
                rays[1].origin.y + LEN * rays[1].direction.y,
                rays[1].origin.z + LEN * rays[1].direction.z
            ], color },
            
            Vertex { pos: [rays[2].origin.x, rays[2].origin.y, rays[2].origin.z], color },
            Vertex { pos: [
                rays[2].origin.x + LEN * rays[2].direction.x,
                rays[2].origin.y + LEN * rays[2].direction.y,
                rays[2].origin.z + LEN * rays[2].direction.z
            ], color },
            
            Vertex { pos: [rays[3].origin.x, rays[3].origin.y, rays[3].origin.z], color },
            Vertex { pos: [
                rays[3].origin.x + LEN * rays[3].direction.x,
                rays[3].origin.y + LEN * rays[3].direction.y,
                rays[3].origin.z + LEN * rays[3].direction.z
            ], color },
        ];

        let vbuffer = VertexBuffer::no_indices(facade, &vertices, PrimitiveType::LinesList);
        UnindexedMesh::new(vbuffer)
    }
}

impl DebugVisualized<'_, Camera> {
    pub fn new_camera(camera: Camera, facade: &dyn glium::backend::Facade) -> Self {
        let mesh = UnindexedMesh::new_empty(facade);
        Self { inner: camera, mesh, static_data: data::get(facade) }
    }

    pub fn render_camera(
        &self,
        facade: &dyn glium::backend::Facade,
        target: &mut impl glium::Surface,
        uniforms: &impl Uniforms
    ) -> Result<(), DrawError> {
        if ENABLED.load(Ordering::Relaxed) {
            let mesh = data::construct_mesh(&self.inner, facade);
            mesh.render(target, &self.static_data.shader, &self.static_data.draw_params, uniforms)
        } else { Ok(()) }
    }
}