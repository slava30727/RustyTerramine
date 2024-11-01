use crate::{
    prelude::*,
    graphics::{Mesh, SimpleMesh, Vertex, vertex_attr_array, VertexAttribute, MeshFlags},
    terrain::{
        chunk::{Chunk, Lod, array::{ChunkAdj, ChunkArray}, FillType, ChunkOption},
        voxel::{voxel_data::data::*, VoxelColor, Voxel, shape::{CubeDetailed, CubeLowered}},
    },
    iterator::CubeBoundary,
};



#[derive(Clone, Copy, PartialEq, Eq, Debug, IsVariant)]
pub enum Resolution {
    High,
    Low(Lod),
}
assert_impl_all!(Resolution: Send, Sync);

impl From<Lod> for Resolution {
    fn from(value: Lod) -> Self {
        match value {
            0 => Self::High,
            lod => Self::Low(lod),
        }
    }
}

impl From<Resolution> for Lod {
    fn from(value: Resolution) -> Self {
        match value {
            Resolution::High => 0,
            Resolution::Low(lod) => lod,
        }
    }
}



/// Full-detailed vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq, Default)]
pub struct HiResVertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub face_idx: u32,
}
assert_impl_all!(HiResVertex: Send, Sync);

impl HiResVertex {
    pub const fn new(position: Vec3, tex_coords: Vec2, face_idx: u32) -> Self {
        Self { position, tex_coords, face_idx }
    }
}

impl Vertex for HiResVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Uint32];
}



/// Low-detailed vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq, Default)]
pub struct LowResVertex {
    pub position: Vec3,
    pub color: Vec3,
    pub face_idx: u32,
}
assert_impl_all!(LowResVertex: Send, Sync);

impl LowResVertex {
    pub const fn new(position: Vec3, color: Vec3, face_idx: u32) -> Self {
        Self { position, color, face_idx }
    }
}

impl Vertex for LowResVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Uint32];
}



pub fn make(array: &ChunkArray) -> Mesh {
    ChunkArray::chunk_pos_range(array.size())
        .filter_map(|pos| {
            let (chunk, adj) = array.chunk_with_adj(pos);
            chunk.map(|chunk| make_high_resolution(&chunk, adj, MeshFlags::RENDERABLE))
        })
        .collect()
}

pub fn make_one(chunk: &Chunk, adj: ChunkAdj, lod: Lod, flags: MeshFlags) -> Mesh {
    match lod {
        0 => make_high_resolution(chunk, adj, flags),
        _ => make_low_resolution(chunk, adj, lod, flags),
    }
}

pub fn make_high_resolution(chunk: &Chunk, adj: ChunkAdj, flags: MeshFlags) -> Mesh {
    let is_filled_and_blocked
        = chunk.is_filled()
        && Chunk::is_adj_filled(&adj);

    if chunk.is_empty() || is_filled_and_blocked {
        return SimpleMesh::new_empty::<LowResVertex>(
            default(), default(), default(),
        ).into();
    }

    let info = chunk.info.load(Relaxed);
    let pos_iter: Box<dyn Iterator<Item = IVec3>> = match info.get_fill_type() {
        FillType::Unspecified =>
            Box::new(Chunk::local_pos_iter()),

        FillType::AllSame(id) => if id == AIR_VOXEL_DATA.id {
            Box::new(std::iter::empty())
        } else {
            Box::new(CubeBoundary::new(Chunk::SIZE as i32))
        },
    };

    let vertices = pos_iter
        .filter_map(|pos| match chunk.get_voxel_local(pos) {
            None => {
                logger::log!(Error, from = "chunk", "failed to get voxel from pos {pos}");
                None
            },
            some => some,
        })
        .filter(|voxel| !voxel.is_air())
        .flat_map(|voxel| {
            let side_iter = Range3d::adj_iter(IVec3::ZERO)
                .filter(|&offset| {
                    let adj_chunk = adj.by_offset(offset).unwrap();

                    match chunk.get_voxel_global(voxel.pos + offset) {
                        ChunkOption::Voxel(voxel) => voxel.is_air(),

                        ChunkOption::OutsideChunk => match adj_chunk {
                            None => true,

                            Some(chunk) => match chunk.get_voxel_global(voxel.pos + offset) {
                                ChunkOption::Voxel(voxel) => voxel.is_air(),
                                ChunkOption::OutsideChunk => true,
                                ChunkOption::Failed => {
                                    logger::log!(
                                        Error, from = "chunk",
                                        "caught on failed chunk voxel in {pos}",
                                        pos = voxel.pos + offset,
                                    );
                                    true
                                },
                            }
                        },

                        ChunkOption::Failed => {
                            logger::log!(
                                Error, from = "chunk",
                                "caught on failed chunk voxel in {pos}",
                                pos = voxel.pos + offset,
                            );
                            true
                        },
                    }
                });

            const N_CUBE_VERTICES: usize = 36;
            let mut vertices = SmallVec::<[_; N_CUBE_VERTICES]>::new();

            let mesh_builder = CubeDetailed::new(voxel.data);

            for offset in side_iter {
                mesh_builder.by_offset(offset, voxel.pos.as_vec3(), &mut vertices);
            }

            vertices
        })
        .collect_vec();
    
    SimpleMesh::new(vertices, None, default(), default(), flags).into()
}

pub fn make_low_resolution(chunk: &Chunk, adj: ChunkAdj, lod: Lod, flags: MeshFlags) -> Mesh {
    if lod == 0 {
        logger::error!(
            from = "chunk-mesh",
            "failed to make low resolution mesh with lod = 0, making high resolution mesh instead",
        );

        return make_high_resolution(chunk, adj, flags);
    }
    
    let is_filled_and_blocked = chunk.is_filled() && Chunk::is_adj_filled(&adj);

    if chunk.is_empty() || is_filled_and_blocked {
        return SimpleMesh::new_empty::<LowResVertex>(default(), default(), flags).into()
    }

    // TODO: optimize for same-filled chunks
    let sub_chunk_size = 1 << lod;
    let vertices = chunk.low_voxel_iter(lod)
        .filter_map(|(voxel, p)| match voxel {
            VoxelColor::Transparent => None,
            VoxelColor::Colored(color) => Some((color, p)),
        })
        .flat_map(|(voxel_color, local_low_pos)| {
            let local_pos = local_low_pos * sub_chunk_size;
            let global_pos = Chunk::local_to_global_pos(chunk.pos.load(Relaxed), local_pos);

            let center_pos = macros::formula!(
                pos + 0.5 * chunk_size - 0.5 * voxel_size, where
                pos = global_pos.as_vec3(),
                chunk_size = Vec3::splat(sub_chunk_size as f32) * Voxel::SIZE,
                voxel_size = 0.5 * Vec3::splat(Voxel::SIZE),
            );

            let is_blocking_voxel = |pos: IVec3, offset: IVec3| match chunk.get_voxel_global(pos) {
                ChunkOption::OutsideChunk => {
                    match adj.by_offset(offset).unwrap() {
                        // There is no chunk so voxel isn't blocked
                        None => false,
                        
                        Some(chunk) => match chunk.get_voxel_global(pos) {
                            ChunkOption::OutsideChunk => unreachable!("Can't fall out of an adjacent chunk"),
                            ChunkOption::Voxel(voxel) => !voxel.is_air(),
                            ChunkOption::Failed => {
                                logger::log!(Error, from = "chunk", "caught failed chunk voxel in {pos}");
                                false
                            },
                        },
                    }
                },

                ChunkOption::Voxel(voxel) => !voxel.is_air(),

                ChunkOption::Failed => {
                    logger::log!(Error, from = "chunk", "caught failed chunk voxel in {pos}");
                    false
                },
            };

            let is_blocked_subchunk = |offset: IVec3| -> bool {
                let start_pos = global_pos + offset * sub_chunk_size;
                let end_pos   = global_pos + (offset + IVec3::ONE) * sub_chunk_size;

                let is_on_surface = match offset.to_array() {
                    [-1, 0, 0] if 0 == local_pos.x => true,
                    [0, -1, 0] if 0 == local_pos.y => true,
                    [0, 0, -1] if 0 == local_pos.z => true,
                    [1, 0, 0] if Chunk::SIZE as i32 == local_pos.x + sub_chunk_size => true,
                    [0, 1, 0] if Chunk::SIZE as i32 == local_pos.y + sub_chunk_size => true,
                    [0, 0, 1] if Chunk::SIZE as i32 == local_pos.z + sub_chunk_size => true,
                    _ => false,
                };
                
                let mut iter = Range3d::from(start_pos..end_pos);
                let pred = |pos| is_blocking_voxel(pos, offset);

                if adj.by_offset(offset).unwrap().is_some() && is_on_surface {
                    iter.all(pred)
                } else {
                    iter.any(pred)
                }
            };

            let mesh_builder = CubeLowered::new(
                sub_chunk_size as f32 * Voxel::SIZE
            );
            
            const N_CUBE_VERTICES: usize = 36;
            let mut vertices = Vec::with_capacity(N_CUBE_VERTICES);

            for offset in Range3d::adj_iter(IVec3::ZERO).filter(|&o| !is_blocked_subchunk(o)) {
                mesh_builder.by_offset(offset, center_pos, voxel_color, &mut vertices);
            }

            vertices
        })
        .collect();

    SimpleMesh::new(vertices, None, default(), default(), flags).into()
}

pub fn insert(parent: &mut Mesh, node: Mesh, lod: Lod)
    -> Option<Mesh>
{
    if parent.is_simple() {
        logger::error!(
            from = "chunk-mesh",
            "failed to insert a simple mesh, only chained \
            mesh supported under chunk array, making tree mesh instead",
        );

        parent.to_tree();
    }

    let Mesh::Tree(meshes) = parent else { unreachable!() };

    Some(mem::replace(meshes.get_mut(lod as usize)?, node))
}