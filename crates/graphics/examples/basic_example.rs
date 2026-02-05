use std::sync::Arc;

use ash::vk::{
    AccessFlags, ClearColorValue, CommandBufferBeginInfo, DependencyFlags, Fence, ImageAspectFlags,
    ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags,
};
use graphics::device::DeviceContext;
use graphics::swapchain::SwapChain;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

use graphics::context::GraphicsContext;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    swapchain: Option<SwapChain>,
    context: Option<Arc<GraphicsContext>>,
    device_context: Option<Arc<DeviceContext>>,
}
impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        println!("Creating window and graphics config");
        let attributes = WindowAttributes::default().with_title("1");
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

        println!("STATE ADDR RESUMED: {:p}", self);
        println!(
            "QUEUE ADDR RESUMED: {:?}",
            shared_device_context.render_queue().graphics_queue()
        );
        self.window = Some(window);
        self.context = Some(shared_graphics_context);
        self.device_context = Some(shared_device_context);
        self.swapchain = Some(swapchain);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.resize();
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let (Some(context), Some(swapchain)) =
                    (&self.device_context, &mut self.swapchain)
                {
                    let frame_data = swapchain.next_frame();
                    let frame_sync = frame_data.sync();
                    frame_sync.wait_until_frame_done(context);
                    frame_sync.reset_frame_done_fence(context);
                    let graphics_queue = context.render_queue().graphics_queue();
                    log::info!("drawing with FIF №{}", frame_data.fif_id());
                    {
                        let mut command_pool = graphics_queue.get_commandpool(context, &frame_data);
                        command_pool.reset(context);

                        let clear_color_values = [
                            ClearColorValue {
                                float32: [1.0, 1.0, 1.0, 1.0],
                            },
                            ClearColorValue {
                                float32: [0.0, 0.0, 1.0, 1.0],
                            },
                            ClearColorValue {
                                float32: [1.0, 1.0, 1.0, 1.0],
                            },
                        ];

                        let command_buffer = command_pool.get_buffer(context);
                        let command_buffer_begin_info = CommandBufferBeginInfo::default();
                        let device = context;
                        let screen_range = [ImageSubresourceRange::default()
                            .aspect_mask(ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1)];
                        unsafe {
                            device
                                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                                .expect("ERR")
                        };
                        let clear_color_value = clear_color_values[frame_data.fif_id()];
                        let swapchain_image = frame_data.image();
                        let image_barrier = [ImageMemoryBarrier::default()
                            .image(*swapchain_image)
                            .old_layout(ImageLayout::UNDEFINED)
                            .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                            .src_access_mask(AccessFlags::empty())
                            .dst_access_mask(AccessFlags::TRANSFER_WRITE)
                            .subresource_range(screen_range[0])];

                        unsafe {
                            device.cmd_pipeline_barrier(
                                command_buffer,
                                PipelineStageFlags::empty(),
                                PipelineStageFlags::TRANSFER,
                                DependencyFlags::empty(),
                                &[],
                                &[],
                                &image_barrier,
                            );

                            device.cmd_clear_color_image(
                                command_buffer,
                                *swapchain_image,
                                ImageLayout::TRANSFER_DST_OPTIMAL,
                                &clear_color_value,
                                &screen_range,
                            );
                            let image_barrier = [image_barrier[0]
                                .old_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
                                .new_layout(ImageLayout::PRESENT_SRC_KHR)
                                .src_access_mask(AccessFlags::TRANSFER_WRITE)
                                .dst_access_mask(AccessFlags::empty())];
                            device.cmd_pipeline_barrier(
                                command_buffer,
                                PipelineStageFlags::TRANSFER,
                                PipelineStageFlags::TOP_OF_PIPE,
                                DependencyFlags::empty(),
                                &[],
                                &[],
                                &image_barrier,
                            );
                        };
                        unsafe { device.end_command_buffer(command_buffer).expect("ERR") };

                        let wait_semaphores = [frame_sync.image_available()];
                        let signal_semaphores = [frame_sync.render_finished()];
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
        if let (Some(graphics_context), Some(device_context), Some(window)) =
            (&self.context, &self.device_context, &self.window)
        {
            self.swapchain = if let Some(swapchain) = &mut self.swapchain {
                println!("swapchain recreated");
                Some(
                    swapchain
                        .recreate(graphics_context, device_context, window)
                        .expect("Error while recreating swapchain"),
                )
            } else {
                println!("new swapchain");
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
