/**
 * Camera handler.
 */

use directx_math::*;

/// Camera handler.
pub struct Camera {
	pub roll: f32,
	pub pitch: f32,
	pub yaw: f32,
	pub fov: f32,
	pub r: f32,

	/* Terrible code hightliting. It's type, not an object, stupid! */
	pub pos: XMVector,
	pub up: XMVector,
	pub front: XMVector,
	pub right: XMVector,

	pub rotation: XMMatrix,
}

impl Camera {
	/// Creates camera.
	pub fn new() -> Self { Default::default() }

	/// This function updates camera vectors from rotatiion matrix.
	pub fn update_vectors(&mut self) {
		/* Transform basic vectors with rotation matrix */
		self.up.0    = XMVector4Transform(XMVectorSet(0.0, 1.0,  0.0, 1.0), self.rotation.0);
		self.front.0 = XMVector4Transform(XMVectorSet(0.0, 0.0, -1.0, 1.0), self.rotation.0);
		self.right.0 = XMVector4Transform(XMVectorSet(1.0, 0.0,  0.0, 1.0), self.rotation.0);

		/* Translate camera relative to origin (0.0, 0.0, 0.0) */
		self.pos.0   = XMVector4Transform(
			XMVectorSet(0.0, 0.0, 0.0, 1.0),
			XMMatrixTranslation(
				self.r * XMVectorGetX(self.front.0),
				self.r * XMVectorGetY(self.front.0),
				self.r * XMVectorGetZ(self.front.0)
			)
		);
	}

	/// Stores rotation.
	pub fn set_rotation(&mut self, r: f32, roll: f32, pitch: f32, yaw: f32) {
		self.r = -r;
		self.roll = roll;
		self.pitch = pitch;
		self.yaw = yaw;

		self.rotation.0 = XMMatrixRotationRollPitchYaw(roll, pitch, yaw);

		self.update_vectors();
	}

	/// Sets rotation to [0.0, 0.0, 0.0].
	pub fn reset_rotation(&mut self) {
		self.rotation.0 = XMMatrixRotationRollPitchYaw(0.0, 0.0, 0.0);
	}

	/// Returns view matrix.
	pub fn get_view(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixLookAtLH(self.pos.0, (self.pos + self.front).0, self.up.0)).into()
	}

	/// Returns projection matrix.
	pub fn get_proj(&self) -> [[f32; 4]; 4] {
		XMMatrix(XMMatrixPerspectiveLH(1.0, 768.0 / 1024.0, 0.5, 1000.0)).into()
	}

	/// Returns X component of pos vector.
	pub fn get_x(&self) -> f32 {
		XMVectorGetX(self.pos.0)
	}

	/// Returns Y component of pos vector.
	pub fn get_y(&self) -> f32 {
		XMVectorGetY(self.pos.0)
	}

	/// Returns Z component of pos vector.
	pub fn get_z(&self) -> f32 {
		XMVectorGetZ(self.pos.0)
	}
}

impl Default for Camera {
	fn default() -> Self {
		let mut cam = Camera {
			roll: 0.0,
			pitch: 0.0,
			yaw: 0.0,
			r: -1.0,
			fov: 60.0,
			pos: XMVector(XMVectorSet(0.0, 0.0, -3.0, 1.0)),
			up: XMVector(XMVectorSet(0.0, 1.0, 0.0, 1.0)),
			front: XMVector(XMVectorSet(0.0, 0.0, -1.0, 1.0)),
			right: XMVector(XMVectorSet(1.0, 0.0, 0.0, 1.0)),
			rotation: XMMatrix(XMMatrixRotationRollPitchYaw(0.0, 0.0, 0.0)),
		};
		cam.update_vectors();

		return cam;
	}
}