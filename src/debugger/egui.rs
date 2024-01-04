use egui::*;
use egui_extras::*;

use crate::{AnimationState, Attachment, BoneHandle, Skeleton, SlotHandle};

enum Command {
    SetAnimationByName {
        track_index: i32,
        name: String,
    },
    SetEmptyAnimation {
        track_index: i32,
    },
    ClearTrack {
        track_index: i32,
    },
    SetToSetupPose,
    SetAttachment {
        slot_index: i32,
        attachment: Option<Attachment>,
    },
}

pub fn egui_spine_debugger(
    ctx: &Context,
    title: impl Into<WidgetText>,
    skeleton: &mut Skeleton,
    animation_state: &mut AnimationState,
) {
    let unique = format!("{:?}{:?}", skeleton.c_ptr(), animation_state.c_ptr());
    let mut bone_windows = ctx
        .data()
        .get_temp::<Vec<BoneHandle>>(Id::new(format!("{}-bones", unique.clone())))
        .unwrap_or(vec![]);
    Window::new(title)
        .id(Id::new(unique.clone()))
        .show(ctx, |ui| {
            let mut commands = vec![];
            ui.heading("Tracks");
            let table = TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Size::initial(16.0).at_least(16.0))
                .column(Size::initial(110.0).at_least(110.0))
                .column(Size::initial(40.0).at_least(40.0))
                .column(Size::initial(70.0).at_least(70.0))
                .column(Size::remainder().at_least(60.0))
                .resizable(true);
            table
                .header(20., |mut header| {
                    header.col(|ui| {
                        ui.label("#");
                    });
                    header.col(|ui| {
                        ui.label("Animation");
                    });
                    header.col(|ui| {
                        ui.label("Alpha");
                    });
                    header.col(|ui| {
                        ui.label("Timescale");
                    });
                })
                .body(|mut body| {
                    for (track_index, track) in animation_state.tracks_mut().enumerate() {
                        if let Some(mut track) = track {
                            body.row(20., |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{}", track_index));
                                });
                                row.col(|ui| {
                                    let mut selected = track.animation().name().to_owned();
                                    egui::ComboBox::new(format!("track {}", track_index), "")
                                        .selected_text(track.animation().name())
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut selected,
                                                "<none>".to_owned(),
                                                "<none>",
                                            );
                                            for animation in skeleton.data().animations() {
                                                ui.selectable_value(
                                                    &mut selected,
                                                    animation.name().to_owned(),
                                                    animation.name(),
                                                );
                                            }
                                        });
                                    if selected != track.animation().name() {
                                        if selected == "<none>" {
                                            commands.push(Command::ClearTrack {
                                                track_index: track_index as i32,
                                            });
                                        } else {
                                            commands.push(Command::SetAnimationByName {
                                                track_index: track_index as i32,
                                                name: selected,
                                            });
                                        }
                                    }
                                });
                                row.col(|ui| {
                                    let mut alpha = track.alpha();
                                    ui.add(
                                        DragValue::new(&mut alpha)
                                            .speed(0.01)
                                            .clamp_range(0.0..=1.0),
                                    );
                                    track.set_alpha(alpha);
                                });
                                row.col(|ui| {
                                    let mut timescale = track.timescale();
                                    ui.add(DragValue::new(&mut timescale).speed(0.01));
                                    track.set_timescale(timescale);
                                });
                            });
                        } else {
                            body.row(20., |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{}", track_index));
                                });
                                row.col(|ui| {
                                    let mut selected = "<none>".to_owned();
                                    egui::ComboBox::new(format!("track {}", track_index), "")
                                        .selected_text("<none>")
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut selected,
                                                "<none>".to_owned(),
                                                "<none>",
                                            );
                                            for animation in skeleton.data().animations() {
                                                ui.selectable_value(
                                                    &mut selected,
                                                    animation.name().to_owned(),
                                                    animation.name(),
                                                );
                                            }
                                        });
                                    if selected != "<none>" {
                                        commands.push(Command::SetAnimationByName {
                                            track_index: track_index as i32,
                                            name: selected,
                                        });
                                    }
                                });
                            });
                        }
                    }
                    body.row(20., |mut row| {
                        row.col(|ui| {
                            if ui.button("+").clicked() {
                                let track_count = animation_state.tracks_count();
                                animation_state.set_empty_animation(track_count, 0.);
                                commands.push(Command::SetEmptyAnimation {
                                    track_index: track_count as i32,
                                });
                            }
                        });
                    });
                });

            ui.add_space(16.);
            ui.heading("Bones");
            egui_draw_bones(
                ui,
                skeleton.bone_root().handle(),
                skeleton,
                animation_state,
                &mut bone_windows,
            );

            ui.add_space(16.);
            ui.heading("Slots");
            egui_draw_slots(
                ui,
                skeleton.bone_root().handle(),
                skeleton,
                animation_state,
                &mut commands,
                true,
            );

            ui.add_space(16.);
            ui.horizontal(|ui| {
                if ui.button("Setup Pose").clicked() {
                    commands.push(Command::SetToSetupPose);
                }
                ui.label("Skin:");
                let current_skin = if let Some(skin) = skeleton.skin() {
                    skin.name().to_owned()
                } else {
                    "default".to_owned()
                };
                let skins: Vec<String> = skeleton
                    .data()
                    .skins()
                    .map(|skin| skin.name().to_owned())
                    .collect();
                let mut selected = current_skin.clone();
                egui::ComboBox::new("skin", "")
                    .selected_text(current_skin.clone())
                    .show_ui(ui, |ui| {
                        for skin in skins.iter() {
                            ui.selectable_value(&mut selected, skin.clone(), skin);
                        }
                    });
                if selected != current_skin {
                    skeleton.set_skin_by_name(&selected).unwrap();
                }
            });

            for command in commands.into_iter() {
                match command {
                    Command::SetAnimationByName { track_index, name } => {
                        animation_state
                            .set_animation_by_name(track_index as usize, &name, true)
                            .unwrap();
                    }
                    Command::SetEmptyAnimation { track_index } => {
                        animation_state.set_empty_animation(track_index as usize, 0.);
                    }
                    Command::ClearTrack { track_index } => {
                        animation_state.clear_track(track_index as usize);
                    }
                    Command::SetToSetupPose => {
                        skeleton.set_to_setup_pose();
                    }
                    Command::SetAttachment {
                        slot_index,
                        attachment,
                    } => unsafe {
                        skeleton
                            .slot_at_index_mut(slot_index as usize)
                            .unwrap()
                            .set_attachment(attachment);
                    },
                }
            }
        });

    let mut remove_bone = None;
    for bone_window in bone_windows.iter() {
        if let Some(mut bone) = bone_window.get_mut(skeleton) {
            let mut open = true;
            egui::Window::new(bone.data().name())
                .id(Id::new(format!("{:?}", bone.c_ptr())))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Translation");
                    ui.horizontal(|ui| {
                        let mut x = bone.x();
                        ui.add(egui::DragValue::new(&mut x).speed(1.));
                        bone.set_x(x);
                        let mut y = bone.y();
                        ui.add(egui::DragValue::new(&mut y).speed(1.));
                        bone.set_y(y);
                    });
                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        let mut scale_x = bone.scale_x();
                        ui.add(egui::DragValue::new(&mut scale_x).speed(0.01));
                        bone.set_scale_x(scale_x);
                        let mut scale_y = bone.scale_y();
                        ui.add(egui::DragValue::new(&mut scale_y).speed(0.01));
                        bone.set_scale_y(scale_y);
                    });
                });
            if !open {
                remove_bone = Some(bone_window);
            }
        }
    }

    if let Some(remove) = remove_bone {
        if let Some(index) = bone_windows.iter().position(|handle| handle == remove) {
            bone_windows.remove(index);
        }
    }

    ctx.data()
        .insert_temp(Id::new(format!("{}-bones", unique)), bone_windows);
}

fn egui_draw_bones(
    ui: &mut Ui,
    bone_handle: BoneHandle,
    skeleton: &mut Skeleton,
    animation_state: &mut AnimationState,
    bone_windows: &mut Vec<BoneHandle>,
) {
    if let Some(bone) = bone_handle.get(&skeleton) {
        let bone_name = bone.data().name().to_owned();
        let child_handles: Vec<BoneHandle> = bone.children().map(|bone| bone.handle()).collect();
        let id = ui.make_persistent_id(bone_name.clone());
        if !child_handles.is_empty() {
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                .show_header(ui, |ui| {
                    if ui.link(bone_name).clicked() {
                        if let Some(index) =
                            bone_windows.iter().position(|other| *other == bone_handle)
                        {
                            bone_windows.remove(index);
                        } else {
                            bone_windows.push(bone_handle);
                        }
                    }
                })
                .body(|ui| {
                    for child_handle in child_handles.into_iter() {
                        egui_draw_bones(ui, child_handle, skeleton, animation_state, bone_windows);
                    }
                });
        } else {
            if ui.link(bone_name).clicked() {
                if let Some(index) = bone_windows.iter().position(|other| *other == bone_handle) {
                    bone_windows.remove(index);
                } else {
                    bone_windows.push(bone_handle);
                }
            }
        }
    }
}

fn egui_draw_slots(
    ui: &mut Ui,
    bone_handle: BoneHandle,
    skeleton: &mut Skeleton,
    animation_state: &mut AnimationState,
    commands: &mut Vec<Command>,
    root: bool,
) {
    if let Some(bone) = bone_handle.get(&skeleton) {
        let bone_name = bone.data().name().to_owned();
        let child_handles: Vec<BoneHandle> = bone.children().map(|bone| bone.handle()).collect();
        let slot_handles: Vec<SlotHandle> = skeleton
            .slots()
            .filter(|slot| slot.bone().handle() == bone_handle)
            .map(|slot| slot.handle())
            .collect();
        if !slot_handles.is_empty() || root {
            if has_slot_children(bone_handle, skeleton, true) || root {
                let id = ui.make_persistent_id(format!("bone-slot-{}", bone_name.clone()));
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                )
                .show_header(ui, |ui| {
                    if root && slot_handles.is_empty() {
                        ui.label(bone.data().name());
                    }
                    ui.vertical(|ui| {
                        for slot_handle in slot_handles.iter() {
                            if let Some(slot) = slot_handle.get(skeleton) {
                                if let Some(attachment) =
                                    egui_slot_dropdown(ui, *slot_handle, skeleton)
                                {
                                    commands.push(Command::SetAttachment {
                                        slot_index: slot.data().index() as i32,
                                        attachment,
                                    });
                                }
                            }
                        }
                    });
                })
                .body(|ui| {
                    for child_handle in child_handles.into_iter() {
                        egui_draw_slots(
                            ui,
                            child_handle,
                            skeleton,
                            animation_state,
                            commands,
                            false,
                        );
                    }
                });
            } else {
                ui.vertical(|ui| {
                    for slot_handle in slot_handles.iter() {
                        if let Some(slot) = slot_handle.get(skeleton) {
                            if let Some(attachment) = egui_slot_dropdown(ui, *slot_handle, skeleton)
                            {
                                commands.push(Command::SetAttachment {
                                    slot_index: slot.data().index() as i32,
                                    attachment,
                                });
                            }
                        }
                    }
                });
            }
        } else {
            for child_handle in child_handles.into_iter() {
                egui_draw_slots(ui, child_handle, skeleton, animation_state, commands, false);
            }
        }
    }
}

fn has_slot_children(bone_handle: BoneHandle, skeleton: &Skeleton, root: bool) -> bool {
    if let Some(bone) = bone_handle.get(&skeleton) {
        let child_handles: Vec<BoneHandle> = bone.children().map(|bone| bone.handle()).collect();
        let slot_handles: Vec<SlotHandle> = skeleton
            .slots()
            .filter(|slot| slot.bone().handle() == bone_handle)
            .map(|slot| slot.handle())
            .collect();
        if !slot_handles.is_empty() && !root {
            return true;
        }
        for child_handle in child_handles.iter() {
            if has_slot_children(*child_handle, skeleton, false) {
                return true;
            }
        }
        false
    } else {
        false
    }
}

fn egui_slot_dropdown(
    ui: &mut Ui,
    slot_handle: SlotHandle,
    skeleton: &Skeleton,
) -> Option<Option<Attachment>> {
    let skin_handle = skeleton
        .skin()
        .map(|skin| skin.handle())
        .unwrap_or(skeleton.data().default_skin().handle());
    let mut set_attachment_name = None;
    if let Some(slot) = slot_handle.get(skeleton) {
        let current = if let Some(attachment) = slot.attachment() {
            attachment.name().to_owned()
        } else {
            "<none>".to_owned()
        };

        let mut attachments = vec![];
        if let Some(skin) = skin_handle.get(&skeleton.data()) {
            for attachment_entry in skin.attachments() {
                if attachment_entry.slot_index == slot.data().index() as i32 {
                    attachments.push(attachment_entry.attachment);
                }
            }
        }

        let mut selected = current.clone();
        egui::ComboBox::from_label(slot.data().name())
            .selected_text(current.clone())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, "<none>".to_owned(), "<none>");
                for attachment in attachments.iter() {
                    ui.selectable_value(
                        &mut selected,
                        attachment.name().to_owned(),
                        attachment.name(),
                    );
                }
            });
        if selected != current {
            if selected == "<none>" {
                set_attachment_name = Some(None);
            } else {
                for attachment in attachments.into_iter() {
                    if attachment.name() == selected {
                        set_attachment_name = Some(Some(attachment));
                        break;
                    }
                }
            }
        }
    }
    set_attachment_name
}
