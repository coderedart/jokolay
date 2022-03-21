use crate::core::gui::Etx;
use crate::core::marker::{LivePack, MarkerManager, SelectedField};
use cached::{Cached, TimedCache};
use egui::{ColorImage, ScrollArea, TextStyle, TextureHandle, Ui, Window};
use jmf::json::{Author, PackDescription};

use jokolink::MumbleCtx;

use crate::WrapErr;
use strum::IntoEnumIterator;
use xxhash_rust::xxh3::xxh3_64;

impl Etx {
    pub async fn marker_gui(
        &mut self,
        mm: &mut MarkerManager,
        _mctx: &MumbleCtx,
    ) -> color_eyre::Result<()> {
        let mut load_pack = false;

        Window::new("Marker Manager")
            .scroll2([true, true])
            .show(&self.ctx, |ui| {
                ui.horizontal(|ui| {
                    for (&id, live_pack) in mm.packs.iter() {
                        let selected = mm.selected_pack.map(|i| i == id).unwrap_or_default();
                        if ui
                            .selectable_label(
                                selected,
                                format!("{id}: {}", &live_pack.pack.pack_description.name),
                            )
                            .clicked()
                        {
                            if selected {
                                mm.selected_pack = None;
                            } else {
                                mm.selected_pack = Some(id);
                            }
                        }
                    }
                });
                ui.separator();
                if let Some(selected_pack_id) = mm.selected_pack {
                    if let Some(_pack) = mm.packs.get_mut(&selected_pack_id) {
                        // live_pack_editor(pack, wtx, ui);
                    }
                }
                // if let Some(id) = mm.pack_editor.pack_id {
                //
                //     if let Some(pack) = mm.packs.get_mut(&id) {
                //
                //     }
                // }
                if let Some((errors, warnings)) = mm.latest_errors.as_ref() {
                    ui.collapsing("marker import errors", |ui| {
                        ui.label(format!("{:#?}", errors));
                    });
                    ui.collapsing("marker import warnings", |ui| {
                        ui.label(format!("{:#?}", warnings));
                    });
                }
                if ui.button("import taco pack").clicked() {
                    load_pack = true;
                }
            });

        if load_pack {
            mm.import_xml_pack().await?;
        }

        Ok(())
    }
}
pub fn live_pack_editor(
    live_pack: &mut LivePack,
    live_textures: &mut TimedCache<u64, TextureHandle>,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        for field in SelectedField::iter() {
            let selected = live_pack
                .pack_editor_state
                .selected_field
                .map(|sf| sf == field)
                .unwrap_or_default();
            if ui.selectable_label(selected, field.as_ref()).clicked() {
                if selected {
                    live_pack.pack_editor_state.selected_field = None;
                } else {
                    live_pack.pack_editor_state.selected_field = Some(field);
                }
            }
        }
    });
    ui.separator();
    if let Some(selected_field) = live_pack.pack_editor_state.selected_field {
        match selected_field {
            SelectedField::PackDescription => {
                pack_description_editor(
                    &mut live_pack.pack.pack_description,
                    &mut live_pack.pack_editor_state.selected_author,
                    &mut live_pack.dirty.pack_desc,
                    ui,
                );
            }
            SelectedField::ImagesDescriptions => {
                image_descripton_editor(live_pack, live_textures, ui);
            }
            SelectedField::TbinsDescriptions => {}
            SelectedField::Markers => {}
            SelectedField::Trails => {}
            SelectedField::Cats => {}
            SelectedField::CatTree => {}
        }
    }
}
pub fn image_descripton_editor(
    live_pack: &mut LivePack,
    live_textures: &mut TimedCache<u64, TextureHandle>,
    ui: &mut Ui,
) {
    let images_descriptions = &mut live_pack.pack.images_descriptions;
    let changed = &mut live_pack.dirty.image_desc;
    let selected_image = &mut live_pack.pack_editor_state.selected_image;
    ui.columns(2, |uis| {
        // let mut i = uis.split_mut(|_| true);
        // let c1 = i.next().unwrap();
        // let c2 = i.next().unwrap();
        if let [ref mut c1, ref mut c2] = uis {
            let delete = None;
            let row_height = c1.text_style_height(&TextStyle::Body);
            ScrollArea::vertical().show_rows(
                c1,
                row_height,
                images_descriptions.len(),
                |ui, row_range| {
                    for (&id, image_desc) in images_descriptions
                        .iter_mut()
                        .skip(row_range.start)
                        .take(row_range.end)
                    {
                        let selected = selected_image
                            .map(|selected_id| selected_id == id)
                            .unwrap_or_default();
                        if ui.selectable_label(selected, &image_desc.name).clicked() {
                            if selected {
                                *selected_image = None;
                            } else {
                                *selected_image = Some(id);
                            }
                        }
                    }

                    if let Some(id) = delete {
                        images_descriptions.remove(&id);
                    }
                },
            );
            if let Some((image_desc, id)) = selected_image
                .map(|id| images_descriptions.get_mut(&id).map(|desc| (desc, id)))
                .flatten()
            {
                if c2.text_edit_singleline(&mut image_desc.name).changed() {
                    *changed = true;
                }
                c2.label(format!(
                    "dimensions: {}x{}",
                    image_desc.width, image_desc.height
                ));
                // author_editor(&mut image_desc.credit, changed, &mut uis[1]);
                if c2.text_edit_singleline(&mut image_desc.extra).changed() {
                    *changed = true;
                }
                c2.checkbox(
                    &mut live_pack.pack_editor_state.preview_image,
                    "Preview Image",
                );
                if live_pack.pack_editor_state.preview_image {
                    let image_tex_id = *live_pack.loaded_textures.entry(id).or_insert_with(|| {
                        let image_path = live_pack
                            .path
                            .join("pack")
                            .join("images")
                            .join(format!("{id}.png"));
                        assert!(image_path.exists());
                        let i = image::open(image_path.as_path())
                            .wrap_err("failed to open image")
                            .unwrap();
                        let width = i.width();
                        let height = i.height();
                        let pixels = i.as_rgba8().unwrap();
                        let hash = xxh3_64(pixels.as_ref());

                        live_textures.cache_get_or_set_with(hash, || {
                            c2.ctx().load_texture(
                                image_desc.name.clone(),
                                ColorImage::from_rgba_unmultiplied(
                                    [width as usize, height as usize],
                                    pixels.as_flat_samples().as_slice(),
                                ),
                            )
                        });
                        hash
                    });
                    let handle = live_textures.cache_get(&image_tex_id).unwrap();
                    c2.image(handle.id(), handle.size_vec2());
                }
            }
        }
    });
}
pub fn pack_description_editor(
    pack_desc: &mut PackDescription,
    selected_author: &mut Option<u16>,
    changed: &mut bool,
    ui: &mut Ui,
) {
    ui.horizontal(|ui| {
        ui.label("pack name:");
        if ui.text_edit_singleline(&mut pack_desc.name).changed() {
            *changed = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("pack url:");
        if ui.text_edit_singleline(&mut pack_desc.url).changed() {
            *changed = true;
        }
    });
    ui.columns(2, |uis| {
        let mut delete = None;

        for (&id, author) in pack_desc.authors.iter_mut() {
            let selected = selected_author
                .map(|selected_id| selected_id == id)
                .unwrap_or_default();
            if uis[0].selectable_label(selected, &author.name).clicked() {
                if selected {
                    *selected_author = None;
                } else {
                    *selected_author = Some(id);
                }
            }
            if selected {
                author_editor(author, changed, &mut uis[1]);
                if uis[1].button("delete author").clicked() {
                    delete = Some(id);
                }
            }
        }
        if let Some(id) = delete {
            pack_desc.authors.remove(&id);
        }
    });
    ScrollArea::vertical().show(ui, |_ui| {});
    if ui.button("create author").clicked() {
        for i in 0..u16::MAX {
            if let std::collections::btree_map::Entry::Vacant(e) = pack_desc.authors.entry(i) {
                e.insert(Author::default());
                pack_desc.edited_author = Some(i);
                *changed = true;
                break;
            }
        }
    }
}

pub fn author_editor(author: &mut Author, changed: &mut bool, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("name:");
        if ui.text_edit_singleline(&mut author.name).changed() {
            *changed = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("email:");
        if ui
            .text_edit_singleline(&mut author.email)
            .on_hover_text("eg: joko@arenanet.com")
            .changed()
        {
            *changed = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("in_game_name:");
        if ui
            .text_edit_singleline(&mut author.ign)
            .on_hover_text("eg: joko.1234")
            .changed()
        {
            *changed = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("extra:");
        if ui
            .text_edit_singleline(&mut author.extra)
            .on_hover_text("eg: patreon / donations link or his cat name etc..")
            .changed()
        {
            *changed = true;
        }
    });
}
