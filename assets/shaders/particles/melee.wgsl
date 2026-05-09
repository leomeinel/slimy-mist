#import bevy_enoki::particle_vertex_out::VertexOutput

const HALF_UV = vec2<f32>(0.5);
const RADIUS_SQ = 0.5 * 0.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = in.uv - HALF_UV;
    let length_sq = dot(dist, dist);
    let is_within_circle = length_sq <= RADIUS_SQ;

    return in.color * select(0., 1., is_within_circle);
}
