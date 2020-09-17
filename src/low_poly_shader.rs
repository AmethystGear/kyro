
use amethyst::renderer::{RenderBase3D, rendy::{
    mesh::{AsVertex, Position, TexCoord, VertexFormat, Normal},
    shader::SpirvShader, hal::pso::ShaderStageFlags,
}, pass::Base3DPassDef, mtl::{TexAlbedo, TexEmission}, skinning::JointCombined};
lazy_static::lazy_static! {
    static ref POS_TEX_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../assets/shaders/compiled/pos_norm_tex.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref POS_TEX_SKIN_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../assets/shaders/compiled/pos_norm_tex_skin.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref FLAT_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../assets/shaders/compiled/shaded.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}
/// Implementation of `Base3DPassDef` to describe a flat 3D pass
#[derive(Debug)]
pub struct MyFlatPassDef;
impl Base3DPassDef for MyFlatPassDef {
    const NAME: &'static str = "Flat";
    type TextureSet =(TexAlbedo, TexEmission);
    fn vertex_shader() -> &'static SpirvShader {
        &POS_TEX_VERTEX
    }
    fn vertex_skinned_shader() -> &'static SpirvShader {
        &POS_TEX_SKIN_VERTEX
    }
    fn fragment_shader() -> &'static SpirvShader {
        &FLAT_FRAGMENT
    }
    fn base_format() -> Vec<VertexFormat> {
        vec![Position::vertex(), Normal::vertex(), TexCoord::vertex()]
    }
    fn skinned_format() -> Vec<VertexFormat> {
        vec![
            Position::vertex(),
            Normal::vertex(),
            TexCoord::vertex(),
            JointCombined::vertex(),
        ]
    }
}

/// Describes a Flat 3D pass
pub type MyRenderFlat3D = RenderBase3D<MyFlatPassDef>;