use std::sync::Arc;

use crate::{
    animation::Animation,
    atlas::Atlas,
    bone::BoneData,
    c::{spAnimation, spBoneData, spSkeletonData, spSkeletonData_dispose, spSkin, spSlotData},
    c_interface::NewFromPtr,
    skin::Skin,
    slot::SlotData,
    sync_ptr::SyncPtr,
};

#[derive(Debug)]
pub struct SkeletonData {
    c_skeleton_data: SyncPtr<spSkeletonData>,
    owns_memory: bool,
    // TODO: this atlas arc is kind of a hack
    // skeleton data should keep a reference to data it requires
    // but that will not be an atlas if a custom attachment loader is used
    _atlas: Option<Arc<Atlas>>,
}

impl NewFromPtr<spSkeletonData> for SkeletonData {
    unsafe fn new_from_ptr(c_skeleton_data: *const spSkeletonData) -> Self {
        Self {
            c_skeleton_data: SyncPtr(c_skeleton_data as *mut spSkeletonData),
            owns_memory: false,
            _atlas: None,
        }
    }
}

impl SkeletonData {
    pub(crate) fn new(c_skeleton_data: *mut spSkeletonData, atlas: Option<Arc<Atlas>>) -> Self {
        Self {
            c_skeleton_data: SyncPtr(c_skeleton_data),
            owns_memory: true,
            _atlas: atlas,
        }
    }

    c_accessor_string!(version, version);
    c_accessor_string!(hash, hash);
    c_accessor_string!(images_path, imagesPath);
    c_accessor_string!(audio_path, audioPath);
    c_accessor!(x, set_x, x, f32);
    c_accessor!(y, set_y, y, f32);
    c_accessor!(width, set_width, width, f32);
    c_accessor!(height, set_height, height, f32);
    c_accessor!(bones_count, set_bones_count, bonesCount, i32);
    c_accessor!(slots_count, set_slots_count, slotsCount, i32);
    c_accessor!(skins_count, set_skins_count, skinsCount, i32);
    c_accessor!(events_count, set_events_count, eventsCount, i32);
    c_accessor!(animations_count, set_animations_count, animationsCount, i32);
    c_accessor!(
        ik_constraints_count,
        set_ik_constraints_count,
        ikConstraintsCount,
        i32
    );
    c_accessor!(
        transform_constraints_count,
        set_transform_constraints_count,
        transformConstraintsCount,
        i32
    );
    c_accessor!(
        path_constraints_count,
        set_path_constraints_count,
        pathConstraintsCount,
        i32
    );
    c_accessor_array!(
        bones,
        bones_mut,
        bone_at_index,
        bone_at_index_mut,
        SkeletonData,
        BoneData,
        spBoneData,
        bones,
        bones_count
    );
    c_accessor_array!(
        slots,
        slots_mut,
        slot_at_index,
        slot_at_index_mut,
        SkeletonData,
        SlotData,
        spSlotData,
        slots,
        slots_count
    );
    c_accessor_array!(
        skins,
        skins_mut,
        skin_at_index,
        skin_at_index_mut,
        SkeletonData,
        Skin,
        spSkin,
        skins,
        skins_count
    );
    c_accessor_array!(
        animations,
        animations_mut,
        animation_at_index,
        animation_at_index_mut,
        SkeletonData,
        Animation,
        spAnimation,
        animations,
        animations_count
    );
    c_ptr!(c_skeleton_data, spSkeletonData);

    // TODO: accessors and methods for the arrays in spSkeletonData
}

impl Drop for SkeletonData {
    fn drop(&mut self) {
        if self.owns_memory {
            unsafe {
                spSkeletonData_dispose(self.c_skeleton_data.0);
            }
        }
    }
}
