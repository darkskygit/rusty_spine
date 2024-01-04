use std::{
    collections::HashMap,
    mem::take,
    sync::{Arc, Mutex},
};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_resource::{PrimitiveTopology, VertexFormat},
    },
    sprite::Mesh2dHandle,
};
use rusty_spine::{
    draw::CullDirection, AnimationStateData, Atlas, Error, SkeletonController,
    SkeletonControllerSettings, SkeletonJson,
};

#[cfg(feature = "egui_debugger")]
use {
    bevy_egui::{EguiContext, EguiPlugin},
    rusty_spine::debugger::egui::egui_spine_debugger,
};

#[derive(Component)]
pub struct Spine {
    controller: SkeletonController,
}

#[derive(Debug)]
struct SpineTexture {
    path: String,
}

struct Demo {
    atlas: Vec<u8>,
    json: Vec<u8>,
    dir: String,
    animation: String,
    position: Vec2,
    scale: f32,
    skin: Option<String>,
    note: String,
}

struct Demos(Vec<Demo>);

#[derive(Clone)]
struct DemoLoad(usize);

#[derive(Component)]
struct NoteText;

fn make_cube(mesh: &mut Mesh) {
    let indices = Indices::U32(vec![]);

    let positions: Vec<[f32; 3]> = vec![];
    let normals: Vec<[f32; 3]> = vec![];
    let uvs: Vec<[f32; 2]> = vec![];

    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
}

struct PersistentImageHandles {
    handles: Arc<Mutex<Vec<(String, Handle<Image>)>>>,
    remember: Arc<Mutex<Vec<String>>>,
    forget: Arc<Mutex<Vec<String>>>,
}

fn main() {
    let image_handles: Arc<Mutex<Vec<(String, Handle<Image>)>>> = Arc::new(Mutex::new(Vec::new()));
    let image_remember: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let image_forget: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let remember = image_remember.clone();
    rusty_spine::extension::set_create_texture_cb(move |page, path| {
        remember.lock().unwrap().push(path.to_owned());
        page.renderer_object().set(SpineTexture {
            path: path.to_owned(),
        });
    });
    let forget = image_forget.clone();
    rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
        forget.lock().unwrap().push(
            page.renderer_object()
                .get_unchecked::<SpineTexture>()
                .path
                .clone(),
        );
        page.renderer_object().dispose::<SpineTexture>();
    });
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(PersistentImageHandles {
            handles: image_handles,
            remember: image_remember,
            forget: image_forget,
        })
        .insert_resource(Demos(vec![Demo {
            atlas: include_bytes!("../assets/spineboy-3.8/export/spineboy.atlas").to_vec(),
            json: include_bytes!("../assets/spineboy-3.8/export/spineboy-pro.json").to_vec(),
            dir: "spineboy-3.8/export/".to_owned(),
            animation: "portal".to_owned(),
            position: Vec2::new(0., -400.),
            scale: 0.5,
            skin: None,
            note: "".to_owned(),
        }]))
        .add_event::<DemoLoad>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(demo_load)
        .add_system(demo_next)
        .add_system(spine_update);
    #[cfg(feature = "egui_debugger")]
    app.add_plugin(EguiPlugin).add_system(spine_debugger);
    app.run();
}

fn startup(
    mut commands: Commands,
    mut ev_demo_load: EventWriter<DemoLoad>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    ev_demo_load.send(DemoLoad(0));
    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section(
            "press space for next demo",
            TextStyle {
                font: asset_server.load("FiraMono-Medium.ttf"),
                font_size: 22.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment {
            horizontal: HorizontalAlign::Center,
            vertical: VerticalAlign::Center,
        }),
        transform: Transform::from_xyz(0., 320., 1.),
        ..Default::default()
    });
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: asset_server.load("FiraMono-Medium.ttf"),
                    font_size: 22.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            }),
            transform: Transform::from_xyz(0., -320., 1.),
            ..Default::default()
        })
        .insert(NoteText);
}

fn demo_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_demo_load: EventReader<DemoLoad>,
    mut note_query: Query<&mut Text, With<NoteText>>,
    entity_query: Query<Entity, With<Spine>>,
    demos: Res<Demos>,
) {
    for event in ev_demo_load.iter() {
        for entity in entity_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        let demo = &demos.0[event.0];
        let mut controller = load_skeleton(&demo.atlas, &demo.json, &demo.dir).unwrap();
        let _ = controller
            .animation_state
            .set_animation_by_name(0, &demo.animation, true);
        if let Some(skin) = &demo.skin {
            let _ = controller.skeleton.set_skin_by_name(skin);
        }
        let mut slots = HashMap::new();
        commands
            .spawn_bundle((
                Transform::from_scale(Vec3::ONE * demo.scale),
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
            ))
            .with_children(|parent| {
                for slot in controller.skeleton.slots() {
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    make_cube(&mut mesh);
                    let mesh = meshes.add(mesh);
                    slots.insert(
                        slot.data().name().to_owned(),
                        parent
                            .spawn_bundle((
                                Mesh2dHandle(mesh.clone()),
                                Transform::from_xyz(demo.position.x, demo.position.y, 0.),
                                GlobalTransform::default(),
                                Visibility::default(),
                                ComputedVisibility::default(),
                                materials.add(ColorMaterial {
                                    color: Color::NONE,
                                    texture: None,
                                }),
                            ))
                            .id(),
                    );
                }
            })
            .insert(Spine { controller });
        for mut note_text in note_query.iter_mut() {
            note_text.sections[0].value = demo.note.clone();
        }
    }
}

#[derive(Default)]
struct DemoNextLocal {
    current_index: usize,
}

fn demo_next(
    mut ev_demo_load: EventWriter<DemoLoad>,
    mut local: Local<DemoNextLocal>,
    keys: Res<Input<KeyCode>>,
    demos: Res<Demos>,
) {
    if keys.just_pressed(KeyCode::Space) {
        local.current_index = (local.current_index + 1) % demos.0.len();
        ev_demo_load.send(DemoLoad(local.current_index));
    }
}

fn spine_update(
    mut spine_query: Query<(&mut Spine, &Children)>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Handle<ColorMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    persistent_image_handles: Res<PersistentImageHandles>,
) {
    let mut image_handles = persistent_image_handles.handles.lock().unwrap();
    let mut image_remember = persistent_image_handles.remember.lock().unwrap();
    let mut image_forget = persistent_image_handles.forget.lock().unwrap();
    while let Some(image) = image_remember.pop() {
        image_handles.push((image.clone(), asset_server.load(&image)));
    }
    while let Some(image) = image_forget.pop() {
        if let Some(index) = image_handles.iter().position(|i| i.0 == image) {
            image_handles.remove(index);
        }
    }
    for (mut spine, spine_children) in spine_query.iter_mut() {
        let Spine { controller, .. } = spine.as_mut();
        controller.update(time.delta_seconds());
        let mut renderables = controller.renderables();
        for (renderable_index, child) in spine_children.iter().enumerate() {
            if let Ok((mesh_handle, color_material_handle)) = colored_mesh2d.get(*child) {
                if let Some(renderable) = renderables.get_mut(renderable_index) {
                    let mut normals = vec![];
                    for _ in 0..renderable.vertices.len() {
                        normals.push([0., 0., 0.]);
                    }
                    let mesh = meshes.get_mut(&mesh_handle.0).unwrap();
                    mesh.set_indices(Some(Indices::U16(take(&mut renderable.indices))));
                    mesh.insert_attribute(
                        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x2),
                        take(&mut renderable.vertices),
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, take(&mut renderable.uvs));
                    if let Some(color_material) = color_materials.get_mut(color_material_handle) {
                        color_material.color.set_r(renderable.color.r);
                        color_material.color.set_g(renderable.color.g);
                        color_material.color.set_b(renderable.color.b);
                        color_material.color.set_a(renderable.color.a);
                        let texture_path = if let Some(attachment_render_object) =
                            renderable.attachment_renderer_object
                        {
                            let spine_texture =
                                unsafe { &mut *(attachment_render_object as *mut SpineTexture) };
                            Some(spine_texture.path.clone())
                        } else {
                            None
                        };
                        color_material.texture =
                            texture_path.map(|p| asset_server.load(p.as_str()));
                    }
                } else {
                    if let Some(color_material) = color_materials.get_mut(color_material_handle) {
                        color_material.color = Color::NONE;
                    }
                }
            }
        }
    }
}

fn load_skeleton(atlas: &Vec<u8>, json: &Vec<u8>, dir: &str) -> Result<SkeletonController, Error> {
    let atlas = Arc::new(Atlas::new(atlas, dir)?);
    let skeleton_json = SkeletonJson::new(atlas.clone());
    let skeleton_data = Arc::new(skeleton_json.read_skeleton_data(json)?);
    let animation_state_data = Arc::new(AnimationStateData::new(skeleton_data.clone()));
    Ok(
        SkeletonController::new(skeleton_data, animation_state_data).with_settings(
            SkeletonControllerSettings::new().with_cull_direction(CullDirection::CounterClockwise),
        ),
    )
}

#[cfg(feature = "egui_debugger")]
fn spine_debugger(mut egui_context: ResMut<EguiContext>, mut spine_query: Query<&mut Spine>) {
    for mut spine in spine_query.iter_mut() {
        let Spine { controller, .. } = spine.as_mut();
        let SkeletonController {
            skeleton,
            animation_state,
            ..
        } = controller;
        egui_spine_debugger(egui_context.ctx_mut(), "Spine", skeleton, animation_state);
    }
}
