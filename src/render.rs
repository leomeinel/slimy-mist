mod camera;
mod light;
mod materials;
mod palette;
mod particles;
mod ysort;
mod z_levels;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::Visible;
    pub(crate) use super::camera::CanvasCamera;
    pub(crate) use super::light::{
        DayTimer, DayUpdateTimer, LightMeshHandle, LightWrapper, StreetLight,
    };
    pub(crate) use super::materials::Light2dShadow;
    pub(crate) use super::palette::*;
    pub(crate) use super::particles::effects::{
        BloodParticle, DeathParticle, DustTrailParticle, MeleeParticle,
    };
    pub(crate) use super::particles::materials::{
        BloodParticleMaterial, DeathParticleMaterial, DustTrailParticleMaterial,
        MeleeParticleMaterial, Particle2dMaterialHandle,
    };
    pub(crate) use super::particles::{
        Particle, ParticleHandle, ParticleTimer, SpawnChildParticleOnce, SpawnParticleOnce,
        ToggleParticle,
    };
    pub(crate) use super::ysort::{YSort, YSortYOffset};
    pub(crate) use super::z_levels::*;
}

use bevy::{prelude::*, reflect::Reflectable};

use crate::{
    characters::prelude::*, core::prelude::*, levels::prelude::*, procgen::prelude::*,
    screens::prelude::*,
};

pub(super) struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((particles::ParticlesPlugin, light::LightPlugin));

        app.add_systems(
            OnEnter(Screen::Gameplay),
            camera::center_camera_on_player.in_set(EnterGameplaySystems::Camera),
        );
        app.add_systems(Startup, camera::spawn_camera);
        app.add_systems(
            Update,
            (
                camera::scale_projection,
                // NOTE: Having `update_camera` in `PausableSystems` is not the only thing that causes the camera to be
                //       offset when pausing while moving. I do however deem that behavior to be acceptable.
                camera::update_camera
                    .run_if(in_state(Screen::Gameplay))
                    .in_set(PausableSystems),
            )
                .in_set(AppSystems::Update),
        );
        app.add_systems(
            PostUpdate,
            (
                ysort::relative_sort::<Player, OverworldProcGen>,
                ysort::relative_sort::<Slime, OverworldProcGen>,
            )
                .after(EnterGameplaySystems::Images)
                .before(TransformSystems::Propagate)
                .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay))),
        );
    }
}

/// Can apply to anything that is visible
pub(super) trait Visible
where
    Self: Component + Default + Reflectable,
{
}
