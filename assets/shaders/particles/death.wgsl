#import bevy_enoki::particle_vertex_out::VertexOutput

const HALF_UV = vec2<f32>(0.5);
const LINE_WIDTH = 0.25;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = in.uv - HALF_UV;
    let dist_main_diag = dist.y - dist.x;
    let dist_cross_diag = dist.y + dist.x;
    let is_within_diag_plus = abs(dist_main_diag) <= LINE_WIDTH || abs(dist_cross_diag) <= LINE_WIDTH;

    return in.color * select(0., 1., is_within_diag_plus);
}
