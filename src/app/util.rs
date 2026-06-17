use log::{debug, error, info, warn};
use obj::{load_obj, Obj, Vertex};
use std::collections::{btree_map, BTreeMap, HashMap};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::VulkanLibrary;
use winit::event_loop::EventLoop;

const DEFAULT_INSTANCE_EXTENSIONS: InstanceExtensions = InstanceExtensions {
    ext_debug_utils: true,
    ..InstanceExtensions::empty()
};
const LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct CommonItems {
    pub _library: Arc<VulkanLibrary>,
    pub instance: Arc<Instance>,
    pub _debug_callback: DebugUtilsMessenger,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub uniform_buffer_allocator: Arc<SubbufferAllocator>,
}

pub fn get_debug_callback(instance: Arc<Instance>) -> DebugUtilsMessenger {
    pretty_env_logger::init();
    
    unsafe {
        DebugUtilsMessenger::new(
            instance.clone(),
            DebugUtilsMessengerCreateInfo {
                message_severity: DebugUtilsMessageSeverity::ERROR
                    | DebugUtilsMessageSeverity::WARNING
                    | DebugUtilsMessageSeverity::INFO
                    | DebugUtilsMessageSeverity::VERBOSE,
                message_type: DebugUtilsMessageType::GENERAL
                    | DebugUtilsMessageType::PERFORMANCE
                    | DebugUtilsMessageType::VALIDATION,
                ..DebugUtilsMessengerCreateInfo::user_callback(DebugUtilsMessengerCallback::new(
                    |message_severity,
                     message_type,
                     callback_data| {
                        if message_severity.intersects(DebugUtilsMessageSeverity::ERROR) {
                            error!("({:?}) {}", message_type, callback_data.message);
                        } else if message_severity.intersects(DebugUtilsMessageSeverity::WARNING) {
                            warn!("({:?}) {}", message_type, callback_data.message);
                        } else if message_severity.intersects(DebugUtilsMessageSeverity::INFO) {
                            info!("({:?}) {}", message_type, callback_data.message);
                        } else {
                            debug!("({:?}) {}", message_type, callback_data.message);
                        }
                    }
                ))

            }
        ).expect("Failed to create debug callback")
    }
}

pub fn get_common_vulkan_items(instance_extensions: Option<InstanceExtensions>,
                               device_extensions: Option<DeviceExtensions>,
                               device_features: Option<DeviceFeatures>,
                               queue_flag: QueueFlags,
                               event_loop: Option<&EventLoop<()>>
) -> CommonItems {
    let library = VulkanLibrary::new().expect("No local Vulkan library/dll");

    let mut library_layers = library.layer_properties().unwrap();
    LAYERS.iter().for_each(|layer| {
        library_layers.find(|l| {l.name() == *layer})
            .expect(format!("Layer {} not available in library", *layer).as_str());
    });

    let instance = Instance::new(
        library.clone(),
        InstanceCreateInfo {
            enabled_layers: LAYERS.iter().map(|l| {l.to_string()}).collect::<Vec<_>>(),
            enabled_extensions: DEFAULT_INSTANCE_EXTENSIONS.union(&instance_extensions.unwrap_or_default()),
            ..Default::default()
        }
    ).expect("Failed to create instance");

    let debug_callback = get_debug_callback(instance.clone());

    let physical_device = instance
        .enumerate_physical_devices().unwrap()
        .filter(|physical_device|
            physical_device.supported_extensions().contains(&device_extensions.unwrap_or_default()))
        .min_by_key(|physical_device| match physical_device.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            _ => 2,
        }).unwrap();

    let queue_family_index = physical_device
        .queue_family_properties().iter().enumerate()
        .position(|(index, queue_family_properties)| {
            queue_family_properties.queue_flags.contains(queue_flag)
                && event_loop.is_some_and(|event_loop| physical_device.presentation_support(index as u32, event_loop).unwrap())
        })
        .expect("No queue with appropriate support available") as u32;

    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions.unwrap_or_default(),
            enabled_features: device_features.unwrap_or_default(),
            ..Default::default()
        }
    ).expect("Failed to create device");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(
        device.clone())
    );
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(), Default::default())
    );
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(), StandardCommandBufferAllocatorCreateInfo::default()
    ));
    let uniform_buffer_allocator = Arc::new(SubbufferAllocator::new(
        memory_allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::UNIFORM_BUFFER,
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        }
    ));

    CommonItems {
        _library: library,
        instance,
        _debug_callback: debug_callback,
        device,
        queue,
        memory_allocator,
        descriptor_set_allocator,
        command_buffer_allocator,
        uniform_buffer_allocator,
    }
}

pub struct InitOption<T> {
    data: Option<T>
}

impl<T> InitOption<T> {

    pub fn none() -> Self {
        InitOption { data: None }
    }

    pub fn some(data: T) -> Self {
        InitOption { data: Some(data) }
    }

    pub fn get_ref(&self) -> &T {
        self.data.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.as_mut().unwrap()
    }
}

impl<T> Deref for InitOption<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> DerefMut for InitOption<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

pub fn radians_from_degrees(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

pub fn degrees_from_radians(radians: f32) -> f32 {
    radians / PI * 180.0
}

pub trait SettableId {
    fn set_id(&mut self, id: u32);
}

pub struct ObjectHolder<T> {
    cur_new_id: u32,
    objects: BTreeMap<u32, T>
}

impl<T> ObjectHolder<T> {

    pub fn new() -> ObjectHolder<T> {
        ObjectHolder {
            cur_new_id: 0,
            objects: BTreeMap::new(),
        }
    }

    pub fn add_with_id(&mut self, mut object: T) -> u32
    where T: SettableId
    {
        object.set_id(self.cur_new_id);
        self.add(object)
    }

    pub fn add(&mut self, object: T) -> u32 {
        let cur_id = self.cur_new_id;
        self.cur_new_id += 1;

        self.objects.insert(cur_id, object);
        cur_id
    }

    pub fn remove(&mut self, id: u32) {
        self.objects.remove(&id).expect("Id not present");
    }

    pub fn get_mut(&mut self, id: u32) -> &mut T {
        self.objects.get_mut(&id).expect("Id not present")
    }

    pub fn get(&self, id: u32) -> &T {
        self.objects.get(&id).expect("Id not present")
    }

    pub fn get_iter(&'_ self) -> btree_map::Iter<'_, u32, T> {
        self.objects.iter()
    }

    pub fn get_iter_mut(&'_ mut self) -> btree_map::IterMut<'_, u32, T> {
        self.objects.iter_mut()
    }
}

pub struct MeshHolder {
    vertex_buffer_holder: ObjectHolder<Subbuffer<[Vertex]>>,
    index_buffer_holder: ObjectHolder<Subbuffer<[u16]>>,

    name_to_id_map: HashMap<String, u32>
}

impl MeshHolder {

    pub fn new() -> Self {
        MeshHolder {
            vertex_buffer_holder: ObjectHolder::new(),
            index_buffer_holder: ObjectHolder::new(),
            name_to_id_map: HashMap::new(),
        }
    }

    pub fn get_by_id(&self, id: u32) -> (&Subbuffer<[Vertex]>, &Subbuffer<[u16]>) {
        (self.vertex_buffer_holder.get(id), self.index_buffer_holder.get(id))
    }

    pub fn get_id(&self, name: &str) -> u32 {
        *self.name_to_id_map.get(name).expect("Name not present")
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.name_to_id_map.contains_key(name)
    }

    pub fn add_mesh(&mut self, name: String,
                    buffers: (Subbuffer<[Vertex]>, Subbuffer<[u16]>)) -> u32
    {
        if self.has_name(&name) {
            panic!("Name already present")
        }

        let mesh_id = self.vertex_buffer_holder.add(buffers.0);
        self.index_buffer_holder.add(buffers.1);

        self.name_to_id_map.insert(name, mesh_id);

        mesh_id
    }

    pub fn load_and_add_mesh(&mut self, name: String, path: &PathBuf, common_items: &CommonItems) -> u32 {
        let buffers = load_mesh(path, common_items);
        self.add_mesh(name, buffers)
    }
}

pub fn load_mesh(path: &PathBuf, common_items: &CommonItems) -> (Subbuffer<[Vertex]>, Subbuffer<[u16]>) {
    info!("Reading object at {:?}", path);

    let buf_reader = BufReader::new(File::open(path).unwrap());
    let obj: Obj<Vertex, u16> = load_obj(buf_reader).unwrap();

    let vertex_buffer = Buffer::from_iter(
        common_items.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        obj.vertices
    ).unwrap();

    let index_buffer = Buffer::from_iter(
        common_items.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        obj.indices
    ).unwrap();

    (vertex_buffer, index_buffer)
} 
