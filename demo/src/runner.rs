// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    fs::File,
    io::Write,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use forma::{cpu, prelude::*};
use winit::{
    dpi::PhysicalSize,
    event::VirtualKeyCode,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{App, Keyboard, Runner};

fn statistics(durations: &mut Vec<f64>) -> (f64, f64, f64) {
    let min = durations
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
        .unwrap();
    let max = durations
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .copied()
        .unwrap();
    let count = durations.len() as f64;
    (durations.drain(..).sum::<f64>() / count, min, max)
}

fn measure<F: FnOnce()>(f: F) -> Duration {
    let start = Instant::now();

    f();

    start.elapsed()
}

use softbuffer::GraphicsContext;

//#[derive(Debug)]
pub struct CpuRunner {
    composition: Composition,
    renderer: cpu::Renderer,
    buffer: Vec<u8>,
    layer_cache: BufferLayerCache,
    window: Window,
    layout: LinearLayout,
    graphics_context: GraphicsContext,
    /*device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,*/
    compose_durations: Vec<f64>,
    render_durations: Vec<f64>,
}

impl CpuRunner {
    pub fn new(event_loop: &EventLoop<()>, width: u32, height: u32) -> Self {
        let composition = Composition::new();
        let mut renderer = cpu::Renderer::new();
        let layer_cache = renderer.create_buffer_layer_cache().unwrap();

        let window = WindowBuilder::new()
            .with_title("demo | compose: ???ms, render: ???ms")
            .with_inner_size(PhysicalSize::new(width, height))
            .build(event_loop)
            .unwrap();

        let layout = LinearLayout::new(width as usize, width as usize * 4, height as usize);

        let graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap(); //?

        /*let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }))
        .expect("failed to find an appropriate adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to get device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };

        surface.configure(&device, &config);*/

        Self {
            composition,
            renderer,
            layer_cache,
            window,
            buffer: vec![0; (width * 4 * height) as usize],
            layout,
            graphics_context,
            /*device,
            queue,
            surface,
            config,*/
            compose_durations: Vec::new(),
            render_durations: Vec::new(),
        }
    }
}

impl Runner for CpuRunner {
    fn resize(&mut self, width: u32, height: u32) {
        self.buffer.resize((width * 4 * height) as usize, 0);
        self.layout = LinearLayout::new(width as usize, width as usize * 4, height as usize);

        /*self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);*/
    }

    fn render(&mut self, app: &mut dyn App, elapsed: Duration, keyboard: &Keyboard) {
        if self.compose_durations.len() == 50 {
            let (compose_avg, compose_min, compose_max) = statistics(&mut self.compose_durations);
            let (render_avg, render_min, render_max) = statistics(&mut self.render_durations);

            self.window.set_title(&format!(
                "demo | compose: {:.2}ms ({:.2}/{:.2}), render: {:.2}ms ({:.2}/{:.2})",
                compose_avg, compose_min, compose_max, render_avg, render_min, render_max,
            ));
        }

        let compose_duration = measure(|| {
            app.compose(&mut self.composition, elapsed, keyboard);
        });

        let render_duration = measure(|| {
            self.renderer.render(
                &mut self.composition,
                &mut BufferBuilder::new(&mut self.buffer, &mut self.layout)
                    .layer_cache(self.layer_cache.clone())
                    .build(),
                BGR1,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.0,
                },
                None,
            );
        });

        self.compose_durations
            .push(compose_duration.as_secs_f64() * 1000.0);
        self.render_durations
            .push(render_duration.as_secs_f64() * 1000.0);

        self.graphics_context.set_buffer(&bytemuck::cast_slice(&self.buffer), self.layout.width() as u16, self.layout.height() as u16);

        /*let frame = self.surface.get_current_texture().unwrap();

        self.queue.write_texture(
            frame.texture.as_image_copy(),
            &self.buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.config.width * 4),
                rows_per_image: NonZeroU32::new(self.config.height),
            },
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(None);

        frame.present();*/

        /*if keyboard.is_key_down(VirtualKeyCode::S) {
            let mut bytes = Vec::with_capacity(self.layout.width() * self.layout.height() * 3);
            for pixel in self.buffer.chunks(4) {
                if let &[b, g, r, _] = pixel {
                    bytes.push(r);
                    bytes.push(g);
                    bytes.push(b);
                }
            }
            let new_path = "capture.ppm";
            let mut output = File::options()
                .write(true)
                .create(true)
                .open(new_path)
                .unwrap();
            output
                .write_all(
                    format!(
                        "P6\n{} {}\n255\n",
                        self.layout.width(),
                        self.layout.height()
                    )
                    .as_bytes(),
                )
                .unwrap();
            output.write_all(&bytes).unwrap();
        }*/
    }
}