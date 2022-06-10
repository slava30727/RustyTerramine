use std::collections::HashMap;
use std::time::Instant;

pub type ID = u64;

/// Represents profiler target
pub struct Profile {
	target_name: String,
	measures: Vec<f64>,
}

impl Profile {
	/// Creates new profile
	pub fn new(target_name: &str) -> Self {
		Profile { target_name: target_name.to_owned(), measures: vec![] }
	}
}

/// Represents a time measure with drop-stop
pub struct Measure {
	pub value: f64,
	pub now: Instant,
	pub id: ID
}

impl Measure {
    pub fn new(id: ID) -> Self {
		Measure { value: 0.0, now: Instant::now(), id }
	}
}

impl Drop for Measure {
	fn drop(&mut self) {
		self.value = self.now.elapsed().as_secs_f64();
		upload_measure(self);
	}
}

/// Handles all profiles
pub struct Profiler {
	pub profiles: Option<HashMap<ID, Profile>>,
}

impl Profiler {
	/// Gives uninitialyzed version of `Profiler` to create static variable
	const fn uninitialized() -> Self { Profiler { profiles: None } }

	/// Initialyzes static
	pub fn initialyze(&mut self) {
		self.profiles = Some(HashMap::new())
	}
}

pub static mut PROFILER: Profiler = Profiler::uninitialized();
static mut IS_INITIALYZED: bool = false;

/// Initialyzes static
/// Can be called only once! If not then it should panic
pub fn initialyze() {
	unsafe {
		if !IS_INITIALYZED {
			IS_INITIALYZED = true;
			PROFILER.initialyze();
		} else {
			panic!("Can not initialyze profiler twice!");
		}
	}
}

/// Adds profile
pub fn add_profile(profile: Profile, id: ID) {
	unsafe {
		PROFILER.profiles.as_mut().unwrap().insert(id, profile);
	}
}

/// Uploads measure
pub fn upload_measure(measure: &Measure) {
	unsafe {
		PROFILER.profiles.as_mut().unwrap()
			.get_mut(&measure.id).unwrap()
			.measures.push(measure.value);
	}
}

/// Starting capturing to to profile under given `id`
pub fn start_capture(target_name: &str, id: ID) -> Measure {
	unsafe {
		match PROFILER.profiles.as_mut().unwrap().get(&id) {
			None => add_profile(Profile::new(target_name), id),
			_ => ()
		}
	};
	Measure::new(id)
}