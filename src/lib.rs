use std::ffi::CString;

use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event::{Event, WindowEvent};

use gpu_memory_manager::prelude::*;

pub fn run() {
    // Windowing
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut spyder = Spyder::new(window);

    event_loop.run(move |event, _, controlflow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *controlflow = winit::event_loop::ControlFlow::Exit;
        }
        Event::MainEventsCleared => {
            // doing the work here (later)
            spyder.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            spyder.swap_chain.current_image =
                (spyder.swap_chain.current_image + 1) % spyder.swap_chain.frames_in_flight;
            let (image_index, _) = unsafe {
                spyder
                    .swap_chain
                    .swap_chain_loader
                    .acquire_next_image(
                        spyder.swap_chain.swap_chain,
                        u64::MAX,
                        spyder.swap_chain.image_available[spyder.swap_chain.current_image as usize],
                        vk::Fence::null(),
                    )
                    .expect("image acquisition trouble")
            };

            unsafe {
                spyder
                    .logical_device
                    .wait_for_fences(
                        &[spyder.swap_chain.may_begin_drawing
                            [spyder.swap_chain.current_image as usize]],
                        true,
                        u64::MAX,
                    )
                    .expect("fence-waiting");
                spyder
                    .logical_device
                    .reset_fences(&[spyder.swap_chain.may_begin_drawing
                        [spyder.swap_chain.current_image as usize]])
                    .expect("resetting fences");
            }

            let semaphores_available =
                [spyder.swap_chain.image_available[spyder.swap_chain.current_image as usize]];
            let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let semaphores_finished =
                [spyder.swap_chain.rendering_finished[spyder.swap_chain.current_image as usize]];
            let command_buffers = [spyder.command_buffers[image_index as usize]];
            let submit_info = [vk::SubmitInfo::builder()
                .wait_semaphores(&semaphores_available)
                .wait_dst_stage_mask(&waiting_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&semaphores_finished)
                .build()];

            unsafe {
                spyder
                    .logical_device
                    .queue_submit(
                        spyder.queues.graphics_queue,
                        &submit_info,
                        spyder.swap_chain.may_begin_drawing
                            [spyder.swap_chain.current_image as usize],
                    )
                    .expect("queue submission");
            };

            let swap_chains = [spyder.swap_chain.swap_chain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&semaphores_finished)
                .swapchains(&swap_chains)
                .image_indices(&indices);
            unsafe {
                spyder
                    .swap_chain
                    .swap_chain_loader
                    .queue_present(spyder.queues.graphics_queue, &present_info)
                    .expect("queue presentation");
            };
        }
        _ => {}
    });
}

struct Spyder {
    window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<VulkanDebug>,
    surface: std::mem::ManuallyDrop<Surface>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    queues: Queues,
    logical_device: ash::Device,
    swap_chain: SwapChain,
    render_pass: vk::RenderPass,
    pipeline: Pipeline,
    pools: Pools,
    command_buffers: Vec<vk::CommandBuffer>,
    allocator: Allocator,
    buffers: Vec<Buffer>,
}

impl Spyder {
    fn new(window: winit::window::Window) -> Self {
        // Vulkan lib integration.
        let entry: ash::Entry = ash::Entry::linked();

        // Debug info
        let instance = init_instance(&entry, &window);

        let debug = VulkanDebug::new(&entry, &instance);

        // Surface
        let surface = Surface::new(&entry, &instance, &window);

        // Device
        let (physical_device, physical_device_properties) =
            init_physical_device_and_properties(&instance);
        dbg!(physical_device_properties);

        let queue_families = QueueFamilies::new(&instance, physical_device, &surface);

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families);

        let mut swap_chain = SwapChain::new(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
        );

        let render_pass = init_render_pass(&logical_device, physical_device, &surface);
        swap_chain.create_frame_buffers(&logical_device, render_pass);

        let pipeline = Pipeline::new(&logical_device, &swap_chain, &render_pass);

        let pools = Pools::new(&logical_device, &queue_families);

        let command_buffers = create_command_buffers(
            &logical_device,
            &pools,
            swap_chain.frames_in_flight as usize,
        );

        let allocator_create_info = AllocatorCreateInfo::builder()
            .instance(&instance)
            .device(&logical_device)
            .physical_device(physical_device)
            .build();

        let mut allocator = Allocator::new(&allocator_create_info);

        let positions = [
            0.5f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.2f32, 0.0f32, 1.0f32, -0.5f32, 0.0f32,
            0.0f32, 1.0f32, -0.9f32, -0.9f32, 0.0f32, 1.0f32, 0.3f32, -0.8f32, 0.0f32, 1.0f32,
            0.0f32, -0.6f32, 0.0f32, 1.0f32,
        ];
        let colours = [
            0.0f32, 1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 0.0f32, 1.0f32, 0.0f32,
            1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32, 0.8f32, 0.7f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32,
            1.0f32, 1.0f32,
        ];

        let mut position_buffer = Buffer::new(
            &mut allocator,
            positions.len() as u64 * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
            "position_buffer",
        );
        let mut colour_buffer = Buffer::new(
            &mut allocator,
            colours.len() as u64 * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
            "colour_buffer",
        );

        position_buffer.fill(&positions);
        colour_buffer.fill(&colours);

        let internal_buffers = vec![position_buffer.buffer, colour_buffer.buffer];

        let buffers = vec![position_buffer, colour_buffer];

        fill_command_buffers(
            &logical_device,
            &command_buffers,
            &swap_chain,
            render_pass,
            &pipeline,
            &internal_buffers,
        );

        Self {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surface: std::mem::ManuallyDrop::new(surface),
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            logical_device,
            swap_chain,
            render_pass,
            pipeline,
            pools,
            command_buffers,
            allocator,
            buffers,
        }
    }
}

impl Drop for Spyder {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("something wrong while waiting");

            for idx in (0..self.buffers.len()).rev() {
                let buffer = self.buffers.remove(idx);
                buffer.cleanup(&mut self.allocator)
            }

            self.allocator.cleanup();

            self.pools.cleanup(&self.logical_device);
            self.pipeline.cleanup(&self.logical_device);
            self.logical_device
                .destroy_render_pass(self.render_pass, None);
            self.swap_chain.cleanup(&self.logical_device);
            self.logical_device.destroy_device(None);

            std::mem::ManuallyDrop::drop(&mut self.surface);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None);
        };
    }
}

fn init_instance(entry: &ash::Entry, window: &winit::window::Window) -> ash::Instance {
    // Engine details
    let engine_name = CString::new("Spyder").unwrap();
    let app_name = CString::new("Example").unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::API_VERSION_1_3);

    // Layers, TODO: setup system to allow different plugins to request layers and extentions.
    let layer_names: Vec<CString> = vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()];

    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let mut extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

    extension_name_pointers.append(
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .expect("Failed to get required extensions")
            .to_vec()
            .as_mut(),
    );

    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    //  Finally initialize the instance
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);

    unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .expect("Instance creation error")
    }
}

fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> (vk::PhysicalDevice, vk::PhysicalDeviceProperties) {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to get devices")
    };

    for physical_device in &physical_devices {
        let props = unsafe { instance.get_physical_device_properties(*physical_device) };
        dbg!(props);
    }

    let (physical_device, physical_device_properties) = physical_devices
        .iter()
        .find_map(|physical_device| {
            let properties = unsafe { instance.get_physical_device_properties(*physical_device) };
            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                Some((*physical_device, properties))
            } else {
                None
            }
        })
        .expect("No discrete GPU found");
    (physical_device, physical_device_properties)
}

fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
) -> (ash::Device, Queues) {
    let queue_priorities = [1.0f32];
    let queue_infos: Vec<vk::DeviceQueueCreateInfo> = vec![
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.graphics_queue_index.unwrap())
            .queue_priorities(&queue_priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_families.transfer_queue_index.unwrap())
            .queue_priorities(&queue_priorities)
            .build(),
    ];

    let device_extension_name_pointers = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers);
    let logical_device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create device handle")
    };
    let graphics_queue =
        unsafe { logical_device.get_device_queue(queue_families.graphics_queue_index.unwrap(), 0) };
    let transfer_queue =
        unsafe { logical_device.get_device_queue(queue_families.transfer_queue_index.unwrap(), 0) };
    (
        logical_device,
        Queues {
            graphics_queue,
            transfer_queue,
        },
    )
}

fn init_render_pass(
    logical_device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    surface: &Surface,
) -> vk::RenderPass {
    let attachments = [vk::AttachmentDescription::builder()
        .format(surface.get_formats(physical_device).first().unwrap().format)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build()];

    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let sub_passes = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];

    let sub_pass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&sub_passes)
        .dependencies(&sub_pass_dependencies);
    unsafe {
        logical_device
            .create_render_pass(&render_pass_info, None)
            .expect("Failed to create render pass")
    }
}

fn create_command_buffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.command_pool_graphics)
        .command_buffer_count(amount as u32);

    unsafe {
        logical_device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate command buffers")
    }
}

fn fill_command_buffers(
    logical_device: &ash::Device,
    command_buffers: &[vk::CommandBuffer],
    swap_chain: &SwapChain,
    render_pass: vk::RenderPass,
    pipeline: &Pipeline,
    buffers: &[vk::Buffer],
) {
    for (i, &command_buffer) in command_buffers.iter().enumerate() {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();
        unsafe {
            logical_device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin command buffer");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.08, 1.0],
            },
        }];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(swap_chain.frame_buffers[i])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swap_chain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                buffers,
                vec![0; buffers.len()].as_slice(),
            );

            logical_device.cmd_draw(command_buffer, 6, 1, 0, 0);
            logical_device.cmd_end_render_pass(command_buffer);
            logical_device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer");
        }
    }
}

struct VulkanDebug {
    loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanDebug {
    fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let messenger = unsafe {
            loader
                .create_debug_utils_messenger(
                    &vk::DebugUtilsMessengerCreateInfoEXT::builder()
                        .message_severity(
                            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                        )
                        .message_type(
                            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                        )
                        .pfn_user_callback(Some(vulkan_debug_callback)),
                    None,
                )
                .expect("Failed to initialize debug messenger")
        };
        Self { loader, messenger }
    }
}

impl Drop for VulkanDebug {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

struct Surface {
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
    fn new(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> Self {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .expect("Failed to create window surface");

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Self {
            surface,
            surface_loader,
        }
    }

    fn get_capabilities(&self, physical_device: vk::PhysicalDevice) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
        .expect("Failed to get surface capabilities")
    }

    fn get_present_modes(&self, physical_device: vk::PhysicalDevice) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
        }
        .expect("Failed to get surface present modes")
    }

    fn get_formats(&self, physical_device: vk::PhysicalDevice) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
        .expect("Failed to get surface formats")
    }

    fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queue_family_index,
                self.surface,
            )
        }
        .expect("Failed to get surface support")
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

struct QueueFamilies {
    graphics_queue_index: Option<u32>,
    transfer_queue_index: Option<u32>,
}

impl QueueFamilies {
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> Self {
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        dbg!(&queue_family_properties);

        let mut graphics = None;
        let mut transfer = None;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surface.get_physical_device_surface_support(physical_device, index as u32)
            {
                graphics = Some(index as u32);
            }
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && (transfer.is_none()
                    || !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                transfer = Some(index as u32);
            }
        }

        Self {
            graphics_queue_index: graphics,
            transfer_queue_index: transfer,
        }
    }
}

struct Queues {
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
}

struct SwapChain {
    swap_chain: vk::SwapchainKHR,
    swap_chain_loader: ash::extensions::khr::Swapchain,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    frame_buffers: Vec<vk::Framebuffer>,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    image_available: Vec<vk::Semaphore>,
    rendering_finished: Vec<vk::Semaphore>,
    may_begin_drawing: Vec<vk::Fence>,
    frames_in_flight: u32,
    current_image: u32,
}

impl SwapChain {
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &Surface,
        queue_families: &QueueFamilies,
    ) -> Self {
        let surface_capabilities = surface.get_capabilities(physical_device);
        let extent = surface_capabilities.current_extent;

        let surface_present_modes = surface.get_present_modes(physical_device);
        let surface_formats = surface.get_formats(physical_device);
        let queue_families_index = [queue_families.graphics_queue_index.unwrap()];
        dbg!(&surface_capabilities);
        dbg!(&surface_present_modes);
        dbg!(&surface_formats);

        let surface_format = *surface_formats.first().unwrap();
        dbg!(&surface_format);

        let frames_in_flight = 3
            .max(surface_capabilities.min_image_count)
            .min(surface_capabilities.max_image_count);

        let swap_chain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(frames_in_flight)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_families_index)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO);

        let swap_chain_loader = ash::extensions::khr::Swapchain::new(&instance, &logical_device);
        let swap_chain = unsafe {
            swap_chain_loader
                .create_swapchain(&swap_chain_create_info, None)
                .expect("Failed to create swap chain")
        };

        let swap_chain_images = unsafe {
            swap_chain_loader
                .get_swapchain_images(swap_chain)
                .expect("Failed to get swap chain images")
        };

        let mut swap_chain_image_views = Vec::with_capacity(frames_in_flight as usize);
        for image in &swap_chain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            let image_view =
                unsafe { logical_device.create_image_view(&image_view_create_info, None) }
                    .expect("Failed to create image view");
            swap_chain_image_views.push(image_view);
        }

        let mut image_available = Vec::with_capacity(frames_in_flight as usize);
        let mut rendering_finished = Vec::with_capacity(frames_in_flight as usize);
        let mut may_begin_drawing = Vec::with_capacity(frames_in_flight as usize);

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..frames_in_flight {
            let semaphore_available = unsafe {
                logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create semaphore")
            };
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
                    .expect("Failed to create semaphore");

            let fence = unsafe {
                logical_device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create fence")
            };

            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            may_begin_drawing.push(fence);
        }

        Self {
            swap_chain,
            swap_chain_loader,
            images: swap_chain_images,
            image_views: swap_chain_image_views,
            frame_buffers: Vec::new(),
            surface_format,
            extent,
            frames_in_flight,
            current_image: 0,
            image_available,
            rendering_finished,
            may_begin_drawing,
        }
    }

    fn create_frame_buffers(&mut self, logical_device: &ash::Device, render_pass: vk::RenderPass) {
        for image_view in &self.image_views {
            let image_view = [*image_view];
            let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&image_view)
                .width(self.extent.width)
                .height(600)
                .layers(1);
            let frame_buffer = unsafe {
                logical_device
                    .create_framebuffer(&frame_buffer_create_info, None)
                    .expect("Failed to create frame buffer")
            };
            self.frame_buffers.push(frame_buffer);
        }
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }
        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for fb in &self.frame_buffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.image_views {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swap_chain_loader
            .destroy_swapchain(self.swap_chain, None)
    }
}

struct Pipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl Pipeline {
    fn new(
        logical_device: &ash::Device,
        swap_chain: &SwapChain,
        render_pass: &vk::RenderPass,
    ) -> Self {
        let vertex_shader_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(renderer_macros::include_glsl!("assets/shaders/tri.vert"));

        let fragment_shader_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(renderer_macros::include_glsl!("assets/shaders/tri.frag"));

        let vertex_shader_module = unsafe {
            logical_device
                .create_shader_module(&vertex_shader_create_info, None)
                .expect("Failed to create vertex shader module")
        };

        let fragment_shader_module = unsafe {
            logical_device
                .create_shader_module(&fragment_shader_create_info, None)
                .expect("Failed to create fragment shader module")
        };

        let entry_point = CString::new("main").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(&entry_point)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(&entry_point)
                .build(),
        ];

        let vertex_attribute_descriptions = [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .offset(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(1)
                .location(1)
                .offset(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .build(),
        ];

        let vertex_binding_descriptions = [
            vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(std::mem::size_of::<f32>() as u32 * 4)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
            vk::VertexInputBindingDescription::builder()
                .binding(1)
                .stride(std::mem::size_of::<f32>() as u32 * 4)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport = [vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(swap_chain.extent.width as f32)
            .height(swap_chain.extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)
            .build()];

        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(swap_chain.extent)
            .build()];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewport)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let colour_blend_attachment = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];

        let colour_blend_create_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colour_blend_attachment);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&colour_blend_create_info)
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .subpass(0)
            .build();

        let pipeline = unsafe {
            logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        unsafe {
            logical_device.destroy_shader_module(fragment_shader_module, None);
            logical_device.destroy_shader_module(vertex_shader_module, None);
        }

        Self {
            pipeline,
            pipeline_layout,
        }
    }

    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

struct Pools {
    command_pool_graphics: vk::CommandPool,
    command_pool_transfer: vk::CommandPool,
}

impl Pools {
    fn new(logical_device: &ash::Device, queue_families: &QueueFamilies) -> Self {
        let graphics_command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_queue_index.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool_graphics = unsafe {
            logical_device
                .create_command_pool(&graphics_command_pool_create_info, None)
                .expect("Failed to create graphics command pool")
        };

        let transfer_command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.transfer_queue_index.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool_transfer = unsafe {
            logical_device
                .create_command_pool(&transfer_command_pool_create_info, None)
                .expect("Failed to create transfer command pool")
        };

        Self {
            command_pool_graphics,
            command_pool_transfer,
        }
    }

    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.command_pool_graphics, None);
            logical_device.destroy_command_pool(self.command_pool_transfer, None);
        }
    }
}

struct Buffer {
    buffer: vk::Buffer,
    allocation: Allocation,
}

impl Buffer {
    fn new(
        allocator: &mut Allocator,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> Self {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let allocation_create_info = AllocationCreateInfo::builder()
            .name(name)
            .location(location)
            .build();

        let (buffer, allocation) =
            allocator.create_buffer(&buffer_create_info, &allocation_create_info);

        Self { buffer, allocation }
    }

    fn fill<T: Sized>(&mut self, data: &[T]) {
        let mapped_ptr = self.allocation.mapped_ptr().as_ptr() as *mut T;
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_ptr, data.len());
        }
    }

    fn cleanup(self, allocator: &mut Allocator) {
        let Self { buffer, allocation } = self;
        allocator.destroy_buffer(buffer, allocation);
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message); // TODO: Logging...
    vk::FALSE
}
