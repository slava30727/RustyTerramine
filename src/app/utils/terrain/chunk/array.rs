use { crate::{prelude::*, iterator::Sides}, super::Chunk, std::ptr::NonNull };



#[derive(Debug, Default)]
pub struct ChunkBorrowInfo {
    n_borrows: AtomicUsize,
}

impl ChunkBorrowInfo {
    pub const UNIQUE_BORROW: usize = usize::MAX;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_free(&self) -> bool {
        self.n_borrows.load(Acquire) == 0
    }

    pub fn is_unique(&self) -> bool {
        self.n_borrows.load(Acquire) == Self::UNIQUE_BORROW
    }

    pub fn is_shared(&self) -> bool {
        !self.is_unique() && self.n_borrows.load(Acquire) > 0
    }

    pub fn shared_count(&self) -> usize {
        if self.is_shared() {
            self.n_borrows.load(Acquire)
        } else { 0 }
    }

    pub fn borrow_mut(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            (value == 0).then_some(Self::UNIQUE_BORROW)
        }).is_ok()
    }

    pub fn borrow(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            (!self.is_unique()).then_some(value + 1)
        }).is_ok()
    }

    pub fn free(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |value| {
            self.is_shared().then_some(value - 1)
        }).is_ok()
    }

    pub fn free_mut(&self) -> bool {
        self.n_borrows.fetch_update(AcqRel, Relaxed, |_| {
            self.is_unique().then_some(0)
        }).is_ok()
    }
}

impl Clone for ChunkBorrowInfo {
    fn clone(&self) -> Self {
        Self { n_borrows: AtomicUsize::new(self.n_borrows.load(Relaxed)) }
    }
}



#[derive(Debug)]
pub struct ChunkArray {
    pub(crate) ptr: NonNull<ChunkArrayBox>,
}
assert_impl_all!(ChunkArray: Send, Sync, Component);

unsafe impl Send for ChunkArray { }
unsafe impl Sync for ChunkArray { }

impl ChunkArray {
    /// Constructs new [`ChunkArray`].
    pub fn new() -> Self {
        Self::from(ChunkArrayBox {
            sizes: USize3::ZERO,
            owned: AtomicBool::new(true),
            borrow_map: vec![],
            chunks: vec![],
        })
    }

    /// Constructs new [`ChunkArray`] with empty chunks
    pub fn new_empty(sizes: USize3) -> Self {
        Self::from(ChunkArrayBox {
            sizes,
            owned: AtomicBool::new(true),
            borrow_map: vec![ChunkBorrowInfo::new(); sizes.volume()],
            chunks: ChunkArray::chunk_pos_range(sizes)
                .map(Chunk::new_empty)
                .collect()
        })
    }

    fn allocate(array: ChunkArrayBox) -> NonNull<ChunkArrayBox> {
        // FIXME:
        eprintln!("ChunkArrayBox allocated");

        Box::leak(Box::new(array)).into()
    }

    /// Computes start and end poses from chunk array sizes.
    pub fn pos_bounds(sizes: USize3) -> (Int3, Int3) {
        (
            Self::volume_index_to_chunk_pos(sizes, USize3::ZERO),
            Self::volume_index_to_chunk_pos(sizes, sizes),
        )
    }

    /// Gives iterator over chunk coordinates.
    pub fn chunk_pos_range(sizes: USize3) -> Range3d {
        let (start, end) = Self::pos_bounds(sizes);
        Range3d::from(start..end)
    }

    /// Convertes global voxel position to 3d-index of a chunk in the array.
    pub fn global_voxel_pos_to_volume_index(
        voxel_pos: Int3, chunk_array_sizes: USize3
    ) -> Option<USize3> {
        let chunk_pos = Chunk::global_to_local(voxel_pos);
        let local_voxel_pos = Chunk::global_to_local_pos(chunk_pos, voxel_pos);

        let chunk_coord_idx
            = Self::local_pos_to_volume_index(chunk_array_sizes, chunk_pos)?;

        let voxel_offset_by_chunk: USize3
            = Chunk::local_to_global(chunk_coord_idx.into()).into();

        Some(voxel_offset_by_chunk + USize3::from(local_voxel_pos))
    }

    /// Convertes 3d-index of a chunk in the array to chunk pos.
    pub fn volume_index_to_chunk_pos(sizes: USize3, coord_idx: USize3) -> Int3 {
        Int3::from(coord_idx) - Int3::from(sizes) / 2
    }

    /// Convertes chunk pos to 3d index.
    pub fn local_pos_to_volume_index(sizes: USize3, pos: Int3)
        -> Option<USize3>
    {
        let sizes = Int3::from(sizes);
        let shifted = pos + sizes / 2;

        (
            0 <= shifted.x && shifted.x < sizes.x &&
            0 <= shifted.y && shifted.y < sizes.y &&
            0 <= shifted.z && shifted.z < sizes.z
        ).then_some(shifted.into())
    }

    /// Convertes 3d index to an array index.
    pub fn volume_index_to_linear(sizes: USize3, coord_idx: USize3) -> usize {
        sdex::get_index(&coord_idx.as_array(), &sizes.as_array())
    }

    /// Convertes array index to 3d index.
    pub fn linear_index_to_volume(idx: usize, sizes: USize3) -> USize3 {
        iterator::linear_index_to_volume(idx, sizes)
    }

    /// Converts array index to chunk pos.
    pub fn index_to_pos(idx: usize, sizes: USize3) -> Int3 {
        let coord_idx = Self::linear_index_to_volume(idx, sizes);
        Self::volume_index_to_chunk_pos(sizes, coord_idx)
    }

    /// Convertes chunk position to its linear index
    pub fn chunk_pos_to_linear_index(sizes: USize3, pos: Int3) -> Option<usize> {
        let coord_idx = Self::local_pos_to_volume_index(sizes, pos)?;
        Some(Self::volume_index_to_linear(sizes, coord_idx))
    }
}

impl From<ChunkArrayBox> for ChunkArray {
    fn from(value: ChunkArrayBox) -> Self {
        Self { ptr: Self::allocate(value) }
    }
}

impl Default for ChunkArray {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ChunkArray {
    fn drop(&mut self) {
        unsafe {
            self.ptr.as_mut().unown();
        }
    }
}

impl Deref for ChunkArray {
    type Target = ChunkArrayBox;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for ChunkArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}



#[derive(Debug)]
pub struct ChunkRef {
    pub(crate) parent: NonNull<ChunkArrayBox>,
    pub(crate) index: usize,
}

unsafe impl Send for ChunkRef { }

impl ChunkRef {
    /// Creates new shared reference to chunk
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArrayBox`]
    pub unsafe fn new(parent: NonNull<ChunkArrayBox>, index: usize) -> Self {
        // FIXME:
        eprintln!("ChunkRef constructed");

        Self { parent, index }
    }
}

impl Drop for ChunkRef {
    fn drop(&mut self) {
        // FIXME:
        eprintln!("ChunkRef destroyed");

        let array = unsafe { self.parent.as_mut() };

        assert!(
            unsafe { array.chunk_shared_free(self.index) },
            "failed to release shared chunk reference",
        );

        unsafe { array.try_deallocate() };
    }
}

impl Deref for ChunkRef {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_ref()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}

impl Clone for ChunkRef {
    fn clone(&self) -> Self {
        unsafe {
            let array = self.parent.as_ref();
            array.borrow_map[self.index].borrow();
            
            Self::new(self.parent, self.index)
        }
    }
}



#[derive(Debug)]
pub struct ChunkMut {
    pub(crate) parent: NonNull<ChunkArrayBox>,
    pub(crate) index: usize,
}

unsafe impl Send for ChunkMut { }

impl ChunkMut {
    /// Creates new unique chunk reference
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArrayBox`]
    pub unsafe fn new(parent: NonNull<ChunkArrayBox>, index: usize) -> Self {
        // FIXME:
        eprintln!("ChunkMut constructed");

        Self { parent, index }
    }
}

impl Drop for ChunkMut {
    fn drop(&mut self) {
        // FIXME:
        eprintln!("ChunkMut destroyed");

        let array = unsafe { self.parent.as_mut() };

        assert!(
            unsafe { array.chunk_mut_free(self.index) },
            "failed to free unique chunk reference",
        );

        unsafe { array.try_deallocate(); }
    }
}

impl Deref for ChunkMut {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_ref()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}

impl DerefMut for ChunkMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let array = self.parent.as_ref();
            let chunk = array.get_chunk_unchecked(self.index)
                .unwrap_or_else(|| panic!("failed to find chunk at index {}", self.index));

            chunk.as_mut()
                .unwrap_or_else(|| panic!("there is no chunk at index {}", self.index))
        }
    }
}



#[derive(Debug)]
pub struct ChunkArrayBox {
    pub sizes: USize3,
    pub owned: AtomicBool,
    pub borrow_map: Vec<ChunkBorrowInfo>,
    pub chunks: Vec<Chunk>,
}
assert_impl_all!(ChunkArrayBox: Send, Sync);

impl ChunkArrayBox {
    /// Gives chunk pointer without checks
    /// 
    /// # Safety
    /// 
    /// - no one can borrow this chunk uniquely
    /// - pointer destroyed before [`ChunkArray`] dropped
    pub unsafe fn get_chunk_unchecked(&self, index: usize) -> Option<*mut Chunk> {
        Some(self.chunks.get(index)? as *const _ as *mut _)
    }

    /// Deallocates chunk array
    /// 
    /// # Safety
    /// 
    /// - there are no chunk references exist
    /// - chunk array box is box-allocated
    unsafe fn deallocate(this: *mut Self) {
        // FIXME:
        eprintln!("ChunkArrayBox deallocated");

        _ = unsafe { Box::from_raw(this) };
    }

    /// Unowns this array allocation so it evetually can be dropped
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkArray`]
    unsafe fn unown(&mut self) {
        // FIXME:
        eprintln!("ChunkArrayBox unowned");

        self.owned.store(false, Release);
        
        self.try_deallocate();
    }

    /// Tries to deallocate chunk array box. Returns success value
    /// 
    /// # Safety
    /// 
    /// Chunk array box should be box-allocated
    unsafe fn try_deallocate(&mut self) -> bool {
        if self.owned.load(Relaxed) {
            return false;
        }

        let is_free = self.borrow_map.iter().all(ChunkBorrowInfo::is_free);

        if is_free {
            Self::deallocate(self as *mut _);
        }

        is_free
    }

    /// Frees shared borrow of the chunk at index `index`. Returns success value
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkRef`]
    unsafe fn chunk_shared_free(&self, index: usize) -> bool {
        self.borrow_map[index].free()
    }

    /// Frees unique borrow of the chunk at index `index`. Returns success value
    /// 
    /// # Safety
    /// 
    /// Can only be called by [`ChunkMut`]
    unsafe fn chunk_mut_free(&self, index: usize) -> bool {
        self.borrow_map[index].free_mut()
    }

    /// Borrows a chunk from chunk array
    pub fn chunk(&self, pos: Int3) -> Option<ChunkRef> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index].borrow().then(move || unsafe {
            ChunkRef::new(NonNull::from(self), index)
        })
    }

    /// Uniquely borrows a chunk from chunk array
    pub fn chunk_mut(&self, pos: Int3) -> Option<ChunkMut> {
        let index = ChunkArray::chunk_pos_to_linear_index(self.sizes, pos)?;

        self.borrow_map[index].borrow_mut().then(move || unsafe {
            ChunkMut::new(NonNull::from(self), index)
        })
    }
}

impl Default for ChunkArrayBox {
    fn default() -> Self {
        Self {
            sizes: USize3::ZERO,
            owned: AtomicBool::new(true),
            borrow_map: vec![],
            chunks: vec![],
        }
    }
}



pub type ChunkAdj = Sides<Option<ChunkRef>>;



#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn array_allocations() {
        let array = ChunkArray::new_empty(USize3::new(2, 2, 2));
        let chunk_ref = array.chunk(Int3::ZERO).unwrap();
        let array = array;
        let chunk_ref2 = chunk_ref.clone();

        assert_eq!(
            2,
            array.borrow_map[ChunkArray::chunk_pos_to_linear_index(
                array.sizes, Int3::ZERO
            ).unwrap()].shared_count()
        );

        drop(array);

        eprintln!("chunk is empty: {}", chunk_ref.is_empty());
    }
}