use std::ffi::CString;

use ash::vk;

pub fn run() {
    // Vulkan lib integration.
    let entry: ash::Entry = ash::Entry::linked();

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

    let extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

    // Debug info
    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_callback),
        ..Default::default()
    };

    //  Finally initialize the instance
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&instance_create_info, None)
            .expect("Instance creation error")
    };

    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);

    let utils_messenger = unsafe {
        debug_utils
            .create_debug_utils_messenger(&debug_create_info, None)
            .expect("Failed to initialize debug messenger")
    };

    // Device
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to get devices")
    };

    for physical_device in &physical_devices {
        let props = unsafe { instance.get_physical_device_properties(*physical_device) };
        dbg!(props);
    }

    let (physical_device, _) = physical_devices
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

    let queue_family_properties =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    dbg!(&queue_family_properties);

    let queue_family_indices = {
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                found_graphics_q_index = Some(index as u32);
            }
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && (found_transfer_q_index.is_none()
                    || !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                found_transfer_q_index = Some(index as u32);
            }
        }
        (
            found_graphics_q_index.unwrap(),
            found_transfer_q_index.unwrap(),
        )
    };

    // Clean up. prob implement drop method where possible.
    unsafe {
        debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
        instance.destroy_instance(None)
    };
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
