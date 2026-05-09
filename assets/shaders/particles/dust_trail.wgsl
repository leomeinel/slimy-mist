#import bevy_enoki::particle_vertex_out::VertexOutput

const HALF_UV = vec2<f32>(0.5);
const RADIUS_SQ = 0.5 * 0.5;
const INV_RADIUS_SQ = 1. / RADIUS_SQ;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = in.uv - HALF_UV;
    let length_sq = dot(dist, dist);
    let is_within_circle = length_sq <= RADIUS_SQ;

    let radius_delta_frac = length_sq * INV_RADIUS_SQ;
    let falloff = smoothstep(1., 0., radius_delta_frac);
    let attenuation = falloff * falloff;

    return in.color * attenuation * select(0., 1., is_within_circle);
}
