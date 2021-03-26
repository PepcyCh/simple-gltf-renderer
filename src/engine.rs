use anyhow::*;

use crate::camera::Camera;
use crate::graphics::GraphicsState;
use crate::light::Light;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::shader::Shader;
use std::collections::HashMap;
use std::convert::TryInto;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::ControlFlow;

pub struct Engine {
    window: winit::window::Window,
    window_size: PhysicalSize<u32>,
    last_mouse_position: PhysicalPosition<f64>,
    pub graphics_state: GraphicsState,
    pub meshes: Vec<Mesh>,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub shaders: HashMap<String, Shader>,
    pub materials: HashMap<String, Material>,
}

impl Engine {
    pub fn new() -> Result<(Self, winit::event_loop::EventLoop<()>)> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Simple glTF Renderer")
            .build(&event_loop)
            .unwrap();
        let graphics_state = futures::executor::block_on(GraphicsState::new(&window))?;

        let window_size = window.inner_size();
        let mut camera = Camera::new(
            (0.0, 5.0, 5.0).into(),
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            45.0,
            window_size.width as f32 / window_size.height as f32,
            0.1,
            1000.0,
        );
        camera.build(
            &graphics_state.device,
            &graphics_state.camera_bind_group_layout,
        );

        let mut light0 = Light::directional_light((-1.0, -8.0, -1.0).into(), [1.0, 1.0, 1.0, 1.0]);
        light0.build(
            &graphics_state.device,
            &graphics_state.light_bind_group_layout,
        );
        let mut light1 = Light::directional_light((1.0, -4.0, 1.0).into(), [0.8, 0.8, 0.8, 1.0]);
        light1.build(
            &graphics_state.device,
            &graphics_state.light_bind_group_layout,
        );

        Ok((
            Self {
                window,
                window_size,
                last_mouse_position: PhysicalPosition { x: 0.0, y: 0.0 },
                graphics_state,
                meshes: vec![],
                camera,
                lights: vec![light0, light1],
                shaders: HashMap::new(),
                materials: HashMap::new(),
            },
            event_loop,
        ))
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => {
                if !self.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        WindowEvent::Resized(new_size) => {
                            self.resize(*new_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            self.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => self.resize(self.window_size),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("Unhandled error {:?}", e),
                }
            }
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => {}
        });
    }

    pub fn load_shaders<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let json_file = std::fs::File::open(path)?;
        let json_reader = std::io::BufReader::new(json_file);
        let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

        let shaders = json_value["shaders"].as_array().unwrap();
        for shader in shaders {
            let mut shader: Shader = shader.try_into()?;
            shader.build(&self.graphics_state.device)?;
            for (sub_shader_tag, sub_shader) in &shader.sub_shaders {
                let render_pipeline = sub_shader.render_pipeline(
                    &shader,
                    &self.graphics_state.device,
                    self.graphics_state.swap_chain_desc.format,
                    GraphicsState::DEPTH_STENCIL_FORMAT,
                    &self.graphics_state.object_bind_group_layout,
                    &self.graphics_state.light_bind_group_layout,
                    &self.graphics_state.camera_bind_group_layout,
                    &self.graphics_state.scene_bind_group_layout,
                );
                self.graphics_state.render_pipelines.insert(
                    format!("{}-{}", &shader.name, sub_shader_tag),
                    render_pipeline,
                );
            }
            self.shaders.insert(shader.name.clone(), shader);
        }

        let materials = json_value["materials"].as_array().unwrap();
        for material in materials {
            let name = material["name"].as_str().unwrap().to_string();
            let shader = material["shader"].as_str().unwrap().to_string();
            let material = Material::from_shader(
                name.clone(),
                &self.shaders[&shader],
                &self.graphics_state.device,
                &self.graphics_state.queue,
            );
            self.materials.insert(name, material);
        }

        Ok(())
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        // TODO - move to a single file ?
        let mut result = false;
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    } => {
                        // TODO - key press
                        result = true;
                        let delta = 0.05;
                        match keycode {
                            VirtualKeyCode::W => self
                                .camera
                                .translate(cgmath::Vector3::new(0.0, 0.0, -delta)),
                            VirtualKeyCode::S => {
                                self.camera.translate(cgmath::Vector3::new(0.0, 0.0, delta))
                            }
                            VirtualKeyCode::A => self
                                .camera
                                .translate(cgmath::Vector3::new(-delta, 0.0, 0.0)),
                            VirtualKeyCode::D => {
                                self.camera.translate(cgmath::Vector3::new(delta, 0.0, 0.0))
                            }
                            VirtualKeyCode::Q => {
                                self.camera.translate(cgmath::Vector3::new(0.0, delta, 0.0))
                            }
                            VirtualKeyCode::E => self
                                .camera
                                .translate(cgmath::Vector3::new(0.0, -delta, 0.0)),
                            _ => result = false,
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // TODO - cursor move
                let delta_x = (position.x - self.last_mouse_position.x) as f32;
                let delta_y = (position.y - self.last_mouse_position.y) as f32;
                self.last_mouse_position = *position;
                self.camera.rotate(delta_x, delta_y);
                result = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                // TODO - mouse press
            }
            WindowEvent::MouseWheel {
                phase: winit::event::TouchPhase::Moved,
                delta,
                ..
            } => {
                // TODO - wheel move
                let (_delta_x, delta_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.camera.move_forward(delta_y);
                result = true;
            }
            _ => {}
        }
        result
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.graphics_state.resize(new_size.width, new_size.height);
        self.camera
            .set_aspect(new_size.width as f32 / new_size.height as f32);
        self.window_size = new_size;
    }

    fn update(&mut self) {
        // TODO
        self.camera.update(&self.graphics_state.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.graphics_state.swap_chain.get_current_frame()?.output;
        let mut encoder =
            self.graphics_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.graphics_state.depth_stencil_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });
            // println!("Begin Frame");
            render_pass.set_bind_group(3, &self.camera.bind_group.as_ref().unwrap(), &[]);
            let mut is_first = true;
            for light in &self.lights {
                let sub_shader_tag = if is_first {
                    "ForwardBase"
                } else {
                    "ForwardAdd"
                };
                is_first = false;
                // println!("- SubShader tag = {}", &sub_shader_tag);
                render_pass.set_bind_group(2, light.bind_group.as_ref().unwrap(), &[]);

                for mesh in &self.meshes {
                    render_pass.set_bind_group(1, mesh.bind_group.as_ref().unwrap(), &[]);
                    render_pass
                        .set_vertex_buffer(0, mesh.vertex_buffer.as_ref().unwrap().slice(..));
                    render_pass.set_index_buffer(
                        mesh.index_buffer.as_ref().unwrap().slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    if let Some(material) = self.materials.get(&mesh.material) {
                        render_pass.set_bind_group(0, material.bind_group.as_ref().unwrap(), &[]);
                        let pipeline_name = format!("{}-{}", &material.shader, sub_shader_tag);
                        if let Some(pipeline) =
                            self.graphics_state.render_pipelines.get(&pipeline_name)
                        {
                            // println!("- Render pipeline: {}, material is {}", &pipeline_name, &material.name);
                            render_pass.set_pipeline(pipeline);
                            render_pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
                        }
                    }
                }
            }
            // println!("End Frame");
        }
        self.graphics_state
            .queue
            .submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
