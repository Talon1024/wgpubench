struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec3<f32>,
};

const SQUARE_SIZE: f32 = 8.0; // pixels
const PI: f32 = 3.14159265358979323846;

@group(0) @binding(0) var<uniform> screen_size: vec2<u32>;

@vertex
fn vertex_main(
    @location(0) inst_cnt_hue: vec3<f32>,
    @location(1) vert_pos_uv: vec4<f32>
) -> VertexOutput {
    var gazouta: VertexOutput;
    let ipos = inst_cnt_hue.xy;
    let hue = inst_cnt_hue.z;
    let vpos = vert_pos_uv.xy;
    let vuv = vert_pos_uv.zw;
    gazouta.position = vec4<f32>(vpos, 0.0, 1.0);
    gazouta.uv = vuv;
    gazouta.colour = 
        // From https://github.com/Talon1024/shader-shite/blob/master/hsl.frag
        clamp(cos(hue - PI * 2. * vec3<f32>(0., 0.333333333333, 0.666666666666)) + .5, vec3(0.0), vec3(1.0));
    return gazouta;
}

@fragment
fn pixel_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(vertex.colour, 1.0);
}
