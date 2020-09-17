#version 450

#include "header/math.frag"

#include "header/environment.frag"

layout(set = 1, binding = 0) uniform Material {
    UvOffset uv_offset;
    float alpha_cutoff;
};

layout(set = 1, binding = 1) uniform sampler2D albedo;
layout(set = 1, binding = 2) uniform sampler2D emission;

layout(location = 0) in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    vec4 color;
} vertex;

layout(location = 0) out vec4 out_color;


void main() {
    vec2 final_tex_coords   = tex_coords(vertex.tex_coord, uv_offset);
    vec4 albedo_alpha       = texture(albedo, final_tex_coords);
    float alpha             = albedo_alpha.a;
    if(alpha < alpha_cutoff) discard;

    vec3 albedo = albedo_alpha.rgb;
    vec3 emission = texture(emission, final_tex_coords).rgb;

    vec3 lighting = vec3(0.0);
    vec3 xTangent = dFdx( vertex.normal );
    vec3 yTangent = dFdy( vertex.normal );
    vec3 normal = normalize( cross( xTangent, yTangent ) );
    for (uint i = 0u; i < directional_light_count; i++) {
        vec3 dir = dlight[i].direction;
        float diff = max(dot(normal, -dir), 0.0);
        vec3 diffuse = diff * dlight[i].color;
        lighting += diffuse * dlight[i].intensity;
    }
    lighting += ambient_color;
    out_color = vec4(lighting * albedo + emission, alpha) * vertex.color;
}
