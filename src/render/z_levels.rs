/// Z-level for the level.
pub(crate) const LEVEL_Z: f32 = 1.;
/// Z-level for any foreground object.
///
/// The value is chosen so that there is a very reasonable distance to [`OrthographicProjection::far`](bevy::camera::OrthographicProjection::far)
/// while considering relative y-sorting.
pub(crate) const FOREGROUND_Z: f32 = 5.;
/// Z-level for any overlay object.
pub(crate) const OVERLAY_Z: f32 = 9.;
// NOTE: Since light is rendered in a full screen render pass, this currently has no effect.
//       Multiplicative blending is missing from bevy 2d, therefore this can't be avoided in bevy_fast_light.
//       Also see: https://github.com/bevyengine/bevy/issues/24230
/// Z-level for light.
pub(crate) const LIGHT_Z: f32 = 10.;

/// Z-level delta for image layers.
///
/// This is set to a somewhat arbitrary meant to be rendering safe minimal delta to only impact local layer rendering.
pub(crate) const LAYER_Z_DELTA: f32 = 1e-5;
/// Z-level delta for objects.
///
/// This is set to a delta compatible with relative y-sorting that should never subtract/add more than this
/// from [`YSort`](crate::render::prelude::YSort)'s field.
pub(crate) const BASE_Z_DELTA: f32 = 1.;
