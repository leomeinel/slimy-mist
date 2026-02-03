/*
 * File: layers.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/custom_dynamic_assets
 */

use std::marker::PhantomData;

use bevy::{
    asset::RenderAssetUsages, platform::collections::HashMap, prelude::*, reflect::Reflectable,
};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::{
    logging::error::{ERR_INVALID_IMAGE, ERR_INVALID_LAYER_MAP},
    visual::Visible,
};

/// Applies to anything that stores layer maps
pub(crate) trait LayerMaps
where
    Self: AssetCollection + Resource + Default + Reflectable,
{
    fn sorted_fields(&self) -> Vec<&HashMap<String, Handle<Image>>>;
    fn to_display_image<T>(
        &self,
        data: &Res<LayerDataCache<T>>,
        images: &mut ResMut<Assets<Image>>,
    ) -> DisplayImage<T>
    where
        T: Visible,
    {
        // Filter for matching layers between `data.layers` and `layer_maps` and collect valid `image_data` and single `metadata`
        // NOTE: We are just assuming that all `Images` have the exact same metadata here. I deem this to be appropriate here.
        let layer_maps = self.sorted_fields();
        let mut image_data = Vec::new();
        let mut metadata = None;
        for (layer, layer_map) in data.layers.iter().zip(layer_maps) {
            let Some(layer) = layer else {
                continue;
            };
            let image = layer_map.get(layer).expect(ERR_INVALID_LAYER_MAP);
            let image = images.get(image).expect(ERR_INVALID_LAYER_MAP);
            image_data.push(image.data.clone().expect(ERR_INVALID_IMAGE));

            let descriptor = &image.texture_descriptor;
            metadata = Some((descriptor.size, descriptor.dimension, descriptor.format));
        }

        // NOTE: We are using `ERR_INVALID_LAYER_MAP` here because a failure here means that no valid layer has been found.
        let metadata = metadata.expect(ERR_INVALID_LAYER_MAP);

        // Combine `Images` into a single `Image` by overriding non-transparent pixels in each previous iteration of `image_data`.
        // FIXME: This probably does not work for transparent pixels.
        // NOTE: We are iterating in reverse order to make the first layer the top layer.
        let image_data = image_data
            .into_iter()
            .rev()
            .reduce(|mut current, next| {
                for (c, n) in current.iter_mut().zip(next) {
                    if n != 0 {
                        *c = n;
                    }
                }
                current
            })
            .expect(ERR_INVALID_IMAGE);
        let image = Image::new(
            metadata.0,
            metadata.1,
            image_data,
            metadata.2,
            RenderAssetUsages::all(),
        );

        DisplayImage::new(images.add(image))
    }
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Reflect, Default)]
pub(crate) struct HumanMaleLayerMaps {
    #[asset(key = "male.upper_accessories", collection(typed, mapped))]
    upper_accessories: HashMap<String, Handle<Image>>,
    #[asset(key = "male.upper_clothing", collection(typed, mapped))]
    upper_clothing: HashMap<String, Handle<Image>>,
    #[asset(key = "male.upper_hair", collection(typed, mapped))]
    upper_hair: HashMap<String, Handle<Image>>,
    #[asset(key = "male.upper_head", collection(typed, mapped))]
    upper_head: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_accessories", collection(typed, mapped))]
    lower_regular_accessories: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_arms_clothing", collection(typed, mapped))]
    lower_regular_arms_clothing: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_arms", collection(typed, mapped))]
    lower_regular_arms: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_armor", collection(typed, mapped))]
    lower_regular_armor: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_chest_clothing", collection(typed, mapped))]
    lower_regular_chest_clothing: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_chest", collection(typed, mapped))]
    lower_regular_chest: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_legs_clothing", collection(typed, mapped))]
    lower_regular_legs_clothing: HashMap<String, Handle<Image>>,
    #[asset(key = "male.lower_regular_legs", collection(typed, mapped))]
    lower_regular_legs: HashMap<String, Handle<Image>>,
}
impl LayerMaps for HumanMaleLayerMaps {
    fn sorted_fields(&self) -> Vec<&HashMap<String, Handle<Image>>> {
        vec![
            &self.upper_accessories,
            &self.upper_clothing,
            &self.upper_hair,
            &self.upper_head,
            &self.lower_regular_accessories,
            &self.lower_regular_arms_clothing,
            &self.lower_regular_arms,
            &self.lower_regular_armor,
            &self.lower_regular_chest_clothing,
            &self.lower_regular_chest,
            &self.lower_regular_legs_clothing,
            &self.lower_regular_legs,
        ]
    }
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Reflect, Default)]
pub(crate) struct SlimeLayerMaps {
    #[asset(key = "slime.body_eyes", collection(typed, mapped))]
    body_eyes: HashMap<String, Handle<Image>>,
    #[asset(key = "slime.body_skin", collection(typed, mapped))]
    body_skin: HashMap<String, Handle<Image>>,
}
impl LayerMaps for SlimeLayerMaps {
    fn sorted_fields(&self) -> Vec<&HashMap<String, Handle<Image>>> {
        vec![&self.body_eyes, &self.body_skin]
    }
}

/// Layer data deserialized from a ron file as a generic
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub(crate) struct LayerData<T>
where
    T: Visible,
{
    #[serde(default)]
    pub(crate) layers: Vec<Option<String>>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// Handle for [`LayerData`] as a generic
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
#[derive(Resource)]
pub(crate) struct LayerHandle<T>(pub(crate) Handle<LayerData<T>>)
where
    T: Visible;

/// Cache for [`LayerData`]
///
/// This is to allow easier access.
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
#[derive(Resource, Default)]
pub(crate) struct LayerDataCache<T>
where
    T: Visible,
{
    pub(crate) layers: Vec<Option<String>>,
    pub(crate) _phantom: PhantomData<T>,
}

/// [`Image`] for displaying `T`
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
#[derive(Resource, Default)]
pub(crate) struct DisplayImage<T>
where
    T: Visible,
{
    pub(crate) image: Handle<Image>,
    pub(crate) _phantom: PhantomData<T>,
}
impl<T> DisplayImage<T>
where
    T: Visible,
{
    fn new(image: Handle<Image>) -> Self {
        Self { image, ..default() }
    }
}
