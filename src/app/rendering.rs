use crate::app::scene::{SceneLayout, SceneObject};
use crate::app::shader_modules::fs_mod_render::RenderFragmentData;
use crate::app::shader_modules::vs_mod_render::RenderVertexData;
use crate::app::shader_modules::vs_mod_shadow::ShadowVertexData;
use crate::app::shader_modules::{fs_mod_render, fs_mod_shadow, vs_mod_render, vs_mod_shadow};
use crate::app::timing::TimingItems;
use crate::app::ui::GuiItems;
use crate::app::util::{CommonItems, MeshHolder};
use crate::app::UniformHolder;
use log::{info, warn};
use std::collections::BTreeMap;
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderingAttachmentInfo, RenderingInfo};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, RasterizationState};
use vulkano::pipeline::graphics::subpass::PipelineRenderingCreateInfo;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{AttachmentLoadOp, AttachmentStoreOp};
use vulkano::swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{Validated, VulkanError};
use vulkano::image::sampler::{Sampler, SamplerCreateInfo};
use winit::window::Window;

const SHADOW_MAP_EXTENT: [u32; 2] = [1000, 1000];

type UniformBufferHolder = BTreeMap<u32,
    (Subbuffer<ShadowVertexData>, Subbuffer<RenderVertexData>, Subbuffer<RenderFragmentData>)>;

pub struct RenderItems {
    // public
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,

    // configuration

    // access through methods
    recreate_swapchain: bool,

    // private
    shadow_attachment_image_view: Arc<ImageView>,
    shadow_pipeline: Arc<GraphicsPipeline>,
    shadow_viewport: Viewport,
    shadow_map_sampler: Arc<Sampler>,

    color_attachment_image_views: Vec<Arc<ImageView>>,
    depth_attachment_image_view: Arc<ImageView>,
    render_pipeline: Arc<GraphicsPipeline>,
    render_viewport: Viewport,

    uniform_buffer_holder: UniformBufferHolder,
}

impl RenderItems {
    pub fn set_recreate_swapchain(&mut self, value: bool) {
        self.recreate_swapchain = value;
    }

    pub fn new(common_items: &CommonItems, window: Arc<Window>, scene_layout: &SceneLayout) -> Self {
        let surface = Surface::from_window(common_items.instance.clone(), window.clone()).unwrap();

        let (swapchain, images) = {
            let surface_capabilities = common_items.device.physical_device()
                .surface_capabilities(&surface, Default::default()).unwrap();

            let (image_format, _) = common_items.device.physical_device()
                .surface_formats(&surface, Default::default()).unwrap()[0];

            Swapchain::new(
                common_items.device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    present_mode: PresentMode::Mailbox,
                    ..Default::default()
                }
            ).unwrap()
        };

        let (shadow_image_view,
             color_image_views,
             depth_image_view) = make_image_views(&common_items, &images);

        let shadow_pipeline = {
            let vertex_shader_module = vs_mod_shadow::load(common_items.device.clone()).expect("Failed to create shader");
            let fragment_shader_module = fs_mod_shadow::load(common_items.device.clone()).expect("Failed to create shader");
            let vertex_shader = vertex_shader_module.entry_point("main").unwrap();
            let fragment_shader = fragment_shader_module.entry_point("main").unwrap();

            let vertex_input_state = obj::Vertex::per_vertex().definition(&vertex_shader).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vertex_shader),
                PipelineShaderStageCreateInfo::new(fragment_shader)
            ];

            let layout = PipelineLayout::new(
                common_items.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(common_items.device.clone()).unwrap()
            ).unwrap();

            let dynamic_rendering_info = PipelineRenderingCreateInfo {
                depth_attachment_format: Some(Format::D16_UNORM),
                ..Default::default()
            };

            GraphicsPipeline::new(
                common_items.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState::simple()),
                        ..Default::default()
                    }),
                    multisample_state: Some(MultisampleState::default()),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(dynamic_rendering_info.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout.clone())
                }
            ).unwrap()
        };

        let render_pipeline = {
            let vertex_shader_module = vs_mod_render::load(common_items.device.clone()).expect("Failed to create shader");
            let fragment_shader_module = fs_mod_render::load(common_items.device.clone()).expect("Failed to create shader");
            let vertex_shader = vertex_shader_module.entry_point("main").unwrap();
            let fragment_shader = fragment_shader_module.entry_point("main").unwrap();

            let vertex_input_state = obj::Vertex::per_vertex().definition(&vertex_shader).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vertex_shader),
                PipelineShaderStageCreateInfo::new(fragment_shader)
            ];

            let layout = PipelineLayout::new(
                common_items.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(common_items.device.clone()).unwrap()
            ).unwrap();

            let dynamic_rendering_info = PipelineRenderingCreateInfo {
                color_attachment_formats: vec![Some(swapchain.image_format())],
                depth_attachment_format: Some(Format::D16_UNORM),
                ..Default::default()
            };

            GraphicsPipeline::new(
                common_items.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState {
                        cull_mode: CullMode::Back,
                        ..Default::default()
                    }),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState::simple()),
                        ..Default::default()
                    }),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        dynamic_rendering_info.color_attachment_formats.len() as u32,
                        ColorBlendAttachmentState::default()
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(dynamic_rendering_info.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout.clone())
                }
            ).unwrap()
        };

        let shadow_viewport = Viewport {
            extent: [SHADOW_MAP_EXTENT[0] as f32, SHADOW_MAP_EXTENT[1] as f32],
            ..Default::default()
        };
        let render_viewport = Viewport {
            extent: window.inner_size().into(),
            ..Default::default()
        };

        let mut uniform_buffer_holder = UniformBufferHolder::new();
        let buf_alloc = common_items.uniform_buffer_allocator.clone();

        for (id, scene_entity) in scene_layout.scene_entities.get_iter()
        {
            if let Some(scene_object) = scene_entity.downcast_ref::<SceneObject>()
                && scene_object.mesh_id.is_some()
            {
                uniform_buffer_holder.insert(*id, (
                    buf_alloc.allocate_sized().unwrap(),
                    buf_alloc.allocate_sized().unwrap(),
                    buf_alloc.allocate_sized().unwrap())
                );
            }
        }

        let shadow_map_sampler = Sampler::new(
            common_items.device.clone(),
            SamplerCreateInfo::default(),
        ).unwrap();

        RenderItems {
            window,
            swapchain,
            color_attachment_image_views: color_image_views,
            depth_attachment_image_view: depth_image_view,
            render_pipeline,
            recreate_swapchain: false,
            uniform_buffer_holder,
            shadow_pipeline,
            shadow_attachment_image_view: shadow_image_view,
            shadow_viewport,
            render_viewport,
            shadow_map_sampler
        }
    }

    pub fn frame_rendering_prep(&mut self,
                                common_items: &CommonItems,
                                timing_items: &mut TimingItems
    ) -> Option<SwapchainAcquireFuture>
    {
        let new_window_size = self.window.inner_size();
        if new_window_size.width == 0 {
            return None;
        }

        let mut frame_render_end_mutex = timing_items.get_frame_render_end_mutex();
        if frame_render_end_mutex.is_some() {
            frame_render_end_mutex.as_mut().unwrap().cleanup_finished();
        }
        drop(frame_render_end_mutex);

        if self.recreate_swapchain {
            info!("Recreating swapchain");
            let (new_swapchain, new_images) = self.swapchain.recreate(
                SwapchainCreateInfo {
                    image_extent: new_window_size.into(),
                    ..self.swapchain.create_info()
                }
            ).unwrap();

            self.swapchain = new_swapchain;
            (self.shadow_attachment_image_view,
             self.color_attachment_image_views,
             self.depth_attachment_image_view) = make_image_views(common_items, &new_images);
            self.render_viewport.extent = new_window_size.into();
            self.recreate_swapchain = false;
        }

        let (_image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(result) => result,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return None;
                },
                Err(error) => panic!("Failed to acquire next image: {error}")
            };

        if suboptimal {
            self.recreate_swapchain = true;
            return None;
        }

        Some(acquire_future)
    }

    fn draw_objects(&self,
                    scene_layout: &SceneLayout,
                    mesh_holder: &MeshHolder,
                    command_buffer_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
                    rendering_info: RenderingInfo,
                    viewport: &Viewport,
                    pipeline: Arc<GraphicsPipeline>,
                    mut buffer_write_and_descriptor_set: impl FnMut(&u32, Arc<GraphicsPipeline>) -> Arc<DescriptorSet>
    ) {
        command_buffer_builder
            .begin_rendering(rendering_info).unwrap()
            .set_viewport(0, [viewport.clone()].into_iter().collect()).unwrap()
            .bind_pipeline_graphics(pipeline.clone()).unwrap();

        for (id, scene_entity) in scene_layout.scene_entities.get_iter()
        {
            if scene_entity.downcast_ref::<SceneObject>().is_none() {
                continue
            }
            let scene_object = scene_entity.downcast_ref::<SceneObject>().unwrap();
            if scene_object.mesh_id.is_none() {
                continue
            }

            let descriptor_set = buffer_write_and_descriptor_set(id, pipeline.clone());

            command_buffer_builder.bind_descriptor_sets(PipelineBindPoint::Graphics,
                                                        pipeline.layout().clone(), 0, descriptor_set).unwrap();

            // --- mesh buffers ---
            let (vertex_buffer, index_buffer) = mesh_holder.get_by_id(scene_object.mesh_id.unwrap());

            command_buffer_builder.bind_vertex_buffers(0, vertex_buffer.clone()).unwrap();
            command_buffer_builder.bind_index_buffer(index_buffer.clone()).unwrap();

            // --- draw ---
            unsafe { command_buffer_builder.draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0).unwrap(); }
        }

        command_buffer_builder
            .end_rendering().unwrap();
    }

    pub fn frame_render(&mut self,
                        vulkan_items: &CommonItems,
                        timing_items: &mut TimingItems,
                        gui_items: &mut GuiItems,
                        acquire_future: SwapchainAcquireFuture,
                        scene_layout: &SceneLayout,
                        mesh_holder: &MeshHolder,
                        uniform_holder: &UniformHolder,
    ) {
        let image_index = acquire_future.image_index();
        let image_view = self.color_attachment_image_views[image_index as usize].clone();

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            vulkan_items.command_buffer_allocator.clone(),
            vulkan_items.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        self.draw_objects(
            scene_layout, mesh_holder, &mut command_buffer_builder,
            RenderingInfo {
                depth_attachment: Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some(1f32.into()),
                    ..RenderingAttachmentInfo::image_view(self.shadow_attachment_image_view.clone())
                }),
                ..Default::default()
            },
            &self.shadow_viewport, self.shadow_pipeline.clone(),
            |id, pipeline| {
                let (shadow_vertex_uniform, _, _) = uniform_holder.get(id).unwrap();
                let (shadow_vertex_uniform_buffer, _, _) = self.uniform_buffer_holder.get(id).unwrap();

                *shadow_vertex_uniform_buffer.write().unwrap() = *shadow_vertex_uniform;

                let descriptor_set_layout = pipeline.layout().set_layouts()[0].clone();
                let descriptor_set = DescriptorSet::new(
                    vulkan_items.descriptor_set_allocator.clone(),
                    descriptor_set_layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, shadow_vertex_uniform_buffer.clone()),
                    ],
                    []
                ).unwrap();

                descriptor_set
            }
        );

        self.draw_objects(
            scene_layout, mesh_holder, &mut command_buffer_builder,
            RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some([0.0, 0.0, 0.0, 1.0].into()),
                    ..RenderingAttachmentInfo::image_view(image_view.clone())
                })],
                depth_attachment: Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::DontCare,
                    clear_value: Some(1f32.into()),
                    ..RenderingAttachmentInfo::image_view(self.depth_attachment_image_view.clone())
                }),
                ..Default::default()
            },
            &self.render_viewport, self.render_pipeline.clone(),
            |id, pipeline| {
                let (_,
                    render_vertex_uniform,
                    render_fragment_uniform) = uniform_holder.get(id).unwrap();
                let (_,
                    render_vertex_uniform_buffer,
                    render_fragment_uniform_buffer) = self.uniform_buffer_holder.get(id).unwrap();

                *render_vertex_uniform_buffer.write().unwrap() = *render_vertex_uniform;
                *render_fragment_uniform_buffer.write().unwrap() = *render_fragment_uniform;



                let descriptor_set_layout = pipeline.layout().set_layouts()[0].clone();
                let descriptor_set = DescriptorSet::new(
                    vulkan_items.descriptor_set_allocator.clone(),
                    descriptor_set_layout.clone(),
                    [
                        WriteDescriptorSet::buffer(0, render_vertex_uniform_buffer.clone()),
                        WriteDescriptorSet::buffer(1, render_fragment_uniform_buffer.clone()),
                        WriteDescriptorSet::image_view_sampler(2, self.shadow_attachment_image_view.clone(),
                                                               self.shadow_map_sampler.clone())
                    ],
                    []
                ).unwrap();

                descriptor_set
            }
        );

        let command_buffer = command_buffer_builder.build().unwrap();

        let scene_future = acquire_future
            .then_execute(vulkan_items.queue.clone(), command_buffer.clone()).unwrap();

        let complete_future = gui_items.gui
            .draw_on_image(scene_future, image_view.clone())
            .then_swapchain_present(vulkan_items.queue.clone(),
                                    SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index))
            .boxed_send()
            .then_signal_fence_and_flush();

        match complete_future.map_err(Validated::unwrap) {
            Ok(future) => {
                *timing_items.get_frame_render_end_mutex() = Some(future);
            }
            Err(error) => {
                if error == VulkanError::OutOfDate {
                    self.recreate_swapchain = true;
                }
                *timing_items.get_frame_render_end_mutex() = None;

                warn!("Rendering failed: {error}");
            }
        }
    }
}

fn make_image_views(vulkan_items: &CommonItems, images: &[Arc<Image>]) -> (Arc<ImageView>, Vec<Arc<ImageView>>, Arc<ImageView>) {
    let color_image_views = images.iter().map(|image| {
        ImageView::new_default(image.clone()).unwrap()
    }).collect();

    let depth_image_view = ImageView::new_default(
        Image::new(
            vulkan_items.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default()
        ).unwrap()
    ).unwrap();

    let shadow_image_view = ImageView::new_default(
        Image::new(
            vulkan_items.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: [SHADOW_MAP_EXTENT[0], SHADOW_MAP_EXTENT[1], 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default()
        ).unwrap()
    ).unwrap();

    (shadow_image_view, color_image_views, depth_image_view)
}
