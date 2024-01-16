use bevy_app::{App, Plugin};
use bevy_asset::{load_internal_asset, Handle};
use bevy_core_pipeline::prelude::Camera3d;
use bevy_ecs::{prelude::Component, query::With};
use bevy_reflect::Reflect;
use bevy_render::{
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    render_asset::RenderAssets,
    render_resource::{
        binding_types::{sampler, texture_cube},
        *,
    },
    texture::{FallbackImageCubemap, Image},
};

pub const ENVIRONMENT_MAP_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(154476556247605696);

pub struct EnvironmentMapPlugin;

impl Plugin for EnvironmentMapPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            ENVIRONMENT_MAP_SHADER_HANDLE,
            "environment_map.wgsl",
            Shader::from_wgsl
        );

        app.register_type::<EnvironmentMapLight>()
            .add_plugins(ExtractComponentPlugin::<EnvironmentMapLight>::default());
    }
}

/// Environment map based ambient lighting representing light from distant scenery.
///
/// When added to a 3D camera, this component adds indirect light
/// to every point of the scene (including inside, enclosed areas) based on
/// an environment cubemap texture. This is similar to [`crate::AmbientLight`], but
/// higher quality, and is intended for outdoor scenes.
///
/// The environment map must be prefiltered into a diffuse and specular cubemap based on the
/// [split-sum approximation](https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf).
///
/// To prefilter your environment map, you can use `KhronosGroup`'s [glTF-IBL-Sampler](https://github.com/KhronosGroup/glTF-IBL-Sampler).
/// The diffuse map uses the Lambertian distribution, and the specular map uses the GGX distribution.
///
/// `KhronosGroup` also has several prefiltered environment maps that can be found [here](https://github.com/KhronosGroup/glTF-Sample-Environments).
#[derive(Component, Reflect, Clone, ExtractComponent)]
#[extract_component_filter(With<Camera3d>)]
pub struct EnvironmentMapLight {
    pub diffuse_map: Handle<Image>,
    pub specular_map: Handle<Image>,
}

impl EnvironmentMapLight {
    /// Whether or not all textures necessary to use the environment map
    /// have been loaded by the asset server.
    pub fn is_loaded(&self, images: &RenderAssets<Image>) -> bool {
        images.get(&self.diffuse_map).is_some() && images.get(&self.specular_map).is_some()
    }
}

pub fn get_bindings<'a>(
    environment_map_light: Option<&EnvironmentMapLight>,
    images: &'a RenderAssets<Image>,
    fallback_image_cubemap: &'a FallbackImageCubemap,
) -> (&'a TextureView, &'a TextureView, &'a Sampler) {
    let (diffuse_map, specular_map) = match (
        environment_map_light.and_then(|env_map| images.get(&env_map.diffuse_map)),
        environment_map_light.and_then(|env_map| images.get(&env_map.specular_map)),
    ) {
        (Some(diffuse_map), Some(specular_map)) => {
            (&diffuse_map.texture_view, &specular_map.texture_view)
        }
        _ => (
            &fallback_image_cubemap.texture_view,
            &fallback_image_cubemap.texture_view,
        ),
    };

    (diffuse_map, specular_map, &fallback_image_cubemap.sampler)
}

pub fn get_bind_group_layout_entries() -> [BindGroupLayoutEntryBuilder; 3] {
    [
        texture_cube(TextureSampleType::Float { filterable: true }),
        texture_cube(TextureSampleType::Float { filterable: true }),
        sampler(SamplerBindingType::Filtering),
    ]
}