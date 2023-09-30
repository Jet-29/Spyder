use std::ffi::CString;

use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub fn run() {
    // Windowing
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let spyder = Spyder::new(window);
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

        let swap_chain = SwapChain::new(
            &instance,
            physical_device,
            &logical_device,
            &surface,
            &queue_families,
            &queues,
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
        }
    }
}

impl Drop for Spyder {
    fn drop(&mut self) {
        unsafe {
            self.swap_chain.cleanup(&self.logical_device);
            self.logical_device.destroy_device(None);

            std::mem::ManuallyDrop::drop(&mut self.surface);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
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
}

impl SwapChain {
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &Surface,
        queue_families: &QueueFamilies,
        queues: &Queues,
    ) -> Self {
        let surface_capabilities = surface.get_capabilities(physical_device);
        let surface_present_modes = surface.get_present_modes(physical_device);
        let surface_formats = surface.get_formats(physical_device);
        let queue_families_index = [queue_families.graphics_queue_index.unwrap()];
        dbg!(&surface_capabilities);
        dbg!(&surface_present_modes);
        dbg!(&surface_formats);

        let swap_chain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_formats.first().unwrap().format)
            .image_color_space(surface_formats.first().unwrap().color_space)
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

        let mut swap_chain_image_views = Vec::with_capacity(swap_chain_images.len());
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

        Self {
            swap_chain,
            swap_chain_loader,
            images: swap_chain_images,
            image_views: swap_chain_image_views,
        }
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device) {
        for image_view in &self.image_views {
            logical_device.destroy_image_view(*image_view, None);
        }

        self.swap_chain_loader
            .destroy_swapchain(self.swap_chain, None);
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
