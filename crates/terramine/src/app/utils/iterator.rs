//! A bunch of handy iterators for chunks.

#![allow(dead_code)]

use {
    crate::prelude::*,
    std::ops::{Range, RangeBounds},
};

/// Iterator over chunk border.
/// 
/// # Example:
/// ```
/// use crate::app::utils::terrain::chunk::{
///     position_function,
///     iterator::CubeBorder,
/// };
/// 
/// // [`CubeBorder`] iterator
/// let border = CubeBorder::new(16);
/// 
/// const MAX: i32 = 16 - 1;
/// let classic_iter = (0 .. 16_i32.pow(3))
///     .map(|i| position_function(i))
///     .filter(|pos|
///         // Check 'boundary' condition
///         pos.x() == 0 || pos.x() == MAX ||
///         pos.y() == 0 || pos.y() == MAX ||
///         pos.z() == 0 || pos.z() == MAX
///     );
/// 
/// // Walk over both together
/// for (b, w) in border.zip(classic_iter) {
///     assert_eq!(b, w)
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CubeBoundary {
    prev: IVec3,
    size: i32,
}

impl CubeBoundary {
    const INITIAL_STATE: i32 = -1;
    pub const fn new(size: i32) -> Self {
        Self { prev: IVec3::splat(Self::INITIAL_STATE), size }
    }
}

impl Iterator for CubeBoundary {
    type Item = IVec3;
    fn next(&mut self) -> Option<Self::Item> {
        /* Maximun corrdinate value in border */
        let max: i32 = self.size - 1;
        let max_minus_one: i32 = max - 1;

        /* Return if maximum reached */
        if self.prev == IVec3::splat(max) {
            return None
        } else if self.prev == IVec3::splat(Self::INITIAL_STATE) {
            let new = IVec3::splat(0);
            self.prev = new;
            return Some(new)
        }

        /* Previous x, y and z */
        let [x, y, z] = self.prev.to_array();

        /* Returning local function */
        let mut give = |pos| {
            self.prev = pos;
            Some(pos)
        };

        /* If somewhere slicing cube (in 1 .. MAX - 1) */
        if x >= 1 && x <= max_minus_one {
            /* If position is slicing square */
            if y >= 1 && y <= max_minus_one  {
                if z == 0 { give(IVec3::new(x, y, max)) }
                else if z == max { give(IVec3::new(x, y + 1, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0 or {max}! But actual value is {y}.",
                    max = max,
                    y = y,
                )}
            }

            /* If position is on flat bottom of square */
            else if y == 0 {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(x, y + 1, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* If position is on flat top of square */
            else if y == max {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(x + 1, 0, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* Other Ys are unreachable */
            else { unreachable!(
                "Invalid y position! Must be in 0..{size}! But actual value is {y}.",
                size = self.size,
                y = y,
            )}
        }

        /* If current position is bottom */
        else if x == 0 {
            /* Y is not maximum */
            if y >= 0 && y <= max_minus_one {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(x, y + 1, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* Y is maximum */
            else if y == max {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(x + 1, 0, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* Others Ys are unreachable */
            else { unreachable!(
                "Invalid y position! Must be in 0..{size}! But actual value is {y}.",
                size = self.size,
                y = y,
            )}
        }

        /* If currents position is top */
        else if x == max {
            /* Y is not maximum */
            if y >= 0 && y <= max_minus_one {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(x, y + 1, 0)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* Y is maximum */
            else if y == max {
                if z >= 0 && z <= max_minus_one { give(IVec3::new(x, y, z + 1)) }
                else if z == max { give(IVec3::new(max, max, max)) }
                else { unreachable!(
                    "Invalid z position! Must be in 0..{size}! But actual value is {z}.",
                    size = self.size,
                    z = z,
                )}
            }

            /* Others Ys are unreachable */
            else { unreachable!(
                "Invalid y position! Must be in 0..{size}! But actual value is {y}.",
                size = self.size,
                y = y,
            )}
        }

        /* Other values of X is unreachable... */
        else { unreachable!(
            "Invalid x position! Must be in 0..{size}! But actual value is {x}.",
            size = self.size,
            x = x,
        )}
    }
}

/// Iterator that yields cartesian product.
/// 
/// # Example:
/// 
/// ```
/// use crate::app::utils::{
///     math::IVec3,
///     terrain::chunk::iterator::SpaceIter,
/// };
/// 
/// let mut res1 = vec![];
/// let mut res2 = vec![];
/// 
/// // [`SpaceIter`] equivalent
/// for pos in SpaceIter::new(IVec3::ZERO..IVec3::splat(16)) {
///     res1.push(pos)
/// }
/// 
/// // Classic 3-fold loop
/// for x in 0..16 {
/// for y in 0..16 {
/// for z in 0..16 {
///     res2.push(IVec3::new(x, y, z))
/// }}}
/// 
/// assert_eq!(res1, res2);
/// ```
#[derive(Debug, Clone)]
pub struct Range3d {
    back_shift: IVec3,
    sizes: U64Vec3,
    idx: u64,
    size: u64,
}

impl Range3d {
    pub fn new(range: Range<IVec3>) -> Self {
        let diff = IVec3::new(
            range.end.x - range.start.x,
            range.end.y - range.start.y,
            range.end.z - range.start.z,
        );

        assert!(
            0 <= diff.x && 0 <= diff.y && 0 <= diff.z,
            "start position should be not greater by each coordinate than end."
        );

        let sizes = diff.as_u64vec3();
        let size = sizes.x * sizes.y * sizes.z;

        Self { sizes, size, idx: 0, back_shift: range.start }
    }

    pub fn new_cubed(range: Range<i32>) -> Self {
        assert!(range.start <= range.end, "space iter cannot go back");
        Self::new(IVec3::splat(range.start)..IVec3::splat(range.end))
    }

    #[allow(dead_code)]
    pub fn zeroed(end: IVec3) -> Self {
        Self::new(IVec3::ZERO..end)
    }

    pub fn zeroed_cubed(end: i32) -> Self {
        Self::new_cubed(0..end)
    }

    pub fn split_chunks(iter_size: IVec3, chunk_size: IVec3) -> impl ExactSizeIterator<Item = Self> {
        assert_eq!(
            iter_size % chunk_size, IVec3::ZERO,
            "iter_size (value is {iter_size:?}) should be divisible by chunk_size (value is {chunk_size:?})"
        );

        Self::zeroed(iter_size / chunk_size)
            .map(move |chunk_pos| {
                let min = chunk_pos * chunk_size;
                Range3d::from(min .. min + chunk_size)
            })
    }

    pub fn adj_iter(pos: IVec3) -> std::array::IntoIter<IVec3, 6> {
        [
            pos + IVec3::new( 1,  0,  0),
            pos + IVec3::new(-1,  0,  0),
            pos + IVec3::new( 0,  1,  0),
            pos + IVec3::new( 0, -1,  0),
            pos + IVec3::new( 0,  0,  1),
            pos + IVec3::new( 0,  0, -1),
        ].into_iter()
    }

    const fn coord_idx_from_idx(idx: u64, sizes: U64Vec3) -> U64Vec3 {
        linear_index_to_volume(idx, sizes)
    }

    const fn pos_from_coord_idx(idx: U64Vec3, back_shift: IVec3) -> IVec3 {
        IVec3::new(
            back_shift.x + idx.x as i32,
            back_shift.y + idx.y as i32,
            back_shift.z + idx.z as i32,
        )
    }

    const fn pos_from_idx(idx: u64, back_shift: IVec3, sizes: U64Vec3) -> IVec3 {
        Self::pos_from_coord_idx(
            Self::coord_idx_from_idx(idx, sizes),
            back_shift,
        )
    }
}

impl<R: RangeBounds<IVec3>> From<R> for Range3d {
    fn from(range: R) -> Self {
        use std::ops::Bound::*;

        let start = match range.start_bound() {
            Included(&bound) => bound,
            Excluded(&bound) => bound + IVec3::ONE,
            Unbounded => panic!("unbounded SpaceIter can't be implemented"),
        };

        let end = match range.end_bound() {
            Included(&bound) => bound + IVec3::ONE,
            Excluded(&bound) => bound,
            Unbounded => panic!("unbounded SpaceIter can't be implemented"),
        };

        let diff = end - start;
        assert!(
            0 <= diff.x && 0 <= diff.y && 0 <= diff.z,
            "start position should be not greater by each coordinate than end. Range: {}..{}",
            start,
            end,
        );

        let sizes = diff.as_u64vec3();
        let size = sizes.x * sizes.y * sizes.z;

        Self { sizes, size, idx: 0, back_shift: start }
    }
}

impl Iterator for Range3d {
    type Item = IVec3;
    fn next(&mut self) -> Option<Self::Item> {
        (self.idx < self.size).then(|| {
            self.idx += 1;
            Self::pos_from_idx(self.idx - 1, self.back_shift, self.sizes)
        })
    }
}

impl ExactSizeIterator for Range3d {
    fn len(&self) -> usize { self.size as usize }
}

/// Position function.
pub const fn linear_index_to_volume(idx: u64, sizes: U64Vec3) -> U64Vec3 {
    let xy = idx / sizes.z;

    let z = idx % sizes.z;
    let y = xy % sizes.y;
    let x = xy / sizes.y;

    U64Vec3::new(x, y, z)
}



/// Walks around 3D array in very specific way.
/// It breaks standart 3-fold cycle into chunks
/// and walks in them like usual 3-fold cycle.
/// 
/// It is in some way relative to 3-fold cycle in 3-fold cycle...
/// 
/// # Example:
/// ```
/// use crate::app::utils::{
///     math::prelude::*,
///     terrain::chunk::iterator::{
///         ChunkSplitten, SpaceIter
///     },
/// };
/// 
/// fn example() {
///     let split = ChunkSplitten::new(IVec3::splat(16), IVec3::splat(2));
///     let space: Vec<_> = SpaceIter::new(IVec3::ZERO..IVec3::splat(16)).collect();
/// 
///     for (entire, _) in split {
///         assert!(space.contains(&entire));
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ChunkSplitten {
    inner: Range3d,
    outer: Range3d,
    current: Option<IVec3>,

    chunk_size: IVec3,
}

impl ChunkSplitten {
    /// Creates new [`ChunkSplitten`] from [`IVec3`] of sizes of
    /// entire data and [`IVec3`] of chunk sizes in elements of entire data structure.
    /// 
    /// # Panic
    /// Pnics if entire chunk size is not divisible into smaller chunk sizes.
    #[allow(dead_code)] 
    pub fn new(entire: IVec3, chunk_size: IVec3) -> Self {
        /* Check that entire chunk are divisible into smaller ones */
        assert_eq!(entire % chunk_size, IVec3::ZERO);

        let mut outer = Range3d::from(IVec3::ZERO .. entire / chunk_size);
        let current = outer.next().unwrap();

        Self {
            inner: Range3d::from(IVec3::ZERO..chunk_size),
            outer, current: Some(current), chunk_size,
        }
    }
}

impl Iterator for ChunkSplitten {
    // Types for outer and inner positions.
    type Item = (IVec3, IVec3, IVec3);
    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.next().unwrap_or_else(|| {
            self.current = self.outer.next();
            self.inner = Range3d::from(IVec3::ZERO..self.chunk_size);

            self.inner.next().unwrap()
        });
        
        let outer = self.current?;

        Some((outer * self.chunk_size + inner, inner, outer))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Sides<T> {
    /// Adjacent chunks in order: `back[0] -> front[1] -> top[2] -> 
    /// bottom[3] -> right[4] -> left[5]`.
    pub inner: [T; 6],
}

impl<T> std::iter::FromIterator<T> for Sides<T> {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        let mut iter = iter.into_iter();

        let arr = std::array::from_fn(|_|
            iter.next()
                .expect("iterator should have exactly 6 elements")
        );

        assert!(iter.next().is_none(), "iterator should have exactly 6 elements");

        Self { inner: arr }
    }
}

impl<T: Copy> Copy for Sides<T> { }

impl<T> Sides<T> {
    pub const fn new(sides: [T; 6]) -> Self {
        Self { inner: sides }
    }

    pub fn all(side: T) -> Self where T: Clone {
        Self::new(std::array::from_fn(|_| side.clone()))
    }

    pub const fn independent(back: T, front: T, top: T, bottom: T, right: T, left: T) -> Self {
        Self::new([back, front, top, bottom, right, left])
    }

    pub fn as_array(&self) -> [T; 6] where T: Clone {
        self.inner.clone()
    }

    pub fn set(&mut self, offset: IVec3, item: T) -> Result<(), OffsetError> {
        match offset.to_array() {
            [ 1,  0,  0] => self.inner[0] = item,
            [-1,  0,  0] => self.inner[1] = item,
            [ 0,  1,  0] => self.inner[2] = item,
            [ 0, -1,  0] => self.inner[3] = item,
            [ 0,  0,  1] => self.inner[4] = item,
            [ 0,  0, -1] => self.inner[5] = item,
            _ => return Err(OffsetError::Invalid { offset }),
        }

        Ok(())
    }

    pub fn by_offset(&self, offset: IVec3) -> Result<T, OffsetError>
    where
        T: Clone,
    {
        Ok(match offset.to_array() {
            [ 1,  0,  0] => self.back(),
            [-1,  0,  0] => self.front(),
            [ 0,  1,  0] => self.top(),
            [ 0, -1,  0] => self.bottom(),
            [ 0,  0,  1] => self.right(),
            [ 0,  0, -1] => self.left(),
            _ => return Err(OffsetError::Invalid { offset }),
        })
    }

    pub fn back_mut(&mut self)   -> &mut T { &mut self.inner[0] }
    pub fn front_mut(&mut self)  -> &mut T { &mut self.inner[1] }
    pub fn top_mut(&mut self)    -> &mut T { &mut self.inner[2] }
    pub fn bottom_mut(&mut self) -> &mut T { &mut self.inner[3] }
    pub fn right_mut(&mut self)  -> &mut T { &mut self.inner[4] }
    pub fn left_mut(&mut self)   -> &mut T { &mut self.inner[5] }

    pub const fn back_ref(&self)   -> &T { &self.inner[0] }
    pub const fn front_ref(&self)  -> &T { &self.inner[1] }
    pub const fn top_ref(&self)    -> &T { &self.inner[2] }
    pub const fn bottom_ref(&self) -> &T { &self.inner[3] }
    pub const fn right_ref(&self)  -> &T { &self.inner[4] }
    pub const fn left_ref(&self)   -> &T { &self.inner[5] }

    pub fn back(&self)   -> T where T: Clone { self.inner[0].clone() }
    pub fn front(&self)  -> T where T: Clone { self.inner[1].clone() }
    pub fn top(&self)    -> T where T: Clone { self.inner[2].clone() }
    pub fn bottom(&self) -> T where T: Clone { self.inner[3].clone() }
    pub fn right(&self)  -> T where T: Clone { self.inner[4].clone() }
    pub fn left(&self)   -> T where T: Clone { self.inner[5].clone() }

    pub fn map<P>(self, map: impl Fn(T) -> P) -> Sides<P> {
        let Self { inner: sides } = self;
        let sides = sides.map(map);
        Sides { inner: sides }
    }
}



#[derive(Debug, Error)]
pub enum OffsetError {
    #[error("Invalid 'sides' offset {offset}")]
    Invalid { offset: IVec3 },
}



impl<T> std::ops::Index<usize> for Sides<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<usize> for Sides<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

#[allow(dead_code)]
pub fn offsets_from_border(pos: IVec3, bounds: impl RangeBounds<IVec3>) -> SmallVec<[IVec3; 6]> {
    let mut result = SmallVec::new();

    use std::ops::Bound::*;

    let start = match bounds.start_bound() {
        Included(&bound) => bound,
        Excluded(&bound) => bound + IVec3::ONE,
        Unbounded => panic!("unbounded can't be implemented"),
    };

    let end = match bounds.end_bound() {
        Included(&bound) => bound + IVec3::ONE,
        Excluded(&bound) => bound,
        Unbounded => panic!("unbounded can't be implemented"),
    };

    if pos.x == start.x { result.push(IVec3::new(-1, 0, 0)) }
    if pos.y == start.y { result.push(IVec3::new(0, -1, 0)) }
    if pos.z == start.z { result.push(IVec3::new(0, 0, -1)) }
    if pos.x == end.x - 1 { result.push(IVec3::new(1, 0, 0)) }
    if pos.y == end.y - 1 { result.push(IVec3::new(0, 1, 0)) }
    if pos.z == end.z - 1 { result.push(IVec3::new(0, 0, 1)) }

    result
}



/// Gives index in `pos.len()` dimension array
/// HINT: X -> Y -> Z and W -> H -> D
pub fn get_index(pos: &[usize], dims: &[usize]) -> usize {
    assert!(
        pos.len() >= dims.len(),
        "`pos.len()` should be dimension of array with dimensions `dims.len()` but both equal: [{}, {}]",
        pos.len(), dims.len()
    );

    pos.iter()
        .zip(dims.iter())
        .skip(1)
        .fold(pos[0], |accum, (&p, &d)| d * accum + p)
}



#[cfg(test)]
mod space_iter_tests {
    use super::*;

    #[test]
    fn test_new() {
        let range = IVec3::new(-10, -3, -1)..IVec3::new(-3, 4, 2);
        let poses1: Vec<_> = Range3d::from(range.clone())
            .collect();
        let poses2: Vec<_> = Range3d::from(range)
            .collect();

        for pos in poses1.iter() {
            assert!(poses2.contains(pos), "{}, {:?}, {:?}", pos, poses1, poses2);
        }
 
        for pos in poses2.iter() {
            assert!(poses1.contains(pos), "{}, {:?}, {:?}", pos, poses1, poses2);
        }
    }

    #[test]
    fn test_split_chunks() {
        let sample: Vec<_> = Range3d::zeroed_cubed(4)
            .collect();
        let chunked: Vec<_> = Range3d::split_chunks(IVec3::splat(4), IVec3::splat(2))
            .flatten()
            .collect();
    
        for pos in sample.iter() {
            if !chunked.contains(pos) {
                panic!("chunked.contains(&pos): {:?}", pos);
            }
        }
    
        for pos in chunked.iter() {
            if !sample.contains(pos) {
                panic!("sample.contains(&pos): {:?}", pos);
            }
        }
    }

    #[test]
    fn zero_start_simple() {
        let mut res1 = vec![];
        let mut res2 = vec![];

        for pos in Range3d::from(IVec3::ZERO .. IVec3::splat(5)) {
            res1.push(pos)
        }

        for x in 0..5 {
        for y in 0..5 {
        for z in 0..5 {
            res2.push(IVec3::new(x, y, z))
        }}}

        assert_eq!(res1, res2);
    }

    #[test]
    fn zero_start_hard() {
        let mut res1 = vec![];
        let mut res2 = vec![];

        for pos in Range3d::from(IVec3::ZERO .. IVec3::new(16, 4, 9)) {
            res1.push(pos)
        }

        for x in 0..16 {
        for y in 0..4 {
        for z in 0..9 {
            res2.push(IVec3::new(x, y, z))
        }}}

        assert_eq!(res1, res2);
    }

    #[test]
    fn simple_start_simple() {
        let mut res1 = vec![];
        let mut res2 = vec![];

        for pos in Range3d::from(IVec3::splat(-5) .. IVec3::splat(5)) {
            res1.push(pos)
        }

        for x in -5..5 {
        for y in -5..5 {
        for z in -5..5 {
            res2.push(IVec3::new(x, y, z))
        }}}

        assert_eq!(res1, res2);
    }

    #[test]
    fn hard_start_hard() {
        let mut res1 = vec![];
        let mut res2 = vec![];

        for pos in Range3d::from(IVec3::new(-8, 2, -10) .. IVec3::new(9, 5, -5)) {
            res1.push(pos)
        }

        for x in -8..9 {
        for y in 2..5 {
        for z in -10..-5 {
            res2.push(IVec3::new(x, y, z))
        }}}

        assert_eq!(res1, res2);
    }

    #[test]
    fn uniqueness() {
        let iter = Range3d::from(IVec3::new(-8, 2, -10) .. IVec3::new(9, 5, -5));
        let mut map = std::collections::HashSet::new();

        for pos in iter {
            assert!(map.insert(pos));
        }
    }
}

#[cfg(test)]
mod border_test {
    use {
        crate::app::utils::terrain::chunk::Chunk,
        super::*,
    };

    #[test]
    fn test_sides() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum Side {
            Top, Bottom, Left, Right, Front, Back,
        }
        
        let back   = IVec3::new( 1,  0,  0);
        let front  = IVec3::new(-1,  0,  0);
        let top    = IVec3::new( 0,  1,  0);
        let bottom = IVec3::new( 0, -1,  0);
        let right  = IVec3::new( 0,  0,  1);
        let left   = IVec3::new( 0,  0, -1);
    
        let mut sides = Sides::new([Side::Top; 6]);
        sides.set(back,   Side::Back).unwrap();
        sides.set(front,  Side::Front).unwrap();
        sides.set(top,    Side::Top).unwrap();
        sides.set(bottom, Side::Bottom).unwrap();
        sides.set(right,  Side::Right).unwrap();
        sides.set(left,   Side::Left).unwrap();
    
        assert_eq!(sides.back(), Side::Back);
        assert_eq!(sides.front(), Side::Front);
        assert_eq!(sides.top(), Side::Top);
        assert_eq!(sides.bottom(), Side::Bottom);
        assert_eq!(sides.right(), Side::Right);
        assert_eq!(sides.left(), Side::Left);
    }

    #[test]
    fn test1() {
        let border = CubeBoundary::new(Chunk::SIZE as i32);
        const MAX: i32 = Chunk::SIZE as i32 - 1;

        for pos in border {
            eprintln!("{:?}", pos);

            assert!(
                pos.x == 0 || pos.x == MAX ||
                pos.y == 0 || pos.y == MAX ||
                pos.z == 0 || pos.z == MAX
            );
        }
    }

    #[test]
    fn test2() {
        let border = CubeBoundary::new(Chunk::SIZE as i32);
        const MAX: i32 = Chunk::SIZE as i32 - 1;

        let works = (0..Chunk::VOLUME)
            .map(|i| linear_index_to_volume(i as u64, Chunk::SIZES.as_u64vec3()).as_ivec3())
            .filter(|pos|
                pos.x == 0 || pos.x == MAX ||
                pos.y == 0 || pos.y == MAX ||
                pos.z == 0 || pos.z == MAX
            );

        for (b, w) in border.zip(works) {
            assert_eq!(b, w)
        }
    }
}

#[cfg(test)]
mod splitten_tests {
    use super::*;

    #[test]
    fn space_contains_split() {
        let split = ChunkSplitten::new(IVec3::splat(16), IVec3::splat(2));
        let space: Vec<_> = Range3d::from(IVec3::ZERO .. IVec3::splat(16)).collect();

        for (entire, _, _) in split {
            assert!(space.contains(&entire));
        }
    }

    #[test]
    fn split_contains_space() {
        let entire: Vec<_> = ChunkSplitten::new(IVec3::splat(16), IVec3::splat(2)).map(|(e, _, _)| e).collect();
        let space = Range3d::from(IVec3::ZERO .. IVec3::splat(16));

        for pos in space {
            assert!(entire.contains(&pos));
        }
    }

    #[test]
    fn length() {
        let all: Vec<_> = ChunkSplitten::new(IVec3::splat(32), IVec3::splat(4)).collect();
        let space: Vec<_> = Range3d::from(IVec3::ZERO .. IVec3::splat(32)).collect();

        assert_eq!(all.len(), space.len());
    }

    #[test]
    fn print() {
        let split = ChunkSplitten::new(IVec3::splat(6), IVec3::splat(2));

        for (entire, inner, _) in split {
            eprintln!("{:?} in {:?}", inner, entire);
        }
    }

    #[test]
    fn uniqueness() {
        let split = ChunkSplitten::new(IVec3::splat(4), IVec3::splat(2));
        let mut set = std::collections::HashSet::new();

        for (entire, inner, _) in split {
            assert!(
                set.insert(entire),
                "Values are: {:?} in {:?}",
                inner, entire
            );
        }
    }
}