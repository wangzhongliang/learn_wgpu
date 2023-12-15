// vertex shader
struct Camera{
    view_pos: vec4f,
    view_proj: mat4x4f
};
@group(1) @binding(0)
var<uniform> camera: Camera;

struct PointLight {
    position: vec3f,
    intensity: f32,
    color: vec3f,
    constant: f32,
    linear: f32,
    quadratic: f32
};
@group(2) @binding(0)
var<uniform> light: PointLight;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
    @location(3) tangent: vec3f,
    @location(4) bitangent: vec3f
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) tangent_position: vec3f,
    @location(2) tangent_light_position: vec3f,
    @location(3) tangent_view_position: vec3f
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4f,
    @location(6) model_matrix_1: vec4f,
    @location(7) model_matrix_2: vec4f,
    @location(8) model_matrix_3: vec4f,

    @location(9) normal_matrix_0: vec3f,
    @location(10) normal_matrix_1: vec3f,
    @location(11) normal_matrix_2: vec3f,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4f(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let normal_matrix = mat3x3f(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let tangent_matrix = transpose(mat3x3f(
        world_tangent,
        world_bitangent,
        world_normal,
    ));
    let world_position = model_matrix * vec4f(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    out.tangent_light_position = tangent_matrix * light.position;
    return out;
}

// fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // flip Y
    let uv = vec2f(in.tex_coords.x, 1. - in.tex_coords.y);
    let object_color = textureSample(t_diffuse, s_diffuse, uv);
    let object_normal = textureSample(t_normal, s_normal, uv);
    
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir_without_normalized = in.tangent_light_position - in.tangent_position;
    let light_dir = normalize(light_dir_without_normalized);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);
    let half_dir = normalize(view_dir + light_dir);
    let distance = length(light_dir_without_normalized);
    let attenuation = 1.0 / (light.constant+light.linear*distance+light.quadratic*(distance*distance));
    
    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0) * attenuation;
    let diffuse_color = light.color * diffuse_strength * light.intensity;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0) * attenuation;
    let specular_color = specular_strength * light.color * light.intensity;

    let ambient_strength = 0.1 * attenuation;
    let ambient_color = light.color * ambient_strength;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.rgb;
    return vec4f(result, object_color.a);
}