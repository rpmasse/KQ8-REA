//! Hardware regression for the production animated-material shader's instance
//! tag lookup. Ignored in ordinary tests: needs a Windows hardware GPU. No game
//! assets, window, presentation or visual inspection. Actual production shader
//! functions and the entry-point call are extracted, then exercised numerically.

use std::{
    borrow::Cow,
    future::Future,
    sync::{mpsc, Arc},
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

use wgpu::util::DeviceExt;

const DEADLINE: Duration = Duration::from_secs(15);
const FIRST_INSTANCES: [u32; 3] = [0, 17, 131_071];
const WIDTH: u32 = 12;
const HEIGHT: u32 = 12;
const ROW_BYTES: u32 = 256;

struct ThreadWake(std::thread::Thread);

impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn wait<F: Future>(future: F, what: &str) -> Result<F::Output, String> {
    let mut future = std::pin::pin!(future);
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let started = Instant::now();
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return Ok(value);
        }
        let Some(remaining) = DEADLINE.checked_sub(started.elapsed()) else {
            return Err(format!("{what}: exceeded 15-second deadline"));
        };
        std::thread::park_timeout(remaining.min(Duration::from_millis(10)));
    }
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    words: &[u32],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let bytes: Vec<u8> = words.iter().flat_map(|v| v.to_le_bytes()).collect();
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &bytes,
        usage,
    })
}

const PRODUCTION: &str = include_str!("object_light.wesl");

fn declaration<'a>(source: &'a str, marker: &str, semicolon: bool) -> Result<&'a str, String> {
    let start = source
        .find(marker)
        .ok_or_else(|| format!("production shader missing {marker}"))?;
    let open = start
        + source[start..]
            .find('{')
            .ok_or("declaration has no brace")?;
    let mut depth = 0usize;
    for (offset, c) in source[open..].char_indices() {
        if c == '{' {
            depth += 1;
        }
        if c == '}' {
            depth -= 1;
            if depth == 0 {
                let end = open + offset + 1;
                if semicolon && source.as_bytes().get(end) != Some(&b';') {
                    return Err(format!(
                        "production declaration {marker} changed terminator"
                    ));
                }
                return Ok(&source[start..end + usize::from(semicolon)]);
            }
        }
    }
    Err(format!("unclosed production declaration {marker}"))
}

fn production_shader() -> Result<String, String> {
    // The final overload is ANIMATED. Retain its signature and body, including
    // the instance-index parameter, and retain the actual entry-point call.
    // Reverting either part of the fix therefore changes this GPU test too.
    let animated = PRODUCTION
        .rfind("fn model_corner(")
        .ok_or("missing animated model_corner")?;
    let call = PRODUCTION
        .lines()
        .find(|line| line.trim().starts_with("let corner = model_corner("))
        .ok_or("missing production model_corner call")?;
    let mut shader = String::new();
    for (source, marker, semicolon) in [
        (PRODUCTION, "struct NodeFrame {", true),
        (PRODUCTION, "struct Corner {", true),
        (PRODUCTION, "fn game_to_bevy(", false),
        (PRODUCTION, "fn node_corner(", false),
        (&PRODUCTION[animated..], "fn model_corner(", false),
    ] {
        // Only remove the WESL module qualification; get_tag's lookup below
        // supplies synthetic mesh tags with the same scalar argument contract.
        shader.push_str(
            &declaration(source, marker, semicolon)?.replace("mesh_functions::get_tag", "get_tag"),
        );
        shader.push('\n');
    }
    shader.push_str(
        r#"
@group(0) @binding(0) var<storage, read> tags: array<u32>;
@group(0) @binding(1) var<storage, read> anim_points: array<u32>;
@group(0) @binding(2) var<storage, read> anim_normals: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> anim_nodes: array<NodeFrame>;
@group(0) @binding(4) var<storage, read> anim_joins: array<vec4<f32>>;
struct Settings { game_units: f32, };
const settings = Settings(1.0);
fn get_tag(index: u32) -> u32 { return tags[index]; }
// The production ANIMATED input layout, including its real vertex attributes.
struct ObjectIn {
    @builtin(instance_index) instance_index: u32,
    @location(2) uv: vec2<f32>,
    @location(9) corner: vec2<u32>,
};
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) value: vec4<f32>,
};
@vertex
fn vertex(v: ObjectIn, @builtin(vertex_index) vertex_id: u32) -> Output {
"#,
    );
    shader.push_str(call);
    shader.push_str(
        r#"
    let corners = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0));
    let column = vertex_id / 4u;
    let raster_corner = corners[vertex_id % 4u];
    let x = (f32(column) + (raster_corner.x + 1.0) * 0.5) / 3.0 * 2.0 - 1.0;
    return Output(vec4<f32>(x, raster_corner.y, 0.0, 1.0), vec4<f32>(corner.position, 1.0));
}
@fragment
fn fragment(input: Output) -> @location(0) vec4<f32> {
    return input.value;
}
"#,
    );
    Ok(shader)
}

fn run_regression() -> Result<(), String> {
    // Match optimized production FXC without environment overrides. The test
    // deliberately never uses indirect-count, which Bevy already avoids on DX12.
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::DX12;
    descriptor.flags = wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL;
    descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::Fxc;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = wait(
        instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            ..Default::default()
        }),
        "adapter request",
    )?
    .map_err(|e| format!("adapter request: {e}"))?;
    let info = adapter.get_info();
    println!("GPU regression adapter: {info:?}");
    if info.backend != wgpu::Backend::Dx12 || info.device_type == wgpu::DeviceType::Cpu {
        return Err("regression requires an actual DX12 hardware adapter".into());
    }
    let draw_features = wgpu::Features::INDIRECT_FIRST_INSTANCE;
    if !adapter.features().contains(draw_features) {
        return Err(format!(
            "adapter cannot exercise the regression: missing {:?}",
            draw_features - adapter.features()
        ));
    }
    let (device, queue) = wait(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("KQ8 renderer indexed draw regression"),
            required_features: draw_features,
            ..Default::default()
        }),
        "device request",
    )?
    .map_err(|e| format!("device request: {e}"))?;
    println!(
        "GPU regression actual features={:?}; production animated shader",
        device.features()
    );

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("instance index and tag lookup"),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(production_shader()?)),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("instance index and tag lookup"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: 16,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 2,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Uint32x2,
                        offset: 8,
                        shader_location: 9,
                    },
                ],
            })],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    if let Some(error) = wait(scope.pop(), "pipeline validation")? {
        return Err(format!("pipeline validation: {error}"));
    }

    let mut tags = vec![0u32; FIRST_INSTANCES[2] as usize + 1];
    let mut nodes = vec![0u32; 3 * 20];
    for (node, first) in FIRST_INSTANCES.into_iter().enumerate() {
        tags[first as usize] = node as u32;
        nodes[node * 20] = 1.0f32.to_bits();
        nodes[node * 20 + 5] = 1.0f32.to_bits();
        nodes[node * 20 + 10] = 1.0f32.to_bits();
        nodes[node * 20 + 3] = node as u32; // point-base bits in f32 row0.w
        nodes[node * 20 + 7] = node as u32; // normal-base bits in f32 row1.w
        nodes[node * 20 + 11] = node as u32; // next-node bits in f32 row2.w
        nodes[node * 20 + 19] = 1.0f32.to_bits();
    }
    let points = [
        u32::from_le_bytes([10, 20, 30, 0]),
        u32::from_le_bytes([40, 50, 60, 0]),
        u32::from_le_bytes([70, 80, 90, 0]),
    ];
    let inputs = [
        buffer(
            &device,
            "mesh tag lookup",
            &tags,
            wgpu::BufferUsages::STORAGE,
        ),
        buffer(
            &device,
            "packed model points",
            &points,
            wgpu::BufferUsages::STORAGE,
        ),
        buffer(
            &device,
            "packed model normals",
            &[0, 32767, 0, 32767, 0, 32767],
            wgpu::BufferUsages::STORAGE,
        ),
        buffer(
            &device,
            "production node layout",
            &nodes,
            wgpu::BufferUsages::STORAGE,
        ),
        buffer(
            &device,
            "unused smooth joins",
            &[0; 4],
            wgpu::BufferUsages::STORAGE,
        ),
    ];
    let entries: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tag lookup"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    });
    let vertices = buffer(
        &device,
        "production UV and corner attributes",
        &[0; 12 * 4],
        wgpu::BufferUsages::VERTEX,
    );
    let indices = buffer(
        &device,
        "quad indices",
        &[0, 1, 2, 2, 1, 3],
        wgpu::BufferUsages::INDEX,
    );
    let arguments: Vec<u32> = FIRST_INSTANCES
        .into_iter()
        .enumerate()
        .flat_map(|(i, first)| [6u32, 1, 0, i as u32 * 4, first])
        .collect();
    let indirect = buffer(
        &device,
        "three indexed arguments",
        &arguments,
        wgpu::BufferUsages::INDIRECT,
    );
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("numeric vertex output"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("numeric vertex readback"),
        size: u64::from(ROW_BYTES * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("direct, selected indirect, Bevy fallback"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        for row in 0..3u32 {
            pass.set_viewport(0.0, row as f32 * 4.0, WIDTH as f32, 4.0, 0.0, 1.0);
            pass.set_scissor_rect(0, row * 4, WIDTH, 4);
            match row {
                0 => {
                    for (i, first) in FIRST_INSTANCES.into_iter().enumerate() {
                        pass.draw_indexed(0..6, i as i32 * 4, first..first + 1);
                    }
                }
                1 => {
                    for i in 0..3u64 {
                        pass.draw_indexed_indirect(&indirect, i * 20);
                    }
                }
                _ => pass.multi_draw_indexed_indirect(&indirect, 0, 3),
            }
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_BYTES),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    let submission = queue.submit([encoder.finish()]);
    let (send, receive) = mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
    let started = Instant::now();
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(DEADLINE),
        })
        .map_err(|e| format!("GPU poll: {e}"))?;
    receive
        .recv_timeout(DEADLINE.saturating_sub(started.elapsed()))
        .map_err(|e| format!("readback callback: {e}"))?
        .map_err(|e| format!("readback map: {e}"))?;
    if let Some(error) = wait(scope.pop(), "draw validation")? {
        return Err(format!("draw validation: {error}"));
    }

    let data = readback
        .slice(..)
        .get_mapped_range()
        .map_err(|e| format!("mapped range: {e}"))?;
    let labels = [
        "direct control",
        "ordinary indirect",
        "Bevy multi-draw fallback",
    ];
    let mut mismatches = Vec::new();
    for (row, label) in labels.into_iter().enumerate() {
        for (column, first) in FIRST_INSTANCES.into_iter().enumerate() {
            let offset = (row * 4 + 2) * ROW_BYTES as usize + (column * 4 + 2) * 16;
            let actual: [f32; 4] = std::array::from_fn(|channel| {
                f32::from_le_bytes(
                    data[offset + channel * 4..offset + channel * 4 + 4]
                        .try_into()
                        .unwrap(),
                )
            });
            // Independent known positions of the three synthetic nodes after
            // the production game (x,y,z) -> Bevy (x,z,-y) conversion.
            let expected = [
                [10.0, 30.0, -20.0, 1.0],
                [40.0, 60.0, -50.0, 1.0],
                [70.0, 90.0, -80.0, 1.0],
            ][column];
            let okay = actual.iter().all(|v| v.is_finite()) && actual == expected;
            println!(
                "{} {label} first_instance={first}: GPU={actual:?}, expected={expected:?}",
                if okay { "PASS" } else { "FAIL" }
            );
            if !okay {
                mismatches.push(format!(
                    "{label} first_instance={first}: GPU={actual:?}, expected={expected:?}"
                ));
            }
        }
    }
    drop(data);
    readback.unmap();
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} indexed draw mismatches:\n{}",
            mismatches.len(),
            mismatches.join("\n")
        ))
    }
}

#[test]
#[ignore = "requires Windows DX12 hardware with INDIRECT_FIRST_INSTANCE; production animated shader and numerical GPU readback"]
fn windows_dx12_animated_material_preserves_instance_tag() {
    run_regression().expect("Windows DX12 production animated-material regression");
}
