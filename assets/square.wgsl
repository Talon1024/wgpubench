struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec3<f32>,
};

const SQUARE_SIZE: f32 = 8.0; // pixels
const PI: f32 = 3.14159265358979323846;

@group(0) @binding(0) var<uniform> screen_size: vec2<u32>;
@group(0) @binding(1) var flare_texture: texture_2d<f32>;
@group(0) @binding(2) var flare_sampler: sampler;

@vertex
fn vertex_main(
    @location(0) inst_pos_hue: vec4<f32>,
    @location(1) vert_pos_uv: vec4<f32>
) -> VertexOutput {
    var gazouta: VertexOutput;
    let ipos = inst_pos_hue.xy;
    let hue = inst_pos_hue.z;
    let index = bitcast<u32>(inst_pos_hue.w);
    let vpos = vert_pos_uv.xy;
    let vuv = vert_pos_uv.zw;
    let depth = select(0.25, 0.125, index % 2u == 0u);
    gazouta.position = vec4<f32>(vpos + ipos, depth, 1.0);
    gazouta.uv = vuv;
    gazouta.colour = 
        // From https://github.com/Talon1024/shader-shite/blob/master/hsl.frag
        clamp(cos(hue - PI * 2. * vec3<f32>(0., 0.333333333333, 0.666666666666)) + .5, vec3(0.0), vec3(1.0));
    return gazouta;
}

@fragment
fn pixel_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let tex_colour = textureSample(flare_texture, flare_sampler, vertex.uv);
    // Assuming green and blue channels are the same, I can change the
    // hue easily
    let blend_factor = tex_colour.r - tex_colour.g;
    let colour = mix(vec3(1.0), vertex.colour, blend_factor);
    return vec4(colour, tex_colour.a);
}
