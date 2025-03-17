use nannou::prelude::*;
use std::sync::{Arc, Mutex};

mod data;

#[derive(Clone)]
struct Model {
    graphics: Arc<Mutex<Graphics>>,
    grid_points: Vec<Vec3>,
    min_bounds: Vec3,
    max_bounds: Vec3,
}

struct Graphics {
    vertex_buffer: wgpu::Buffer,
    normal_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

// The vertex type that we will use to represent a point on our triangle.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    position: (f32, f32, f32),
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    world: Mat4,
    view: Mat4,
    proj: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Instance {
    transformation: Mat4,
    color: [f32; 4],
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn main() {
    nannou::app(model).render(render).run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window::<Model>().hdr(true).size(1024, 576).build();

    // The gpu device associated with the window's swapchain
    let window = app.window(w_id);
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let UVec2 { x: win_w, y: win_h } = window.size_pixels();

    // Load shader modules.
    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Create the vertex, normal and index buffers.
    let vertices_bytes = vertices_as_bytes(&data::VERTICES);
    let normals_bytes = normals_as_bytes(&data::NORMALS);
    let indices_bytes = indices_as_bytes(&data::INDICES);
    let vertex_usage = wgpu::BufferUsages::VERTEX;
    let index_usage = wgpu::BufferUsages::INDEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage: vertex_usage,
    });
    let normal_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: normals_bytes,
        usage: vertex_usage,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: indices_bytes,
        usage: index_usage,
    });

    let size = 15;
    let spacing = 5.0;
    let grid_points = make_3d_grid(size, spacing);
    let (min_bounds, max_bounds) = calculate_grid_bounding_box(size, spacing);
    println!("Number of points in the grid: {}", grid_points.len());
    println!("min_bounds: {}, max_bounds: {}", min_bounds, max_bounds);


    // Create the depth texture.
    let depth_texture = create_depth_texture(&device, [win_w, win_h], DEPTH_FORMAT, msaa_samples);
    let depth_texture_view = depth_texture.view().build();

    // Create the uniform buffer.
    let uniforms = create_uniforms(0.0, [win_w, win_h]);
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    // Create the render pipeline.
    let bind_group_layout = create_bind_group_layout(&device);
    let bind_group = create_bind_group(&device, &bind_group_layout, &uniform_buffer);
    let pipeline_layout = create_pipeline_layout(&device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        &device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        DEPTH_FORMAT,
        msaa_samples,
    );

    let graphics = Arc::new(Mutex::new(Graphics {
        vertex_buffer,
        normal_buffer,
        index_buffer,
        uniform_buffer,
        depth_texture,
        depth_texture_view,
        bind_group,
        render_pipeline,
    }));

    Model { graphics, grid_points, min_bounds, max_bounds }
}

fn calculate_grid_bounding_box(size: usize, spacing: f32) -> (Vec3, Vec3) {
    let offset = (size as f32 - 1.0) * spacing / 2.0;
    let min_point = vec3(-offset, -offset, -offset);
    let max_point = vec3(offset, offset, offset);
    (min_point, max_point)
}

fn make_instance(
    position: Vec3,
    local_rotation: f32,
    scale: f32,
    color: [f32; 4],
) -> Instance {
    let scale_m = Mat4::from_scale(Vec3::splat(scale));
    let local_rotation_m = Mat4::from_rotation_y(local_rotation);
    let translation_m = Mat4::from_translation(position);

    Instance {
        transformation: translation_m * local_rotation_m * scale_m,
        color,
    }
}

fn render(app: &RenderApp, model: &Model, frame: Frame) {
    let mut g = model.graphics.lock().unwrap();

    // If the window has changed size, recreate our depth texture to match.
    let depth_size = g.depth_texture.size();
    let frame_size = frame.texture_size();
    let device = frame.device();
    if frame_size != depth_size {
        let depth_format = g.depth_texture.format();
        let sample_count = frame.resolve_target_msaa_samples();
        g.depth_texture = create_depth_texture(&device, frame_size, depth_format, sample_count);
        g.depth_texture_view = g.depth_texture.view().build();
    }

    let t = app.time();
    let world_rotation = 0.05f32 * t;
    let rotation_matrix = Mat4::from_rotation_y(world_rotation);
    let camera_position = rotation_matrix.transform_point3(EYE);
    
    let mut instances_with_distance: Vec<_> = model
        .grid_points
        .iter()
        .enumerate()
        .map(|(i, position)| {
            // use distance from center:
            let opacity = 1.0 - (position.length() / 50.0).min(1.0);
            let norm_i = map_range(i, 0, model.grid_points.len(), 0.0, 1.0);
            let lfo = map_range((t * norm_i).sin(), 0.0, 1.0, 0.0, 1.0);
            let xp = map_range(position.x, model.min_bounds.x,model.max_bounds.x,0.0,1.0);
            let yp = map_range(position.y, model.min_bounds.y,model.max_bounds.y,0.0,1.0);
            let zp = map_range(position.z, model.min_bounds.z,model.max_bounds.z,0.0,1.0);
            
            let x = (xp + t * 0.5).sin().abs();
            let y = (yp + t * 0.25).sin().abs(); 
            let z = (zp + t * 0.125).cos().abs();
            let opacity = x * y * z * lfo;
            (
                camera_position.distance(*position),
                make_instance(
                    position.clone(),
                    0.0,
                    2.475,
                    [x, y, z, opacity],
                )
            )
        })
        .collect();

    // Sort by distance, furthest first for correct transparency
    instances_with_distance.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Extract just the instances after sorting
    let instances: Vec<_> = instances_with_distance.into_iter().map(|(_, instance)| instance).collect();
    
    // Continue with the rest of your render function as before
    let instances_bytes = instances_as_bytes(&instances);

    let usage = wgpu::BufferUsages::VERTEX;
    let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: instances_bytes,
        usage,
    });

    // Update the uniforms (rotate around the teapot).
    let uniforms = create_uniforms(world_rotation, frame_size);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    let mut encoder = frame.command_encoder();
    encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.resolve_target_view().unwrap(), |color| color)
        // We'll use a depth texture to assist with the order of rendering fragments based on depth.
        .depth_stencil_attachment(&g.depth_texture_view, |depth| depth)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &g.bind_group, &[]);
    render_pass.set_pipeline(&g.render_pipeline);
    render_pass.set_vertex_buffer(0, g.vertex_buffer.slice(..));
    render_pass.set_vertex_buffer(1, g.normal_buffer.slice(..));
    render_pass.set_vertex_buffer(2, instance_buffer.slice(..));
    render_pass.set_index_buffer(g.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..instances.len() as u32;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);
}

pub const EYE: Vec3 = Point3::new(0.3,0.3,1.0);

fn create_uniforms(world_rotation: f32, [w, h]: [u32; 2]) -> Uniforms {
    let world_rotation = Mat4::from_rotation_y(world_rotation);
    let aspect_ratio = w as f32 / h as f32;
    let fov_y = std::f32::consts::FRAC_PI_2;
    let near = 0.01;
    let far = 100.0;
    let proj = Mat4::perspective_rh_gl(fov_y, aspect_ratio, near, far);
    let target = Point3::ZERO;
    let up = Vec3::Y;
    let view = Mat4::look_at_rh(EYE, target, up);
    let world_scale = Mat4::from_scale(Vec3::splat(0.015));
    Uniforms {
        world: world_rotation,
        view: (view * world_scale).into(),
        proj: proj.into(),
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .buffer::<Uniforms>(uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(dst_format)
        // .color_blend(wgpu::BlendComponent::REPLACE)
        // .alpha_blend(wgpu::BlendComponent::REPLACE)
        .color_blend(wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        })
        .alpha_blend(wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        })
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x3])
        .add_vertex_buffer::<Normal>(&wgpu::vertex_attr_array![1 => Float32x3])
        // TODO: this can use the macro again when https://github.com/gfx-rs/wgpu/issues/836 is fixed
        .add_instance_buffer::<Instance>(&[
            wgpu::VertexAttribute {
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 0,
            },
            wgpu::VertexAttribute {
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 1,
            },
            wgpu::VertexAttribute {
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 2,
            },
            wgpu::VertexAttribute {
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 3,
            },
            wgpu::VertexAttribute {
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
                offset: std::mem::size_of::<[f32; 4]>() as u64 * 4,
            },
        ])
        .depth_stencil(wgpu::DepthStencilState {
            format: depth_format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
        .cull_mode(None)
        .depth_format(depth_format)
        .sample_count(sample_count)
        .build(device)
}

/* Creates a 3D grid of positions for teapot instances
 * The grid is centered around the origin (0,0,0)
 */
fn make_3d_grid(size: usize, spacing: f32) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(size * size * size);
    let offset = (size as f32 - 1.0) * spacing / 2.0;
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                let x_pos = (x as f32 * spacing) - offset;
                let y_pos = (y as f32 * spacing) - offset;
                let z_pos = (z as f32 * spacing) - offset;
                points.push(vec3(x_pos, y_pos, z_pos));
            }
        }
    }
    points
}

// See the `nannou::wgpu::bytes` documentation for why the following are necessary.

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn normals_as_bytes(data: &[Normal]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn indices_as_bytes(data: &[u16]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}

fn instances_as_bytes(data: &[Instance]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}
