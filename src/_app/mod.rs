use std::{sync::Arc, time::Instant};

use wgpu::{
    CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceId, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{UserAttentionType, Window, WindowBuilder, WindowId},
};

use crate::{_engine::Engine, Model, APP_NAME};

pub mod app_config;
pub use app_config::*;

pub mod app_input_handler;
pub use app_input_handler::*;

pub mod app_object;
pub use app_object::*;

pub mod app_objects;
pub use app_objects::*;

use self::keyboard_input_handler::AppKeyboardInputHandler;

pub struct App {
    app_config: AppConfig,
    app_input_handler: AppInputHandler,
    app_window: Option<Arc<Window>>,

    last_time: Instant,
    delta_time: f64,
    ups: u64,

    objects: Vec<Box<dyn AppObject>>,
}

impl App {
    pub fn from_app_config_default_path() -> Self {
        let default_config_path = AppConfig::request_default_path();
        let app_config = AppConfig::read_or_write_default(&default_config_path);

        App::from_app_config(app_config)
    }

    pub fn from_app_config_path(app_config_path: &str) -> Self {
        let app_config = AppConfig::read_or_write_default(app_config_path);

        Self::from_app_config(app_config)
    }

    pub fn from_app_config(app_config: AppConfig) -> Self {
        Self {
            app_config,
            app_input_handler: AppInputHandler::new(),
            app_window: None,
            last_time: Instant::now(),
            delta_time: 0.0,
            ups: 0,
            objects: Vec::new(),
        }
    }

    pub async fn hijack_thread_and_run(mut self) {
        // Event Loop & Window creation
        let event_loop = EventLoop::new();
        let window_arc = Arc::new(self.build_window(&event_loop));
        self.app_window = Some(window_arc.clone());

        // Engine creation
        let mut engine: Engine = Engine::initialize(window_arc.clone()).await;
        engine.configure();

        // Spawn world
        // TODO: Must be move to the outside into Main!
        // let cube = Cube::new(
        //     &engine.get_device(),
        //     &engine.get_queue(),
        //     engine.get_default_texture_layout(),
        // )
        // .expect("failed to make cube ...");
        // let cube_boxed = Box::new(cube);
        // self.spawn(cube_boxed);
        let round_cube = RoundCube::new(
            &engine.get_device(),
            &engine.get_queue(),
            engine.get_default_texture_layout(),
        )
        .expect("failed to make cube ...");
        let round_cube_boxed = Box::new(round_cube);
        self.spawn(round_cube_boxed);

        // Get the engine backend and capitalize it
        let engine_backend = engine.get_backend_name();
        log::info!("Engine Backend: {engine_backend}");

        // Add Engine camera to objects

        // << Cycle Calculation >>
        self.last_time = Instant::now();
        self.delta_time = 0.0;
        self.ups = 0;

        event_loop.run(move |event, _target, control_flow| {
            // Immediately start a new cycle once a loop is completed.
            // Ideal for games, but more resource intensive.
            *control_flow = ControlFlow::Poll;

            // << Events >>
            match event {
                Event::WindowEvent { window_id, event } => {
                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != window_arc.clone().id() {
                        log::warn!("Invalid window ID for 'Window Event :: Window ID: {window_id:?}, Event: {event:?}'");
                        return;
                    }

                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),
                        WindowEvent::Resized(new_size) => self.handle_resize(new_size, &mut engine),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.handle_resize(*new_inner_size, &mut engine),
                        WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic
                        } => self.handle_keyboard_input(device_id, input, is_synthetic),
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => self.handle_redraw(&mut engine, window_id),
                Event::RedrawEventsCleared => self.handle_redraw_events_cleared(),
                Event::MainEventsCleared => self.handle_main_events_cleared(&engine_backend, control_flow, &mut engine),
                _ => (),
            }
        });
    }

    fn handle_redraw_events_cleared(&mut self) {
        // Request to redraw the next cycle
        self.app_window.clone().unwrap().request_redraw();
    }

    /// Handles the main events cleared event.
    /// This event is triggered after **all** other events have been processed.
    /// This event **should** be relatively consistent in comparison.
    ///
    /// This function is used to update our app, including:
    /// - Updateables
    /// - UPS & Delta calculation
    /// - Performance outputs
    fn handle_main_events_cleared(
        &mut self,
        backend_name: &str,
        control_flow: &mut ControlFlow,
        engine: &mut Engine,
    ) {
        // Take now time
        let now = Instant::now();
        // Get the duration of elapsed time since last update
        let elapsed = self.last_time.elapsed();
        // Update last time with now time
        self.last_time = now;

        // Add the elapsed time to the delta time
        self.delta_time += elapsed.as_secs_f64();
        // Increment UPS cycle count
        self.ups += 1;

        // Call updateables
        self.call_on_dynamic_update(self.delta_time, engine);

        // TODO: Move out ...
        engine.update();

        // If a second has past, call updates
        if self.delta_time >= 1.0 {
            #[cfg(debug_assertions)]
            {
                // Update performance outputs
                log::debug!("UPS: {}/s (delta time: {}s)", self.ups, self.delta_time);

                // Update Window Title
                self.app_window.clone().unwrap().set_title(&format!(
                    "WGPU @ {} - UPS: {}/s (Δ {}s)",
                    backend_name, self.ups, self.delta_time
                ));
            }

            // Check for main keyboard inputs
            if self
                .app_input_handler
                .are_all_keys_pressed(&vec![VirtualKeyCode::LAlt, VirtualKeyCode::F4])
                || self
                    .app_input_handler
                    .is_key_pressed(&VirtualKeyCode::Escape)
            {
                log::warn!("Exit condition reached!");
                *control_flow = ControlFlow::Exit;
            }

            // Call updateables
            self.call_on_second_update(self.delta_time);

            // Reset counters
            self.ups = 0;
            self.delta_time -= 1.0;
        }
    }

    fn handle_keyboard_input(
        &mut self,
        _device_id: DeviceId,
        input: KeyboardInput,
        _is_synthetic: bool,
    ) {
        let input_handler: &mut AppInputHandler = &mut self.app_input_handler;
        let keyboard_handler: &mut AppKeyboardInputHandler =
            input_handler.get_keyboard_input_handler();

        keyboard_handler.handle_keyboard_input(input);
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>, engine: &mut Engine) {
        log::info!(
            "Resize detected! Changing from {}x{} to {}x{}",
            self.app_config.window_config.size.0,
            self.app_config.window_config.size.1,
            &new_size.width,
            &new_size.height
        );

        if new_size.width <= 0 || new_size.height <= 0 {
            log::error!("Invalid new window size received!");
            return;
        }

        if !engine.get_device().poll(wgpu::MaintainBase::Wait) {
            log::error!("Failed to poll device before resizing!");
            return;
        }

        // Update config
        self.app_config.window_config.size = new_size.into();
        if self.app_config.monitor_config.is_some() {
            self.app_config.monitor_config.as_mut().unwrap().size = new_size.into();
        }
        self.app_config
            .write_to_path(&AppConfig::request_default_path());

        // Reconfigure the surface
        engine.configure();
    }

    fn handle_redraw(&mut self, engine: &mut Engine, window_id: WindowId) {
        // Validate that the window ID match.
        // Should only be different if multiple windows are used.
        if window_id != self.app_window.clone().unwrap().id() {
            log::warn!("A window with an ID not matching our window wants to be redrawn by us ... Skipping?");
            return;
        }

        let output_surface_texture = engine
            .get_surface()
            .get_current_texture()
            .expect("failed acquiring current texture of target window");

        let output_surface_texture_view = output_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Retrieve models
        let models: Vec<&Model> = self
            .objects
            .iter_mut()
            .filter(|x| x.do_render())
            .map(|x| x.model())
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        // Make command encoder
        let mut command_encoder =
            engine
                .get_device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        // let cube = Cube::new(
        //     &engine.get_device(),
        //     &engine.get_queue(),
        //     engine.get_default_texture_layout(),
        // )
        // .expect("cube failed");
        // let cube_model = cube.model().expect("model failed");
        {
            // Start RenderPass
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_surface_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            // Sky blue - ish
                            r: 0.0,
                            g: 0.61176,
                            b: 0.77647,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &engine.get_depth_texture().get_view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(engine.get_render_pipeline());

            render_pass.set_bind_group(1, &engine.get_camera().get_bind_group(), &[]);
            render_pass.set_bind_group(2, &engine.get_ambient_light_bind_group(), &[]);
            render_pass.set_bind_group(3, &engine.get_point_light_bind_group(), &[]);

            // render_pass.set_vertex_buffer(
            //     0,
            //     cube_model.meshes.first().unwrap().vertex_buffer.slice(..),
            // );
            // render_pass.set_index_buffer(
            //     cube_model.meshes.first().unwrap().index_buffer.slice(..),
            //     IndexFormat::Uint32,
            // );
            // render_pass.draw_indexed(0..cube_model.meshes.first().unwrap().num_elements, 0, 0..1);

            models.iter().for_each(|x| {
                x.meshes.iter().for_each(|y| {
                    render_pass.set_vertex_buffer(0, y.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(y.index_buffer.slice(..), IndexFormat::Uint32);

                    // Instances
                    render_pass.set_vertex_buffer(1, engine.get_instance_buffer().slice(..));

                    // Texture
                    let material = &x.materials[y.material];
                    render_pass.set_bind_group(0, &material.bind_group, &[]);

                    // render_pass.draw_indexed(0..y.num_elements, 0, y.instance_range.clone());
                    render_pass.draw_indexed(0..y.num_elements, 0, engine.get_instance_count());
                    // TODO: Instances must be moved into Model
                });
            });
        }

        let command_buffer = command_encoder.finish();

        // Submit command buffer
        engine.get_queue().submit(vec![command_buffer]);
        output_surface_texture.present();
    }

    fn build_window(&self, event_loop: &EventLoop<()>) -> Window {
        let fullscreen = match &self.app_config.monitor_config {
            Some(x) => Some(x.fullscreen.to_winit_fullscreen(&event_loop, &x)),
            None => None,
        };

        let size = self.app_config.window_config.to_physical_size();

        let mut builder = WindowBuilder::new();
        builder = builder.with_active(true);
        builder = builder.with_visible(true);
        builder = builder.with_title(APP_NAME);
        builder = builder.with_inner_size(size);
        builder = builder.with_maximized(false);
        builder = builder.with_resizable(true);

        if fullscreen.is_some() {
            builder = builder.with_fullscreen(fullscreen);
        }

        match builder.build(&event_loop) {
            Ok(window) => {
                window.request_user_attention(Some(UserAttentionType::Informational));
                window.focus_window();
                window
            }
            Err(err) => panic!("Window building failed! {:#?}", err),
        }
    }

    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }

    pub fn spawn(&mut self, object: Box<dyn AppObject>) {
        self.objects.push(object);
    }

    /// Calls all registered updateables if their [`UpdateFrequency`]
    /// is set to [`UpdateFrequency::OnSecond`]
    pub fn call_on_second_update(&mut self, delta_time: f64) {
        // Call the main update function for each object
        self.objects
            .iter_mut()
            .filter(|x| x.do_second_update())
            .for_each(|x| x.on_second_update(delta_time));
    }

    /// Calls all registered updateables if their [`UpdateFrequency`]
    /// is set to [`UpdateFrequency::OnCycle`]
    pub fn call_on_dynamic_update(&mut self, delta_time: f64, engine: &mut Engine) {
        // Call the main update function for each object
        self.objects
            .iter_mut()
            .filter(|x| x.do_dynamic_update())
            .for_each(|x| x.on_dynamic_update(delta_time));

        // -- Call other update functions:

        // Special case: Camera
        let camera = engine.get_camera_mut();
        if camera.do_dynamic_update() {
            camera.on_dynamic_update(delta_time);
        }
        if camera.do_input() {
            camera.on_input(delta_time, &self.app_input_handler);
        }

        // Call input handling
        self.objects
            .iter_mut()
            .filter(|x| x.do_input())
            .for_each(|x| x.on_input(delta_time, &self.app_input_handler));
    }
}
