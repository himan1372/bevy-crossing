// Retail inventory's cloth colour and frame coverage use independent tiles.
// All parts sample one panel-local pattern; masks retain their own UVs.
#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> offset: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var cloth: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var cloth_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var mask: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var mask_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2((in.world_position.x-offset.x)/16.0+160.0,
                     120.0-in.world_position.y/16.0);
    let uv = (pixel+vec2(offset.y,0.0))/32.0;
    let rgb = textureSample(cloth,cloth_sampler,uv).rgb;
    let alpha = textureSample(mask,mask_sampler,in.uv).a;
    if alpha < 0.01 { discard; }
    let linear = select(rgb/12.92,pow((rgb+vec3(0.055))/1.055,vec3(2.4)),rgb>vec3(0.04045));
    return vec4(linear,alpha);
}
