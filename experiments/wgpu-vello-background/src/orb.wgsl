struct OrbUniforms {
    resolution_time: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: OrbUniforms;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let local = fract(p);
    let blend = local * local * (3.0 - 2.0 * local);
    return mix(
        mix(hash(cell), hash(cell + vec2<f32>(1.0, 0.0)), blend.x),
        mix(hash(cell + vec2<f32>(0.0, 1.0)), hash(cell + vec2<f32>(1.0, 1.0)), blend.x),
        blend.y,
    );
}

fn fbm(p0: vec2<f32>) -> f32 {
    var p = p0;
    var value = 0.0;
    var amplitude = 0.5;
    for (var index = 0; index < 5; index += 1) {
        value += amplitude * noise(p);
        p = mat2x2<f32>(1.7, 1.2, -1.2, 1.7) * p;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = uniforms.resolution_time.xy;
    let time = uniforms.resolution_time.z;
    var uv = (position.xy - resolution * 0.5) / resolution.y;
    uv.x += 0.28;

    let warp = vec2<f32>(
        fbm(uv * 3.2 + vec2<f32>(time * 0.13, -time * 0.09)),
        fbm(uv * 3.0 + vec2<f32>(-time * 0.11, time * 0.15) + 9.3),
    ) - 0.5;
    let warped = uv + warp * 0.16;
    let radius = length(warped);
    let sphere = smoothstep(0.46, 0.435, radius);

    let normal_z = sqrt(max(0.0, 1.0 - min(1.0, radius / 0.46) * min(1.0, radius / 0.46)));
    let normal = normalize(vec3<f32>(warped / 0.46, normal_z));
    let light = normalize(vec3<f32>(-0.55, -0.65, 1.0));
    let diffuse = max(0.0, dot(normal, light));
    let rim = pow(1.0 - max(normal.z, 0.0), 2.7);
    let liquid = fbm(warped * 5.5 + warp * 2.0 + time * 0.08);

    let cyan = vec3<f32>(0.04, 0.82, 1.0);
    let violet = vec3<f32>(0.52, 0.18, 1.0);
    let magenta = vec3<f32>(1.0, 0.16, 0.68);
    var orb = mix(violet, cyan, smoothstep(0.2, 0.85, liquid + warped.y * 0.55));
    orb = mix(orb, magenta, smoothstep(0.52, 0.95, -warped.x + liquid * 0.4));
    orb *= 0.42 + diffuse * 0.9;
    orb += rim * mix(cyan, magenta, 0.45) * 1.4;
    orb += pow(max(0.0, dot(normal, light)), 42.0) * 2.2;

    let glow = exp(-max(radius - 0.42, 0.0) * 13.0) * 0.22;
    let backdrop = mix(vec3<f32>(0.018, 0.025, 0.07), vec3<f32>(0.055, 0.025, 0.12), position.y / resolution.y);
    let stars = step(0.997, hash(floor(position.xy / 4.0))) * 0.32;
    let color = mix(backdrop + glow * vec3<f32>(0.2, 0.35, 0.9) + stars, orb, sphere);
    return vec4<f32>(color, 1.0);
}
