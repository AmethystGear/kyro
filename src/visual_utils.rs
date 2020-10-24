use amethyst::{
    assets::{self, AssetStorage, Handle},
    renderer::{mtl, palette::LinSrgba, rendy::texture, types},
};

pub fn create_material(
    loader: &assets::Loader,
    tex_storage: &AssetStorage<types::Texture>,
    mat_storage: &AssetStorage<mtl::Material>,
    mat_defaults: &mtl::MaterialDefaults,
    color: LinSrgba,
    metallic: f32,
    roughness: f32,
) -> Handle<mtl::Material> {
    // Material creation
    let asset_storage = tex_storage;
    let albedo = loader.load_from_data(
        texture::palette::load_from_linear_rgba(color).into(),
        (),
        &asset_storage,
    );

    let metallic_roughness = loader.load_from_data(
        texture::palette::load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0))
            .into(),
        (),
        &asset_storage,
    );

    let asset_storage = mat_storage;
    let mat_defaults = mat_defaults.0.clone();

    loader.load_from_data(
        mtl::Material {
            albedo,
            metallic_roughness,
            ..mat_defaults
        },
        (),
        &asset_storage,
    )
}
