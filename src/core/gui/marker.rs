use crate::core::gui::Etx;
use crate::core::marker::MarkerManager;

use egui::{Ui, Window};
use jmf::json::{Author, FullPack, PackDescription};

use jmf::xmlpack::load::ErrorWithLocation;
use jokolink::MumbleCtx;

impl Etx {
    pub async fn marker_gui(
        &mut self,
        mm: &mut MarkerManager,
        _mctx: &MumbleCtx,
    ) -> color_eyre::Result<()> {
        let mut load_pack = false;

        Window::new("Marker Manager").show(&self.ctx, |ui| {
            for (id, live_pack) in mm.packs.iter() {
                ui.label(format!("{id}: {}", &live_pack.pack.pack_description.name));
            }
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
pub fn temp_pack_gui(
    _pack: &mut FullPack,
    _warnings: Vec<ErrorWithLocation>,
    _errors: Vec<ErrorWithLocation>,
) {
}
pub fn pack_description_editor(pack_desc: &mut PackDescription, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("pack name:");
        ui.text_edit_singleline(&mut pack_desc.name);
    });
    ui.horizontal(|ui| {
        ui.label("pack url:");
        ui.text_edit_singleline(&mut pack_desc.name);
    });
    // ui.horizontal(|ui| {
    //     ui.label("pack git link:");
    //     ui.text_edit_singleline(&mut pack_desc.name);
    // });
    if ui.button("create author").clicked() {
        for i in 0..u16::MAX {
            if let std::collections::btree_map::Entry::Vacant(e) = pack_desc.authors.entry(i) {
                e.insert(Author::default());
                pack_desc.edited_author = Some(i);
                break;
            }
        }
    }

    let mut delete = None;
    for (id, author) in pack_desc.authors.iter_mut() {
        ui.horizontal(|ui| {
            ui.label(&author.name);
            if ui.button("edit").clicked() {
                if let Some(prev_id) = pack_desc.edited_author {
                    if prev_id == *id {
                        pack_desc.edited_author = None;
                    }
                    pack_desc.edited_author = Some(*id);
                } else {
                    pack_desc.edited_author = Some(*id);
                }
            }
        });
        if let Some(edited_id) = pack_desc.edited_author {
            if edited_id == *id {
                author_editor(author, ui);
                if ui.button("delete").clicked() {
                    delete = Some(edited_id);
                    pack_desc.edited_author = None;
                }
            }
        }
    }
    if let Some(id) = delete {
        pack_desc.authors.remove(&id);
    }
}

pub fn author_editor(author: &mut Author, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("name:");
        ui.text_edit_singleline(&mut author.name);
    });
    ui.horizontal(|ui| {
        ui.label("email:");
        ui.text_edit_singleline(&mut author.email);
    });
    ui.horizontal(|ui| {
        ui.label("in_game_name:");
        ui.text_edit_singleline(&mut author.ign)
            .on_hover_text("account name like joko.1234");
    });
    ui.horizontal(|ui| {
        ui.label("extra:");
        ui.text_edit_singleline(&mut author.extra).on_hover_text(
            "anything to say about the author. he likes black cats or his patreon/donations link",
        );
    });
}
