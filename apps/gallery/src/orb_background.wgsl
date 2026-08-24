struct Uniforms { resolution_time: vec4<f32> }
@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    return vec4(p[i], 0.0, 1.0);
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p); let f = fract(p); let s = f*f*(3.0-2.0*f);
    return mix(mix(hash21(i),hash21(i+vec2(1,0)),s.x),
               mix(hash21(i+vec2(0,1)),hash21(i+vec2(1,1)),s.x),s.y);
}
fn fbm(p0: vec2<f32>) -> f32 {
    var p=p0; var v=0.0; var a=0.5;
    for(var i=0;i<5;i+=1){ v += a*noise(p); p=mat2x2(1.62,1.15,-1.15,1.62)*p; a*=0.5; }
    return v;
}

@fragment fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let res=u.resolution_time.xy; let t=u.resolution_time.z;
    let screen=pos.xy/res;
    var p=(pos.xy-res*vec2(0.62,0.47))/res.y;
    let r=length(p); let edge=smoothstep(0.335,0.328,r);

    let n=normalize(vec3(p/0.335, sqrt(max(0.0,1.0-r*r/(0.335*0.335)))));
    let flow=fbm(p*5.0+vec2(t*0.10,-t*0.075));
    let flow2=fbm(p*8.0+vec2(-t*0.07,t*0.09)+11.0);
    let band=0.5+0.5*sin(p.x*11.0-p.y*7.0+flow*6.0+t*0.55);
    let cyan=vec3(0.12,0.84,1.0); let blue=vec3(0.16,0.32,1.0);
    let violet=vec3(0.62,0.18,1.0); let rose=vec3(1.0,0.24,0.62);
    var orb=mix(blue,violet,smoothstep(0.2,0.85,flow+p.y*0.7));
    orb=mix(orb,cyan,smoothstep(0.58,0.95,band));
    orb=mix(orb,rose,smoothstep(0.70,0.98,flow2-p.x*0.65));
    let light=normalize(vec3(-0.55,-0.72,1.15));
    let diffuse=max(dot(n,light),0.0); let rim=pow(1.0-max(n.z,0.0),2.6);
    orb*=0.34+0.88*diffuse; orb+=rim*mix(cyan,rose,0.32)*1.35;
    orb+=pow(max(dot(n,light),0.0),48.0)*vec3(1.0,0.94,1.0)*1.35;
    orb+=smoothstep(0.012,0.0,abs(r-0.235-flow*0.018))*cyan*0.18;

    let glow=exp(-max(r-0.30,0.0)*15.0)*0.20;
    let vignette=1.0-0.33*length(screen-0.5);
    var bg=mix(vec3(0.018,0.025,0.060),vec3(0.048,0.022,0.095),screen.y);
    bg+=glow*mix(blue,violet,0.45);
    let star=step(0.9978,hash21(floor(pos.xy/3.0)))*smoothstep(0.35,0.0,r)*0.38;
    bg+=star;
    let halo=smoothstep(0.004,0.0,abs(r-(0.405+0.008*sin(atan2(p.y,p.x)*3.0+t*0.35))))*0.20;
    bg+=halo*cyan;
    let color=mix(bg,orb,edge)*vignette;
    return vec4(color,1.0);
}
