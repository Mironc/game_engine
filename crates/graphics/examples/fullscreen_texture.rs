use std::sync::Arc;
use std::time::Instant;

use ash::vk::{ImageLayout, PipelineStageFlags};
use graphics::context::GraphicsContext;
use graphics::device::DeviceContext;
use graphics::render_graph::execution::{EasyExecutor, Executor};
use graphics::render_graph::operations::draw_call::{DrawCall, DrawGeometry, DrawParameters};
use graphics::render_graph::operations::gpu_operation::{Operation, UploadImageOp, WriteBufferOp};
use graphics::render_graph::render_graph::RenderGraph;
use graphics::rendering::descriptor_container::DescriptorId;
use graphics::rendering::framebuffer_container::FramebufferCreate;
use graphics::rendering::pipeline_container::{CreatePipeline, PipelineId};
use graphics::rendering::render_pass_container::{
    LoadOption, RenderPassAttachment, RenderPassDescription, StoreOption, SubPass,
};
use graphics::rendering::renderer_bundle::RendererBundle;
use graphics::rendering::shader_container::ShaderType;
use graphics::rendering::texture_container::{
    CreateTexture, CreateTextureView, SamplingOptions, TextureFormat, TextureViewId,
};
use graphics::swapchain::SwapChain;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    swapchain: Option<SwapChain>,
    context: Option<Arc<GraphicsContext>>,
    device_context: Option<Arc<DeviceContext>>,
    bundle: Option<RendererBundle>,
    pipeline_id: Option<PipelineId>,
    render_graph: Option<RenderGraph>,
    descriptor_id: Option<[DescriptorId; 2]>,
    texture_view: Option<TextureViewId>,
}
impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        log::warn!("Recreating window");
        let attributes = WindowAttributes::default().with_title("Tri");
        let window = event_loop
            .create_window(attributes)
            .expect("Window creation went wrong");

        let graphics_context =
            GraphicsContext::init(&window).expect("Couldn't create graphic config");

        let binding = graphics_context
            .instance()
            .list_devices()
            .expect("Couldn't get devices");
        let best_device = binding
            .iter()
            .max_by(|x, x1| x.rate_default().cmp(&x1.rate_default()))
            .expect("No gpu is available");
        let device_context = DeviceContext::new(&graphics_context, best_device)
            .expect("Couldn't init device context");

        let shared_graphics_context = Arc::new(graphics_context);
        let shared_device_context = Arc::new(device_context);

        let swapchain = SwapChain::new(&shared_graphics_context, &shared_device_context, &window)
            .expect("couldn't create swapchain");
        let mut bundle = RendererBundle::new();

        let render_pass_desc = RenderPassDescription {
            attachments: vec![
                RenderPassAttachment::new()
                    .format(TextureFormat::B8G8R8A8)
                    .initial_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .final_layout(ImageLayout::PRESENT_SRC_KHR)
                    .load_op(LoadOption::Clear)
                    .store_op(StoreOption::Store),
            ],
            subpass: SubPass::new(Vec::new(), vec![0], Vec::new()),
        };
        let _ = bundle
            .render_pass_container
            .create_renderpass(&shared_device_context, render_pass_desc.clone());
        let render_pass = bundle
            .render_pass_container
            .get_render_pass(&render_pass_desc)
            .cloned()
            .unwrap();
        let vertex_shader_id = bundle
            .shader_container
            .insert(
                "
                #version 450

                layout(location = 0) out vec2 uv;

                void main() {
                    uv = vec2((gl_VertexIndex << 1u) & 2u, gl_VertexIndex & 2u);
                    gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
                }",
                ShaderType::Vertex,
            )
            .unwrap();
        let fragment_shader_id = bundle
            .shader_container
            .insert(
                "#version 450

            layout(location = 0) in vec2 uv;
            layout(location = 0) out vec4 outColor;

            layout(set = 0, binding = 0) uniform sampler tex_s;
            layout(set = 0, binding = 1) uniform texture2D tex;

            float lum(vec3 color){
                return dot(color, vec3(0.3,0.59,0.11));
            }

            void main() {
                vec4 fetched_color = texture(sampler2D(tex, tex_s), uv).rgba;
                outColor = vec4(fetched_color.rgb*fetched_color.a,1.0);
            }",
                ShaderType::Fragment,
            )
            .unwrap();

        let pipeline_id = bundle
            .pipeline_container
            .create_pipeline(
                &shared_device_context,
                &bundle.shader_container,
                CreatePipeline::<()>::new()
                    .shaders(&[vertex_shader_id, fragment_shader_id])
                    .render_pass(&render_pass),
            )
            .unwrap();
        let pipeline = bundle
            .pipeline_container
            .get(pipeline_id)
            .unwrap()
            .pipeline_layout()
            .shader_layout();

        let texture = bundle
            .texture_container
            .create_texture(
                &shared_device_context,
                CreateTexture::new()
                    .image_format(TextureFormat::R8G8B8A8)
                    .dimensions(3000, 2000, 1),
            )
            .unwrap();
        let texture_view = bundle
            .texture_container
            .create_texture_view(
                &shared_device_context,
                CreateTextureView::new()
                    .texture_id(texture)
                    .format(TextureFormat::R8G8B8A8),
            )
            .unwrap();

        let mut descriptor_group = bundle
            .descriptor_container
            .create_descriptor_set(&shared_device_context, pipeline.clone())
            .unwrap();
        descriptor_group.set_sampler(
            "tex_s",
            SamplingOptions::new()
                .filter(graphics::rendering::texture_container::Filter::Linear)
                .wrap(graphics::rendering::texture_container::WrapOption::Repeat),
        );
        descriptor_group.set_texture("tex", texture_view);
        bundle.descriptor_container.apply_changes(
            &shared_device_context,
            &descriptor_group,
            &bundle.buffer_container,
            &mut bundle.texture_container,
        );

        let mut descriptor_group_2 = bundle
            .descriptor_container
            .create_descriptor_set(&shared_device_context, pipeline.clone())
            .unwrap();
        descriptor_group_2.set_sampler(
            "tex_s",
            SamplingOptions::new()
                .filter(graphics::rendering::texture_container::Filter::Point)
                .wrap(graphics::rendering::texture_container::WrapOption::Repeat),
        );
        descriptor_group_2.set_texture("tex", texture_view);

        bundle.descriptor_container.apply_changes(
            &shared_device_context,
            &descriptor_group_2,
            &bundle.buffer_container,
            &mut bundle.texture_container,
        );

        let mut render_graph = RenderGraph::new();
        let image = image::load_from_memory(include_bytes!("Vulkan-logo.png"))
            .unwrap()
            .flipv()
            .to_rgba8();
        render_graph.add_operation(Operation::UploadImage(UploadImageOp::new(image, texture)));

        self.window = Some(window);
        self.context = Some(shared_graphics_context);
        self.device_context = Some(shared_device_context);
        self.swapchain = Some(swapchain);
        self.bundle = Some(bundle);
        self.render_graph = Some(render_graph);
        self.pipeline_id = Some(pipeline_id);
        self.descriptor_id = Some([descriptor_group, descriptor_group_2]);
        self.texture_view = Some(texture_view);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_new_size) => {
                self.resize();
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let (
                    Some(context),
                    Some(swapchain),
                    Some(bundle),
                    Some(pipeline_id),
                    Some(render_graph),
                    Some(descs),
                ) = (
                    &self.device_context,
                    &mut self.swapchain,
                    &mut self.bundle,
                    self.pipeline_id,
                    &mut self.render_graph,
                    &mut self.descriptor_id,
                ) {
                    let device = context;
                    let frame_data = swapchain.next_frame(device);
                    let frame_sync = frame_data.sync();
                    let graphics_queue = context.render_queue().graphics_queue();
                    device
                        .render_queue()
                        .graphics_queue()
                        .get_commandpool(device, &frame_data)
                        .value_mut()
                        .reset(device);
                    let pipeline = bundle.pipeline_container.get(pipeline_id).unwrap();
                    let (texture, view) = bundle.texture_container.insert_framedata(&frame_data);

                    let desc = &descs[frame_data.fif_id()];
                    let framebuffer_id = bundle
                        .framebuffer_container
                        .insert_framebuffer(
                            device,
                            &bundle.texture_container,
                            FramebufferCreate::new([view].to_vec(), pipeline.render_pass()),
                        )
                        .unwrap();

                    render_graph.add_target_op(Operation::DrawCall(DrawCall::Direct {
                        draw_param: DrawParameters::new(
                            DrawGeometry::Procedural { count: 3 },
                            framebuffer_id,
                            pipeline_id,
                            None,
                            Some(desc.clone()),
                        ),
                    }));
                    let executor = EasyExecutor {
                        actions: render_graph.compile(bundle).unwrap(),
                    };
                    let command_buffer = executor.execute(device, bundle, &frame_data);

                    render_graph.clear();
                    let wait_semaphores = [frame_sync.image_available()];
                    let signal_semaphores = [frame_data.image().image_sync().render_finished()];
                    let wait_stages = [ash::vk::PipelineStageFlags::ALL_COMMANDS];
                    let command_buffers = [command_buffer];
                    let submit_info = [ash::vk::SubmitInfo::default()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .command_buffers(&command_buffers)
                        .signal_semaphores(&signal_semaphores)];
                    unsafe {
                        context
                            .queue_submit(
                                graphics_queue.handle(),
                                &submit_info,
                                frame_data.sync().frame_done(),
                            )
                            .expect("Error while submiting");
                    }
                    let present_queue = context.render_queue().present_queue();
                    swapchain
                        .present_frame(present_queue, frame_data)
                        .expect("Couldn't present image");
                }
            }
            _ => (),
        }
        self.window.as_ref().map(|x| x.request_redraw());
    }
}
impl App {
    pub fn resize(&mut self) {
        if let (Some(graphics_context), Some(device_context), Some(window), Some(bundle)) = (
            &self.context,
            &self.device_context,
            &self.window,
            &mut self.bundle,
        ) {
            unsafe {
                device_context
                    .device_wait_idle()
                    .expect("Error waiting device idle");
            }
            self.swapchain = if let Some(swapchain) = &mut self.swapchain {
                swapchain.frames().iter().for_each(|f| {
                    bundle.remove_frameimage(device_context, f);
                });
                log::debug!("Swapchain recreated!");
                Some(
                    swapchain
                        .recreate(graphics_context, device_context, window)
                        .expect("Error while recreating swapchain"),
                )
            } else {
                log::debug!("New swapchain!");
                Some(
                    SwapChain::new(graphics_context, device_context, window)
                        .expect("Error while recreating swapchain"),
                )
            };
        }
    }
}
fn main() {
    simple_logger::init().expect("Couldn't initialize logger");
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
