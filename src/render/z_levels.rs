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
/// Z-level delta for moving beyond y-sorted objects.
///
/// This is set to a higher delta than anything y-sorted could have from the y-sorted entity this is applied to.
pub(crate) const Y_SORT_OVERRIDE_Z_DELTA: f32 = 1.;
