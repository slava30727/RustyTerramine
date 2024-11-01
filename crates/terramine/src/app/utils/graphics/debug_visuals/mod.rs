// pub mod camera;
// pub mod chunk_array;



// use {
//     crate::{
//         prelude::*,
//         graphics::{
//             glium_mesh::UnindexedMesh,
//             glium_shader::Shader,
//         }
//     },
//     glium::{
//         DrawParameters,
//         implement_vertex,
//     },
//     std::marker::PhantomData,
// };



// /// Adds debug visuals to type `T`.
// #[derive(Debug, Deref)]
// pub struct DebugVisualized<'s, T> {
//     #[deref]
//     pub inner: T,
    
//     pub mesh: UnindexedMesh<Vertex>,
//     pub static_data: DebugVisualsStatics<'s, T>,
// }



// /// [`DebugVisualized`] with `'static` lifetime of debug visuals.
// pub type DebugVisualizedStatic<T> = DebugVisualized<'static, T>;



// #[derive(Debug)]
// pub struct DebugVisualsStatics<'s, T> {
//     pub shader: &'s Shader,
//     pub draw_params: &'s DrawParameters<'s>,

//     _phantom: PhantomData<fn() -> T>
// }



// static ENABLED: AtomicBool = AtomicBool::new(false);

// pub fn switch() {
//     ENABLED.fetch_update(AcqRel, Relaxed, |old| Some(!old)).unwrap();
// }

// pub fn update() {
//     if keyboard::just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
//         self::switch();
//     }
// }



// #[derive(Clone, Copy, PartialEq, Debug)]
// pub struct Vertex {
//     pos: [f32; 3],
//     color: [f32; 4],
// }
// assert_impl_all!(Vertex: Send, Sync);

// implement_vertex!(Vertex, pos, color);



// #[repr(transparent)]
// #[derive(Debug, Deref)]
// struct ShaderWrapper(Shader);

// unsafe impl Send for ShaderWrapper { }
// unsafe impl Sync for ShaderWrapper { }



// #[repr(transparent)]
// #[derive(Debug, Deref)]
// struct DrawParametersWrapper<'a>(DrawParameters<'a>);

// unsafe impl<'a> Send for DrawParametersWrapper<'a> { }
// unsafe impl<'a> Sync for DrawParametersWrapper<'a> { }