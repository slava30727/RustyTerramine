#![allow(unused)]

pub mod noise;

use {
    crate::{
        prelude::*,
        terrain::chunk::{Chunk, array::ChunkArray},
    },
    self::noise::Noise2d,
    spin::RwLock,
};



lazy_static! {
    static ref GENERATOR_SIZES: Mutex<U16Vec3> = Mutex::new(U16Vec3::new(0, 0, 0));
}



module_constructor! {
    use crate::graphics::ui::egui_util::push_window_builder_lock_free;

    // * Safety
    // * 
    // * Safe, because it's going on in module
    // * constructor, so no one access the update list.
    unsafe { push_window_builder_lock_free(spawn_control_window) };
}



macros::atomic_static! {
    static FREQUENCY: f32 = 0.05;
    static N_OCTAVES: usize = 6;
    static PERSISTENCE: f32 = 3.0;
    static LACUNARITY: f32 = 0.5;
    static SEED: u32 = 10;
}

lazy_static! {
    static ref NOISE_VALS: RwLock<Noise2d> = RwLock::new({
        let sizes = Chunk::SIZES * *GENERATOR_SIZES.lock();

        Noise2d::new(
            SEED.load(Relaxed),
            sizes.x as usize,
            sizes.z as usize,
            FREQUENCY.load(Relaxed),
            LACUNARITY.load(Relaxed),
            N_OCTAVES.load(Relaxed),
            PERSISTENCE.load(Relaxed),
        )
    });
}



pub fn spawn_control_window(ui: &mut egui::Ui) {
    ui.collapsing("Generator settings", |ui| {
        _ = FREQUENCY.fetch_update(AcqRel, Relaxed, |mut value| {
            ui.add(egui::DragValue::new(&mut value).prefix("Frequency: "));
            Some(value)
        });
        
        _ = N_OCTAVES.fetch_update(AcqRel, Relaxed, |mut value| {
            ui.add(egui::DragValue::new(&mut value).prefix("Octaves: "));
            Some(value)
        });
        
        _ = PERSISTENCE.fetch_update(AcqRel, Relaxed, |mut value| {
            ui.add(egui::DragValue::new(&mut value).prefix("Persistence: "));
            Some(value)
        });
        
        _ = LACUNARITY.fetch_update(AcqRel, Relaxed, |mut value| {
            ui.add(egui::DragValue::new(&mut value).prefix("Lacunarity: "));
            Some(value)
        });
        
        _ = SEED.fetch_update(AcqRel, Relaxed, |mut value| {
            ui.add(egui::DragValue::new(&mut value).prefix("Seed: "));
            Some(value)
        });

        if ui.button("Build").clicked() {
            let mut noise_values = NOISE_VALS.write();

            let sizes = Chunk::SIZES * *GENERATOR_SIZES.lock();

            _ = mem::replace(&mut *noise_values, Noise2d::new(
                SEED.load(Relaxed),
                sizes.x as usize,
                sizes.z as usize,
                FREQUENCY.load(Relaxed),
                LACUNARITY.load(Relaxed),
                N_OCTAVES.load(Relaxed),
                PERSISTENCE.load(Relaxed),
            ));
        }
    });
}

pub fn perlin(pos: IVec3, chunk_array_sizes: U16Vec3) -> i32 {
    todo!()
}