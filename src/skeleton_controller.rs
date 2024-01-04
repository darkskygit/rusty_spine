use std::{mem::take, sync::Arc};

use crate::{
    animation_state::AnimationState,
    animation_state_data::AnimationStateData,
    c::{c_void, spSkeleton_setToSetupPose},
    color::Color,
    draw::{CullDirection, SimpleDrawer},
    skeleton::Skeleton,
    skeleton_clipping::SkeletonClipping,
    skeleton_data::SkeletonData,
    BlendMode,
};

#[derive(Debug)]
pub struct SkeletonController {
    pub skeleton: Skeleton,
    pub animation_state: AnimationState,
    pub clipper: SkeletonClipping,
    pub settings: SkeletonControllerSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkeletonControllerSettings {
    pub premultiplied_alpha: bool,
    pub cull_direction: CullDirection,
}

impl Default for SkeletonControllerSettings {
    fn default() -> Self {
        Self {
            premultiplied_alpha: false,
            cull_direction: CullDirection::Clockwise,
        }
    }
}

impl SkeletonControllerSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_premultiplied_alpha(self, premultiplied_alpha: bool) -> Self {
        Self {
            premultiplied_alpha,
            ..self
        }
    }

    pub fn with_cull_direction(self, cull_direction: CullDirection) -> Self {
        Self {
            cull_direction,
            ..self
        }
    }
}

impl SkeletonController {
    pub fn new(
        skeleton_data: Arc<SkeletonData>,
        animation_state_data: Arc<AnimationStateData>,
    ) -> Self {
        let mut skeleton = Skeleton::new(skeleton_data);
        unsafe {
            spSkeleton_setToSetupPose(skeleton.c_ptr());
        }
        skeleton.update_world_transform();
        Self {
            skeleton,
            animation_state: AnimationState::new(animation_state_data),
            clipper: SkeletonClipping::new(),
            settings: SkeletonControllerSettings::default(),
        }
    }

    pub fn with_settings(self, settings: SkeletonControllerSettings) -> Self {
        Self { settings, ..self }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.animation_state.update(delta_seconds);
        self.animation_state.apply(&mut self.skeleton);
        self.skeleton.update_world_transform();
    }

    pub fn renderables(&mut self) -> Vec<SkeletonRenderable> {
        let renderables = SimpleDrawer {
            cull_direction: self.settings.cull_direction,
            premultiplied_alpha: self.settings.premultiplied_alpha,
        }
        .draw(&mut self.skeleton, Some(&mut self.clipper));
        renderables
            .into_iter()
            .map(|mut renderable| SkeletonRenderable {
                slot_index: renderable.slot_index,
                vertices: take(&mut renderable.vertices),
                uvs: take(&mut renderable.uvs),
                indices: take(&mut renderable.indices),
                color: renderable.color,
                dark_color: renderable.dark_color,
                blend_mode: renderable.blend_mode,
                premultiplied_alpha: self.settings.premultiplied_alpha,
                attachment_renderer_object: renderable.attachment_renderer_object,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct SkeletonRenderable {
    pub slot_index: i32,
    pub vertices: Vec<[f32; 2]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
    pub color: Color,
    pub dark_color: Color,
    pub blend_mode: BlendMode,
    pub premultiplied_alpha: bool,
    pub attachment_renderer_object: Option<*const c_void>,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_spineboy_instance_data;
    use crate::SkeletonController;
    use std::env;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};

    #[test]
    fn test_generated_data_skeletoncontroller() {
        let reference_filename = if cfg!(feature = "spine38") {
            "assets/test-reference-files/simple_drawer.spineboy_spine38"
        } else {
            "assets/test-reference-files/simple_drawer.spineboy_spine41"
        };

        let (skeleton_data, animation_state_data) = test_spineboy_instance_data();
        let mut controller = SkeletonController::new(skeleton_data, animation_state_data);
        let generated_data = generate_test_data(&mut controller, "run", 0.05, 60).unwrap();

        if env::var("GENERATE_TEST_REFERENCE_FILES").is_ok() {
            eprintln!("Generating test reference file {}", reference_filename);
            let mut reference_file = File::create(reference_filename).unwrap();
            reference_file.write(generated_data.as_slice()).unwrap();
        } else {
            let reference_file = BufReader::new(File::open(reference_filename).unwrap());
            for (reference_line, test_line) in reference_file.lines().zip(generated_data.lines()) {
                let reference_line = reference_line.unwrap();
                let test_line = test_line.unwrap();
                assert_eq!(reference_line, test_line);
            }
        }
    }

    fn generate_test_data(
        controller: &mut SkeletonController,
        animation_name: &str,
        delta_seconds: f32,
        frame_count: i32,
    ) -> std::io::Result<Vec<u8>> {
        controller
            .animation_state
            .set_animation_by_name(0, animation_name, true)
            .unwrap();

        let mut d = Vec::new();

        for frame in 0..frame_count {
            for renderable in controller.renderables() {
                write!(d, "frame {} slot {}", frame, renderable.slot_index)?;
                write!(d, " blend={}", renderable.blend_mode as i32)?;
                write!(d, " pma={}", renderable.premultiplied_alpha)?;
                write!(
                    d,
                    " color=({:.4},{:.4},{:.4},{:.4})",
                    renderable.color.r, renderable.color.g, renderable.color.b, renderable.color.a
                )?;
                write!(
                    d,
                    " dark=({:.4},{:.4},{:.4},{:.4})",
                    renderable.dark_color.r,
                    renderable.dark_color.g,
                    renderable.dark_color.b,
                    renderable.dark_color.a
                )?;

                write!(
                    d,
                    "\nframe {} slot {} indices",
                    frame, renderable.slot_index
                )?;
                for index in renderable.indices {
                    write!(d, " {}", index)?;
                }

                write!(
                    d,
                    "\nframe {} slot {} vertices",
                    frame, renderable.slot_index
                )?;
                for vertex in renderable.vertices {
                    write!(d, " {:.0} {:.0}", vertex[0], vertex[1])?;
                }

                write!(d, "\nframe {} slot {} uvs", frame, renderable.slot_index)?;
                for uv in renderable.uvs {
                    write!(d, " {:.4} {:.4}", uv[0], uv[1])?;
                }

                write!(d, "\n")?;
            }
            controller.update(delta_seconds);
        }

        Ok(d)
    }
}
