/**
 * Provides some `type-byte` and `byte-type` reinterpretations to common types
 */

use std::mem::transmute;

pub unsafe trait Reinterpret: ReinterpretAsBytes + ReinterpretFromBytes + ReinterpretSize { }

pub unsafe trait ReinterpretAsBytes {
	fn reinterpret_as_bytes(&self) -> Vec<u8>;
}

pub unsafe trait ReinterpretFromBytes {
	fn reinterpret_from_bytes(source: &[u8]) -> Self;
}

pub unsafe trait ReinterpretSize {
	fn reinterpret_size(&self) -> usize;
}

pub unsafe trait StaticSize {
	fn static_size() -> usize;
}



unsafe impl Reinterpret for u8 { }

unsafe impl ReinterpretAsBytes for u8 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> { vec![*self] }
}

unsafe impl ReinterpretFromBytes for u8 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		source[0]
	}
}

unsafe impl ReinterpretSize for u8 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u8 {
	fn static_size() -> usize { 1 }
}



unsafe impl Reinterpret for i8 { }

unsafe impl ReinterpretAsBytes for i8 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> { unsafe { vec![transmute(*self)] } }
}

unsafe impl ReinterpretFromBytes for i8 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe { transmute(source[0]) }
	}
}

unsafe impl ReinterpretSize for i8 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i8 {
	fn static_size() -> usize { 1 }
}



unsafe impl Reinterpret for u16 { }

unsafe impl ReinterpretAsBytes for u16 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 2] = transmute(*self);
			vec![bytes[0], bytes[1]]
		}
	}
}

unsafe impl ReinterpretFromBytes for u16 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1]])
		}
	}
}

unsafe impl ReinterpretSize for u16 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u16 {
	fn static_size() -> usize { 2 }
}



unsafe impl Reinterpret for i16 { }

unsafe impl ReinterpretAsBytes for i16 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 2] = transmute(*self);
			vec![bytes[0], bytes[1]]
		}
	}
}

unsafe impl ReinterpretFromBytes for i16 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1]])
		}
	}
}

unsafe impl ReinterpretSize for i16 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i16 {
	fn static_size() -> usize { 2 }
}



unsafe impl Reinterpret for u32 { }

unsafe impl ReinterpretAsBytes for u32 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 4] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3]]
		}
	}
}

unsafe impl ReinterpretFromBytes for u32 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3]])
		}
	}
}

unsafe impl ReinterpretSize for u32 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u32 {
	fn static_size() -> usize { 4 }
}



unsafe impl Reinterpret for i32 { }

unsafe impl ReinterpretAsBytes for i32 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 4] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3]]
		}
	}
}

unsafe impl ReinterpretFromBytes for i32 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3]])
		}
	}
}

unsafe impl ReinterpretSize for i32 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i32 {
	fn static_size() -> usize { 4 }
}



unsafe impl Reinterpret for u64 { }

unsafe impl ReinterpretAsBytes for u64 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 8] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
		}
	}
}

unsafe impl ReinterpretFromBytes for u64 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
		}
	}
}

unsafe impl ReinterpretSize for u64 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u64 {
	fn static_size() -> usize { 8 }
}



unsafe impl Reinterpret for i64 { }

unsafe impl ReinterpretAsBytes for i64 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 8] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
		}
	}
}

unsafe impl ReinterpretFromBytes for i64 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
		}
	}
}

unsafe impl ReinterpretSize for i64 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i64 {
	fn static_size() -> usize { 8 }
}



unsafe impl Reinterpret for u128 { }

unsafe impl ReinterpretAsBytes for u128 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 16] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2],  bytes[3],  bytes[4],  bytes[5],  bytes[6],  bytes[7],
				 bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]
		}
	}
}

unsafe impl ReinterpretFromBytes for u128 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
					   source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
		}
	}
}

unsafe impl ReinterpretSize for u128 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for u128 {
	fn static_size() -> usize { 16 }
}



unsafe impl Reinterpret for i128 { }

unsafe impl ReinterpretAsBytes for i128 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 16] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2],  bytes[3],  bytes[4],  bytes[5],  bytes[6],  bytes[7],
				 bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]
		}
	}
}

unsafe impl ReinterpretFromBytes for i128 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
					   source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
		}
	}
}

unsafe impl ReinterpretSize for i128 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for i128 {
	fn static_size() -> usize { 16 }
}



unsafe impl Reinterpret for f32 { }

unsafe impl ReinterpretAsBytes for f32 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 4] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3]]
		}
	}
}

unsafe impl ReinterpretFromBytes for f32 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3]])
		}
	}
}

unsafe impl ReinterpretSize for f32 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for f32 {
	fn static_size() -> usize { 4 }
}



unsafe impl Reinterpret for f64 { }

unsafe impl ReinterpretAsBytes for f64 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 8] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
		}
	}
}

unsafe impl ReinterpretFromBytes for f64 {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
		}
	}
}

unsafe impl ReinterpretSize for f64 {
	fn reinterpret_size(&self) -> usize { Self::static_size() }
}

unsafe impl StaticSize for f64 {
	fn static_size() -> usize { 8 }
}



unsafe impl<T: Reinterpret + StaticSize> Reinterpret for Vec<T> { }

unsafe impl<T: ReinterpretAsBytes + ReinterpretSize> ReinterpretAsBytes for Vec<T> {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(self.reinterpret_size());

		for elem in self.iter() {
			bytes.append(&mut elem.reinterpret_as_bytes());
		}

		return bytes;
	}
}

unsafe impl<T: ReinterpretFromBytes + StaticSize> ReinterpretFromBytes for Vec<T> {
	fn reinterpret_from_bytes(source: &[u8]) -> Self {
		if source.len() == 0 {
			return vec![];
		} else {
			/* Byte data should be aligned by destination byte size */
			debug_assert_eq!(
				source.len() % T::static_size(), 0,
				"Attempting to reinterpret unaligned bytes as aligned by {} bytes. Actual length is {}",
				T::static_size(),
				source.len()
			);

			/* Counter */
			let mut current: usize = 0;

			/* Result */
			let mut result = Vec::with_capacity(source.len() % T::static_size());

			/* Reintepret bytes until vector is full */
			while current <= source.len() - T::static_size() {
				result.push(T::reinterpret_from_bytes(&source[current .. current + T::static_size()]));
				current += T::static_size();
			}

			return result;
		}
	}
}

unsafe impl<T: ReinterpretSize> ReinterpretSize for Vec<T> {
	fn reinterpret_size(&self) -> usize {
		if self.len() == 0 {
			0
		} else {
			self.len() * self[0].reinterpret_size()
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reinterpret_u8() {
		let before: u8 = 23;
		let after = u8::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), u8::static_size());
		assert_eq!(u8::static_size(), 1);
	}

	#[test]
	fn reinterpret_i8() {
		let before: i8 = 23;
		let after = i8::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), i8::static_size());
		assert_eq!(i8::static_size(), 1);
	}

	#[test]
	fn reinterpret_u16() {
		let before: u16 = 13243;
		let after = u16::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), u16::static_size());
		assert_eq!(u16::static_size(), 2);
	}

	#[test]
	fn reinterpret_i16() {
		let before: i16 = 1442;
		let after = i16::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), i16::static_size());
		assert_eq!(i16::static_size(), 2);
	}

	#[test]
	fn reinterpret_u32() {
		let before: u32 = 41432;
		let after = u32::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), u32::static_size());
		assert_eq!(u32::static_size(), 4);
	}

	#[test]
	fn reinterpret_i32() {
		let before: i32 = 2454;
		let after = i32::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), i32::static_size());
		assert_eq!(i32::static_size(), 4);
	}

	#[test]
	fn reinterpret_u64() {
		let before: u64 = 234;
		let after = u64::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), u64::static_size());
		assert_eq!(u64::static_size(), 8);
	}

	#[test]
	fn reinterpret_i64() {
		let before: i64 = 5424254;
		let after = i64::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), i64::static_size());
		assert_eq!(i64::static_size(), 8);
	}

	#[test]
	fn reinterpret_u128() {
		let before: u128 = 23452523453452334;
		let after = u128::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), u128::static_size());
		assert_eq!(u128::static_size(), 16);
	}

	#[test]
	fn reinterpret_i128() {
		let before: i128 = 243523452345;
		let after = i128::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), i128::static_size());
		assert_eq!(i128::static_size(), 16);
	}

	#[test]
	fn reinterpret_f32() {
		let before: f32 = 12.54;
		let after = f32::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), f32::static_size());
		assert_eq!(f32::static_size(), 4);
	}

	#[test]
	fn reinterpret_f64() {
		let before: f64 = 134442.4454;
		let after = f64::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), f64::static_size());
		assert_eq!(f64::static_size(), 8);
	}

	#[test]
	fn reinterpret_vec() {
		let before: Vec<i32> = vec![1, 124, 11, 44, 111, 4523, 765];
		let after = Vec::<i32>::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
		assert_eq!(before.reinterpret_size(), before.len() * i32::static_size());
	}
}