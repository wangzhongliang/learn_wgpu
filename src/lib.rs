mod texture;
mod vertex;
mod camera;
mod camera_controller;
mod global;
mod instance;
mod resources;
mod model;
mod light;

use camera::{Camera, CameraUniform, Projection};
use camera_controller::CameraController;
use instance::Instance;
use light::DrawLight;
use model::{DrawModel, Model};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;
use vertex::Vertex;
use cgmath::prelude::*;

use crate::{instance::InstanceRaw, texture::Texture, model::{ModelVertex, Mesh}, light::{PointLightUniform, DirectionalLightUniform, SpotLightUniform}};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// // 顶点数据
// const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
// ];
// // 索引数据
// const INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];
// 物体旋转速度
const ROTATION_SPEED: f32 = 1.0 * std::f32::consts::PI / 60.0;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    // num_indices: u32,
    // diffuse_texture: texture::Texture,
    // diffuse_bind_group: wgpu::BindGroup,
    camera: Camera,
    projection: Projection,
    camera_controller: CameraController,
    mouse_pressed: bool,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    obj_model: Model,
    light_uniform: SpotLightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    // light_render_pipeline: wgpu::RenderPipeline,
    // light_mesh: Mesh
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::all(), ..Default::default() });
        // Surface
        let surface = unsafe { instance.create_surface(window).unwrap() };
        // Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false, // 如果是true就会用软件级的渲染，如果是false就会用GPU
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        // Device, Queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        // SurfaceConfiguration
        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        // Image
        // let diffuse_bytes = include_bytes!("happy-tree.png");
        // let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ],
            label: Some("texture_bind_group_layout")
        });
        // let diffuse_bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         layout: &texture_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&diffuse_texture.view)
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
        //             }
        //         ],
        //         label: Some("diffuse_bind_group")
        //     }
        // );
        // Camera
        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        // Camera Controller
        let camera_controller = CameraController::new(4.0, 0.4);
        // Camera Uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor{
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ],
                label: Some("camera_bind_group_layout")
            }
        );
        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding()
                    }
                ],
                label: Some("camera_bind_group")
            }
        );

        // Light
        let light_uniform = SpotLightUniform::new([1.0, 1.0, 1.0], [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0], 1.0, cgmath::Deg(12.5).cos());
        // let light_uniform = DirectionalLightUniform::new([1.0, 1.0, -1.0], [1.0, 1.0, 1.0], 1.0);
        // let light_uniform = PointLightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0], 1.0);
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                // use copy_dst to update light position
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }]
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: None, 
            layout: &light_bind_group_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding()
            }] 
        });

        // instances
        const SPACE_BETWEEN: f32 = 3.0;
        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);
        
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = SPACE_BETWEEN * (cgmath::Vector3::new(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT);
                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(45.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };
                Instance {
                    position,
                    rotation
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            }
        );

        // Depth Texture
        let depth_texture: Texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        
        // Render Pipeline
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/spotlight_shader.wgsl").into())
            };
            let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout
                ],
                push_constant_ranges: &[]
            });
            create_render_pipeline(
                &device, 
                &render_pipeline_layout, 
                config.format, 
                Some(Texture::DEPTH_FORMAT), 
                &[ModelVertex::desc(), InstanceRaw::desc()],
                shader
            )
        };

        // Light Render 
        // let light_render_pipeline = {
        //     let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
        //         label: Some("Light Pipeline Layout"), 
        //         bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
        //         push_constant_ranges: &[] 
        //     });
        //     let shader = wgpu::ShaderModuleDescriptor {
        //         label: Some("Light Shader"),
        //         source: wgpu::ShaderSource::Wgsl(include_str!("light/light.wgsl").into())
        //     };
        //     create_render_pipeline(
        //         &device, 
        //         &layout, 
        //         config.format, 
        //         Some(Texture::DEPTH_FORMAT), 
        //         &[ModelVertex::desc()], 
        //         shader
        //     )
        // };

        // Clear Color
        let clear_color = wgpu::Color::BLACK;
        // // Vertex Bufer
        // let vertex_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Vertex Buffer"),
        //         contents: bytemuck::cast_slice(VERTICES),
        //         usage: wgpu::BufferUsages::VERTEX
        //     }
        // );
        // let index_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Index Buffer"),
        //         contents: bytemuck::cast_slice(INDICES),
        //         usage: wgpu::BufferUsages::INDEX
        //     }
        // );
        // // indices Number
        // let num_indices = INDICES.len() as u32;

        // load obj
        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();

        // light mesh
        let light_vertices: &[ModelVertex] = &[
            ModelVertex { position: [-0.5, -0.5, -0.5], ..Default::default() },
            ModelVertex { position: [0.5, -0.5, -0.5], ..Default::default() },             
            ModelVertex { position: [0.5, 0.5, -0.5], ..Default::default() },           
            ModelVertex { position: [-0.5, 0.5, -0.5], ..Default::default() },             
            ModelVertex { position: [-0.5, -0.5, 0.5], ..Default::default() },
            ModelVertex { position: [0.5, -0.5, 0.5], ..Default::default() },             
            ModelVertex { position: [0.5, 0.5, 0.5], ..Default::default() },          
            ModelVertex { position: [-0.5, 0.5, 0.5], ..Default::default() },

            // ModelVertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], normal: [0., 0., 1.], tangent: [1.0, 0., 0.], bitangent: [0., 1., 0.] },
            // ModelVertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 0.0], normal: [0., 0., 1.], tangent: [1.0, 0., 0.], bitangent: [0., 1., 0.] },             
            // ModelVertex { position: [0.5, 0.5, 0.0], tex_coords: [1., 1.], normal: [0., 0., 1.], tangent: [1.0, 0., 0.], bitangent: [0., 1., 0.] },          
            // ModelVertex { position: [-0.5, 0.5, 0.0], ..Default::default() },             
        ];
        // 索引数据
        let light_indices: &[u16] = &[
            0, 1, 2,   // 三角面1
            2, 3, 0,   // 三角面2
            4, 5, 6,   // 三角面3
            6, 7, 4,   // 三角面4
            1, 0, 4,   // 三角面5
            4, 5, 1,   // 三角面6
            2, 1, 5,   // 三角面7
            5, 6, 2,   // 三角面8
            3, 2, 6,   // 三角面9
            6, 7, 3,   // 三角面10
            0, 3, 7,   // 三角面11
            7, 4, 0,   // 三角面12
        ];
        let light_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_light_vertex"),
            contents: bytemuck::cast_slice(light_vertices),
            usage: wgpu::BufferUsages::VERTEX
        });
        let light_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("default_light_indices"),
            contents: bytemuck::cast_slice(light_indices),
            usage: wgpu::BufferUsages::INDEX
        });
        println!("index length {}",light_indices.len());
        let light_mesh = Mesh {
            name: "light_mesh".to_owned(),
            vertex_buffer: light_vertex_buffer,
            index_buffer: light_index_buffer,
            num_elements: light_indices.len() as u32,
            material: 0
        };
        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            // vertex_buffer,
            // index_buffer,
            // num_indices,
            // diffuse_texture,
            // diffuse_bind_group,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            instances,
            instance_buffer,
            depth_texture,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            // light_render_pipeline,
            // light_mesh,
            mouse_pressed: false,
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.projection.resize(new_size.width, new_size.height);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { 
                input: KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { 
                delta,
                ..
            } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput { 
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false
        }
    }
    fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        // update instances rotation
        // self.update_instances();
        self.update_spot_light(dt);
    }
    // fn update_point_light(&mut self, dt: instant::Duration){
    //     let old_position = cgmath::Vector3::from(self.light_uniform.position);
    //     self.light_uniform.position = (cgmath::Quaternion::from_angle_y(cgmath::Deg(60.0 * dt.as_secs_f32())).rotate_vector(old_position)).into();
    //     self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    // }
    // fn update_directional_light(&mut self, dt: instant::Duration){
    //     let old_direction = cgmath::Vector3::from(self.light_uniform.direction);
    //     self.light_uniform.direction = (cgmath::Quaternion::from_angle_y(cgmath::Deg(60.0 * dt.as_secs_f32())).rotate_vector(old_direction)).into();
    //     self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    // }
    fn update_spot_light(&mut self, dt: instant::Duration){
        let old_direction = cgmath::Vector3::from(self.light_uniform.direction);
        self.light_uniform.direction = (cgmath::Quaternion::from_angle_y(cgmath::Deg(60.0 * dt.as_secs_f32())).rotate_vector(old_direction)).into();
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    }
    fn update_instances(&mut self){
        self.instances.iter_mut().for_each(|i| {
            let rad = cgmath::Rad(ROTATION_SPEED);
            let amount = cgmath::Quaternion::from_angle_y(rad);
            let current = i.rotation;
            i.rotation = amount * current;
        });
        let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instance_data));
    }
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        // 写在花括号里是为了让_render_pass在花括号执行完后销毁，
        // 否则_render_pass可能一直borrow着encoder，会造成encoder.finish销毁encoder时报错
        // 因为_render_pass可能在encoder销毁后才销毁
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                    view: &self.depth_texture.view, 
                    depth_ops: Some(wgpu::Operations { 
                        load: wgpu::LoadOp::Clear(1.0), 
                        store: true
                    }), 
                    stencil_ops: None 
                })
            });
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // render_pass.set_pipeline(&self.light_render_pipeline);
            // render_pass.draw_light_mesh(
            //     &self.light_mesh, 
            //     &self.camera_bind_group, 
            //     &self.light_bind_group
            // );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model, 
                0..self.instances.len() as u32, 
                &self.camera_bind_group,
                &self.light_bind_group
            );
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}



#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(800, 600));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { event: DeviceEvent::MouseMotion{ delta, }, .. } => if state.mouse_pressed {
            state.camera_controller.process_mouse(delta.0, delta.1)
        },
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() && !state.input(event) => {
            match event {
                #[cfg(not(target_arch="wasm32"))]
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => {}
            }
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let now = instant::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;
            state.update(dt);
            match state.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // 其他报错
                Err(e) => eprintln!("{:?}", e)
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    })
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
        label: Some("Render Pipeline"), 
        layout: Some(layout), 
        vertex: wgpu::VertexState { 
            module: &shader, 
            entry_point: "vs_main", 
            buffers: vertex_layouts 
        }, 
        primitive: wgpu::PrimitiveState { 
            topology: wgpu::PrimitiveTopology::TriangleList, 
            strip_index_format: None, 
            front_face: wgpu::FrontFace::Ccw, 
            cull_mode: Some(wgpu::Face::Back), 
            unclipped_depth: false, 
            polygon_mode: wgpu::PolygonMode::Fill, 
            conservative: false 
        }, 
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState { 
            format, 
            depth_write_enabled: true, 
            depth_compare: wgpu::CompareFunction::Less, 
            stencil: wgpu::StencilState::default(), 
            bias: wgpu::DepthBiasState::default() 
        }), 
        multisample: wgpu::MultisampleState { 
            count: 1, 
            mask: !0, 
            alpha_to_coverage_enabled: false 
        }, 
        fragment: Some(wgpu::FragmentState {
            module: &shader, 
            entry_point: "fs_main", 
            targets: &[Some(wgpu::ColorTargetState { 
                format: color_format, 
                blend: Some(wgpu::BlendState { 
                    color: wgpu::BlendComponent::REPLACE, 
                    alpha: wgpu::BlendComponent::REPLACE 
                }), 
                write_mask: wgpu::ColorWrites::ALL 
            })] 
        }),
        multiview: None,
    })
}