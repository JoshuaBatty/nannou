struct FragmentOutput {
    @location(0) f_color: vec4<f32>,
};

@fragment
fn main(
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
) -> FragmentOutput {
    let light: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let brightness: f32 = dot(normalize(normal), normalize(light));
    let dark_color: vec3<f32> = vec3<f32>(0.1, 0.1, 0.1) * color.rgb;
    let out_color = vec4<f32>(mix(dark_color, color.rgb, vec3<f32>(brightness)), color.a);
    //return FragmentOutput(out_color);
    return FragmentOutput(color);
}

// @fragment
// fn main(
//     @location(0) normal: vec3<f32>,
//     @location(1) color: vec4<f32>,
// ) -> FragmentOutput {
//     // Debug coloring based on normal direction
//     var face_color: vec3<f32> = vec3<f32>(0.5, 0.5, 0.5); // Default gray
    
//     let abs_normal = abs(normal);
    
//     // X-facing faces
//     if (abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z) {
//         if (normal.x > 0.0) {
//             face_color = vec3<f32>(1.0, 0.0, 0.0); // Right face (red)
//         } else {
//             face_color = vec3<f32>(0.5, 0.0, 0.0); // Left face (dark red)
//         }
//     } 
//     // Y-facing faces
//     else if (abs_normal.y > abs_normal.x && abs_normal.y > abs_normal.z) {
//         if (normal.y > 0.0) {
//             face_color = vec3<f32>(0.0, 1.0, 0.0); // Top face (green)
//         } else {
//             face_color = vec3<f32>(0.0, 0.5, 0.0); // Bottom face (dark green)
//         }
//     } 
//     // Z-facing faces
//     else {
//         if (normal.z > 0.0) {
//             face_color = vec3<f32>(0.0, 0.0, 1.0); // Front face (blue)
//         } else {
//             face_color = vec3<f32>(0.0, 0.0, 0.5); // Back face (dark blue)
//         }
//     }
    
//     return FragmentOutput(vec4<f32>(face_color * color.rgb, color.a));
// }