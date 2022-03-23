use crate::config::ConfigManager;
use crate::core::gui::theme::ThemeManager;
use crate::core::marker::MarkerManager;
use crate::core::window::OverlayWindow;
use crate::{WgpuContext, WgpuContextImpl};
use color_eyre::eyre::WrapErr;
use egui::epaint::Vertex;
use egui::{
    ClippedPrimitive, Color32, ImageData, RawInput, RichText, TextureId, WidgetText, Window,
};
use glm::Vec2;
use jokolink::MumbleCtx;
use std::num::NonZeroU32;
use std::path::PathBuf;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, BufferAddress, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    Extent3d, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, IndexFormat, LoadOp,
    Operations, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState,
    PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexBufferLayout, VertexState, VertexStepMode,
};

mod config;
pub mod marker;
mod theme;
pub mod window;

pub struct Etx {
    pub ctx: egui::Context,
    pub enabled_windows: WindowEnabled,
    pub theme_manager: ThemeManager,
    pub egui_render_state: EguiRenderState,
}
pub struct EguiRenderState {
    pub font_texture: Texture,
    pub font_texture_view: TextureView,
    pub font_texture_bindgroup: BindGroup,
    pub pipeline: RenderPipeline,
    pub pipeline_layout: PipelineLayout,
    pub bindgroup_layout: BindGroupLayout,
    pub shader_module: ShaderModule,
}

impl Etx {
    pub fn new(
        wtx: WgpuContext,
        theme_folder_path: PathBuf,
        default_theme_name: &str,
        fonts_dir: PathBuf,
    ) -> color_eyre::Result<Self> {
        let ctx = egui::Context::default();
        let enabled_windows = WindowEnabled::default();
        let theme_manager = ThemeManager::new(theme_folder_path, fonts_dir, default_theme_name)
            .wrap_err("failed to create theme manager")?;

        ctx.set_fonts(theme_manager.font_definitions.clone());
        ctx.set_style(theme_manager.get_current_theme()?.style.clone());
        let wtx = wtx.write();
        let font_texture = wtx.device.create_texture(&TextureDescriptor {
            label: Some("egui font texture"),
            size: Extent3d {
                width: 2048,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        let font_texture_view = font_texture.create_view(&TextureViewDescriptor {
            label: Some("font texture view"),
            format: Some(TextureFormat::Rgba8UnormSrgb),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(
                NonZeroU32::new(1).expect("mip level count, 1 doesn't fit in nonzero u32"),
            ),
            base_array_layer: 0,
            array_layer_count: Some(
                NonZeroU32::new(1).expect("array layer count, 1 doesn't fit into nonzerou32"),
            ),
        });
        let font_texture_bindgroup = wtx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("font texture bindgroup"),
            layout: &wtx.linear_bindgroup_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&wtx.linear_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&font_texture_view),
                },
            ],
        });
        let bindgroup_layout = wtx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let pipeline_layout = wtx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("egui pipeline layout"),
                bind_group_layouts: &[&bindgroup_layout, &wtx.linear_bindgroup_layout],
                push_constant_ranges: &[],
            });
        let shader_module = wtx.device.create_shader_module(&include_wgsl!("egui.wgsl"));
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Unorm8x4,
        ];

        let pipeline = wtx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("egui render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 20,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &attributes,
                    }],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false,
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format: wtx.config.format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::OneMinusDstAlpha,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
                multiview: None,
            });

        Ok(Self {
            ctx,
            enabled_windows,
            theme_manager,
            egui_render_state: EguiRenderState {
                font_texture,
                font_texture_view,
                font_texture_bindgroup,
                pipeline,
                pipeline_layout,
                bindgroup_layout,
                shader_module,
            },
        })
    }
    pub async fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
        wtx: WgpuContext,
        cm: &mut ConfigManager,
        mctx: &mut MumbleCtx,
        // mm: &mut MarkerManager,
        // handle: tokio::runtime::Handle,
    ) -> color_eyre::Result<(
        egui::PlatformOutput,
        egui::TexturesDelta,
        Vec<ClippedPrimitive>,
    )> {
        self.ctx.begin_frame(input);
        {
            let ctx = self.ctx.clone();
            egui::containers::Area::new("top menu container").show(&ctx, |ui| {
                ui.style_mut().visuals.widgets.inactive.bg_fill =
                    Color32::from_rgba_unmultiplied(0, 0, 0, 100);
                let joko_icon_title = WidgetText::RichText(RichText::from("Joko\u{1F451}"))
                    .strong()
                    .text_style(egui::TextStyle::Heading);
                ui.menu_button(joko_icon_title, |ui| {
                    ui.checkbox(
                        &mut self.enabled_windows.config_window,
                        "show config window",
                    );
                    ui.checkbox(&mut self.enabled_windows.theme_window, "show theme window");
                    ui.checkbox(
                        &mut self.enabled_windows.overlay_controls,
                        "show overlay controls",
                    );
                    ui.checkbox(&mut self.enabled_windows.debug_window, "show debug window");
                    ui.checkbox(
                        &mut self.enabled_windows.marker_pack_window,
                        "show marker pack manager",
                    );
                    ui.checkbox(
                        &mut self.enabled_windows.mumble_window,
                        "show mumble window",
                    );
                });
            });

            self.theme_manager
                .gui(ctx.clone(), &mut self.enabled_windows.theme_window)?;
            ow.gui(ctx.clone(), &mut self.enabled_windows.overlay_controls, wtx)?;
            cm.gui(ctx.clone(), &mut self.enabled_windows.config_window)?;
            // self.marker_gui(mm, mctx).await?;
            Window::new("Mumble Window")
                .open(&mut self.enabled_windows.mumble_window)
                .scroll2([true, true])
                .show(&ctx, |ui| {
                    ui.set_width(300.0);

                    ui.horizontal(|ui| {
                        ui.label("mumble link name: ");
                        ui.label(&mctx.config.link_name);
                    });
                    ui.label("time since the last change");

                    ui.label(&format!("gw2 pid: {}", mctx.src.gw2_pid));
                    ui.label(&format!("gw2 xid: {}", mctx.src.gw2_window_handle));
                    ui.label(&format!("gw2 position: {:#?}", mctx.src.gw2_pos));
                    ui.label(&format!("gw2 size: {:#?}", mctx.src.gw2_size));
                    ui.collapsing("mumble link data", |ui| {
                        ui.label(&format!("{:#?}", mctx.src.get_link()));
                    });
                });
        }
        let egui::FullOutput {
            platform_output,
            needs_repaint: _,
            textures_delta,
            shapes,
        } = self.ctx.end_frame();
        let shapes = self.ctx.tessellate(shapes);
        Ok((platform_output, textures_delta, shapes))
    }
    pub fn draw_egui(
        &mut self,
        wtx: WgpuContext,
        textures_delta: egui::TexturesDelta,
        shapes: Vec<egui::ClippedPrimitive>,
        framebuffer_scale: Vec2,
    ) -> color_eyre::Result<()> {
        let _tex_update = !textures_delta.set.is_empty() || !textures_delta.free.is_empty();
        let mut wtx = wtx.write();
        for (id, delta) in textures_delta.set {
            assert_eq!(id, TextureId::Managed(0));
            let whole = delta.is_whole();
            let width = delta.image.width() as u32;
            let height = delta.image.height() as u32;

            let pixels: Vec<u8> = match delta.image {
                ImageData::Color(c) => c
                    .pixels
                    .into_iter()
                    .flat_map(|c32| c32.to_array())
                    .collect(),
                ImageData::Alpha(a) => a
                    .pixels
                    .into_iter()
                    .flat_map(|a8| Color32::from_white_alpha(a8).to_array())
                    .collect(),
            };
            let size = pixels.len() as u32;
            let position = [
                delta.pos.unwrap_or([0, 0])[0] as u32,
                delta.pos.unwrap_or([0, 0])[1] as u32,
            ];
            assert_eq!(size, width * height * 4);
            if whole {
                self.egui_render_state.font_texture =
                    wtx.device.create_texture(&TextureDescriptor {
                        label: Some("egui font texture"),
                        size: Extent3d {
                            width: 2048,
                            height: 64,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Rgba8UnormSrgb,
                        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    });
                self.egui_render_state.font_texture_view = self
                    .egui_render_state
                    .font_texture
                    .create_view(&TextureViewDescriptor {
                        label: Some("font texture view"),
                        format: Some(TextureFormat::Rgba8UnormSrgb),
                        dimension: Some(TextureViewDimension::D2),
                        aspect: TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: Some(
                            NonZeroU32::new(1)
                                .expect("mip level count, 1 doesn't fit in nonzero u32"),
                        ),
                        base_array_layer: 0,
                        array_layer_count: Some(
                            NonZeroU32::new(1)
                                .expect("array layer count, 1 doesn't fit into nonzerou32"),
                        ),
                    });
                self.egui_render_state.font_texture_bindgroup =
                    wtx.device.create_bind_group(&BindGroupDescriptor {
                        label: Some("font texture bindgroup"),
                        layout: &wtx.linear_bindgroup_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::Sampler(&wtx.linear_sampler),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::TextureView(
                                    &self.egui_render_state.font_texture_view,
                                ),
                            },
                        ],
                    });
            }
            wtx.queue.write_texture(
                ImageCopyTexture {
                    texture: &self.egui_render_state.font_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: position[0],
                        y: position[1],
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                pixels.as_slice(),
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some((width as u32 * 4).try_into()?),
                    rows_per_image: None,
                },
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            )
        }
        let size_in_points: [f32; 2] = [
            wtx.config.width as f32 / framebuffer_scale.x,
            wtx.config.height as f32 / framebuffer_scale.y,
        ];

        let ub = wtx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("egui uniform buffer"),
            contents: bytemuck::cast_slice(size_in_points.as_slice()),
            usage: BufferUsages::UNIFORM,
        });
        let ub_bindgroup = wtx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("egui uniform bindgroup"),
            layout: &self.egui_render_state.bindgroup_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &ub,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let vb_size: usize = shapes
            .iter()
            .map(|cm| {
                if let egui::epaint::Primitive::Mesh(m) = &cm.primitive {
                    m.vertices.len()
                } else {
                    0
                }
            })
            .sum::<usize>()
            * std::mem::size_of::<egui::epaint::Vertex>();
        let ib_size: usize = shapes
            .iter()
            .map(|cm| {
                if let egui::epaint::Primitive::Mesh(m) = &cm.primitive {
                    m.indices.len()
                } else {
                    0
                }
            })
            .sum::<usize>()
            * 4;
        let vb = wtx.device.create_buffer(&BufferDescriptor {
            label: Some("egui vertex buffer"),
            size: vb_size as BufferAddress,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        let ib = wtx.device.create_buffer(&BufferDescriptor {
            label: Some("egui index buffer"),
            size: (ib_size) as BufferAddress,
            usage: BufferUsages::INDEX,
            mapped_at_creation: true,
        });
        let WgpuContextImpl { fb, ce, .. } = &mut *wtx;
        if let Some((_fb, fbv)) = fb.as_ref() {
            let mut render_pass = ce
                .as_mut()
                .expect("failed to find command encoder at egui render pass")
                .begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: fbv,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
            render_pass.set_pipeline(&self.egui_render_state.pipeline);
            render_pass.set_bind_group(0, &ub_bindgroup, &[]);
            render_pass.set_vertex_buffer(0, vb.slice(..));
            render_pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32);
            {
                let mut vb_view = vb.slice(..).get_mapped_range_mut();
                let mut ib_view = ib.slice(..).get_mapped_range_mut();
                let mut vb_offset: usize = 0;
                let mut ib_offset: usize = 0;

                for primitive in shapes {
                    let ClippedPrimitive {
                        clip_rect,
                        primitive,
                    } = primitive;
                    if let egui::epaint::Primitive::Mesh(mesh) = primitive {
                        let vb_len = mesh.vertices.len() * 20;
                        let ib_len = mesh.indices.len() * std::mem::size_of::<u32>();
                        vb_view[vb_offset..(vb_offset + vb_len)].copy_from_slice(
                            bytemuck::cast_slice::<Vertex, u8>(mesh.vertices.as_slice()),
                        );
                        ib_view[ib_offset..(ib_offset + ib_len)]
                            .copy_from_slice(bytemuck::cast_slice(mesh.indices.as_slice()));
                        render_pass.set_bind_group(
                            1,
                            &self.egui_render_state.font_texture_bindgroup,
                            &[],
                        );

                        // Transform clip rect to physical pixels:
                        let pixels_per_point = framebuffer_scale.x;
                        let clip_min_x = pixels_per_point * clip_rect.min.x;
                        let clip_min_y = pixels_per_point * clip_rect.min.y;
                        let clip_max_x = pixels_per_point * clip_rect.max.x;
                        let clip_max_y = pixels_per_point * clip_rect.max.y;

                        // // Make sure clip rect can fit within a `u32`:
                        // let clip_min_x = clip_min_x.clamp(0.0, wtx.config.width as f32);
                        // let clip_min_y = clip_min_y.clamp(0.0, wtx.config.height as f32);
                        // let clip_max_x = clip_max_x.clamp(clip_min_x, wtx.config.width as f32);
                        // let clip_max_y = clip_max_y.clamp(clip_min_y, wtx.config.height as f32);

                        // let clip_min_x = clip_min_x.round() as i32;
                        // let clip_min_y = clip_min_y.round() as i32;
                        // let clip_max_x = clip_max_x.round() as i32;
                        // let clip_max_y = clip_max_y.round() as i32;
                        // wgpu cannot handle zero sized scissor rectangles, so this workaround is necessary
                        // https://github.com/gfx-rs/wgpu/issues/1750
                        if (clip_max_y - clip_min_y) >= 1.0 && (clip_max_x - clip_min_x) >= 1.0 {
                            render_pass.set_scissor_rect(
                                clip_min_x as u32,
                                (clip_min_y) as u32,
                                (clip_max_x - clip_min_x) as u32,
                                (clip_max_y - clip_min_y) as u32,
                            );

                            render_pass.draw_indexed(
                                ((ib_offset / 4) as u32)
                                    ..(((ib_offset / 4) + mesh.indices.len()) as u32),
                                (vb_offset / 20).try_into()?,
                                0..1,
                            );
                        }

                        vb_offset += vb_len;
                        ib_offset += ib_len;
                    }
                }
            }
            vb.unmap();
            ib.unmap();
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct WindowEnabled {
    pub config_window: bool,
    pub theme_window: bool,
    pub debug_window: bool,
    pub marker_pack_window: bool,
    pub overlay_controls: bool,
    pub mumble_window: bool,
}
