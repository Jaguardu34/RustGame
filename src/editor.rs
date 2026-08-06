use std::fmt::format;

use bevy::camera::Hdr;
use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::relationship::RelationshipSourceCollection;
use bevy::{camera::Viewport, prelude::*};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiGlobalSettings};
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::LayerId;
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector};
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::plugin::ReadRapierContext;
use bevy_window::PrimaryWindow;
use egui_dock::egui::UiBuilder;
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::free_camera::FreeCam;
use crate::game_var::GameVar;
use crate::load_chunks::{ChunkMap, MapGen, MapParams, RegenerateMapEvent, clear_chunk_cache};
use crate::player::PlayerVar;
use crate::player_ui::UICamera;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(EditorState::new())
            .init_resource::<SelectedItems>()
            .add_systems(Update, (draw_gizmo, pick_object_in_viewport, pause_app))
            .add_systems(Startup, (spawn_ui_cam, update_gizmo_parameters))
            .add_systems(EguiPrimaryContextPass, show_ui_system)
            .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
    }
}

#[derive(Resource, Default)]
pub struct SelectedItems(pub EntityHashSet);

//enregistrer le type de chaque fenetre
#[derive(Debug)]
enum WindowType {
    GameView,
    Inspector,
    PlayerResourceInspector,
    SelectedEntitieInspector,
    GameResourceInspector,
    GameManager,
    MapManager,
}

//global states de l'editor
#[derive(Resource)]
pub struct EditorState {
    state: DockState<WindowType>,
    viewport_rect: egui_dock::egui::Rect,
    pub pointer_in_viewport: bool,
    pub game_playing: bool,
    pub free_cam: bool,
}

impl EditorState {
    pub fn new() -> Self {
        //decoupage intial des tabs
        let mut state = DockState::new(vec![WindowType::GameView]);
        let tree = state.main_surface_mut();
        let [game, inspector] =
            tree.split_left(NodeIndex::root(), 0.2, vec![WindowType::Inspector]);
        let [_inspector, _resource_inspector] =
            tree.split_below(inspector, 0.8, vec![WindowType::PlayerResourceInspector]);

        let [_game, selected_entity_inspector] =
            tree.split_right(game, 0.8, vec![WindowType::SelectedEntitieInspector]);
        let [_selected_entity_inspector, game_resource_inspector] = tree.split_below(
            selected_entity_inspector,
            0.5,
            vec![WindowType::GameResourceInspector],
        );

        let [_game_resource_inspector, game_manager] =
            tree.split_below(game_resource_inspector, 0.5, vec![WindowType::GameManager]);

        let [_game_manager, _map_manager] =
            tree.split_below(game_manager, 0.5, vec![WindowType::MapManager]);

        Self {
            state,
            viewport_rect: egui_dock::egui::Rect::from_min_size(
                egui_dock::egui::Pos2::ZERO,
                egui_dock::egui::vec2(1.0, 1.0),
            ),
            pointer_in_viewport: false,
            game_playing: false,
            free_cam: true,
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            pointer_in_viewport: &mut self.pointer_in_viewport,
            game_playing: &mut self.game_playing,
            free_cam: &mut self.free_cam,
        };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(&ui.global_style()))
            .show_inside(ui, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    viewport_rect: &'a mut egui::Rect,
    pointer_in_viewport: &'a mut bool,
    game_playing: &'a mut bool,
    free_cam: &'a mut bool,
}

//tab egui
impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = WindowType;

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        match window {
            WindowType::GameView => {
                *self.viewport_rect = ui.clip_rect();
                ui.allocate_rect(ui.clip_rect(), egui::Sense::hover());
            }
            WindowType::Inspector => {
                bevy_inspector::ui_for_world(self.world, ui);
            }
            WindowType::PlayerResourceInspector => {
                bevy_inspector::ui_for_resource::<PlayerVar>(self.world, ui);
            }
            WindowType::SelectedEntitieInspector => {
                let selected: Vec<Entity> = self
                    .world
                    .get_resource::<SelectedItems>()
                    .map(|s| s.0.iter().copied().collect())
                    .unwrap_or_default();

                if selected.len() == 1 {
                    egui::ScrollArea::both().show(ui, |ui| {
                        bevy_inspector::ui_for_entity(self.world, selected[0], ui);
                    });
                } else if selected.is_empty() {
                    ui.label("Aucune entité sélectionnée");
                } else {
                    ui.label("Plusieurs entités sélectionnées");
                }
            }
            WindowType::GameResourceInspector => {
                bevy_inspector::ui_for_resource::<GameVar>(self.world, ui);
            }
            WindowType::GameManager => {
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(24.0, egui::epaint::FontFamily::Proportional),
                );
                let button_label = if !*self.game_playing {
                    String::from("Play")
                } else {
                    String::from("Pause")
                };
                let game_paused_clone = *self.game_playing;
                if ui.button(button_label).clicked() {
                    *self.game_playing = !*self.game_playing;
                };
                if game_paused_clone != *self.game_playing && *self.game_playing {
                    self.world
                        .get_resource_mut::<GameVar>()
                        .unwrap()
                        .mouse_grabbed = true;
                }

                let diagnostics = self.world.get_resource::<DiagnosticsStore>().unwrap();

                if let Some(value) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    ui.label(String::from(format!("{}", value)));
                }
            }
            WindowType::MapManager => {
                let mut map_params = self.world.get_resource_mut::<MapParams>().unwrap();

                let mut changed = false;

                ui.label("Seed");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.seed).speed(1))
                    .changed();
                ui.label("Scale");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.scale).speed(0.001))
                    .changed();
                ui.label("Lacunarity");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.lacunarity).speed(0.001))
                    .changed();
                ui.label("Persistance");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.persistance).speed(0.001))
                    .changed();
                ui.label("Octaves");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.octaves).speed(1))
                    .changed();
                ui.label("Height Mult");
                changed |= ui
                    .add(egui::DragValue::new(&mut map_params.height_mult).speed(1))
                    .changed();

                if changed {
                    let (seed, scale, lacunarity, persistance, octaves) = (
                        map_params.seed,
                        map_params.scale,
                        map_params.lacunarity,
                        map_params.persistance,
                        map_params.octaves,
                    );

                    drop(map_params);

                    clear_chunk_cache(self.world);

                    let mut map_gen = self.world.get_resource_mut::<MapGen>().unwrap();
                    map_gen
                        .0
                        .regenerate(seed, scale, lacunarity, persistance, octaves);
                    drop(map_gen);

                    self.world.write_message(RegenerateMapEvent);
                }
            }
        }

        *self.pointer_in_viewport = ui
            .ctx()
            .rect_contains_pointer(LayerId::background(), self.viewport_rect.shrink(16.));
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, WindowType::GameView)
    }
}

//comp to render cam with this comp inside the viewport tab
#[derive(Component)]
pub struct GameViewCam;

//spawn the 2d cam that render ui
fn spawn_ui_cam(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    egui_global_settings.auto_create_primary_context = false;
    commands.spawn((
        Camera2d,
        Name::new("UiCamera"),
        Camera {
            order: 10,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Msaa::Off,
        RenderLayers::none(),
        IsDefaultUiCamera,
        PrimaryEguiContext,
        Hdr,
        UICamera,
    ));
}

//show the egui ui
fn show_ui_system(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
    else {
        return;
    };
    let ctx = egui_context.get_mut();
    let mut ui = egui::Ui::new(
        ctx.clone(),
        "viewport".into(),
        UiBuilder::new()
            .layer_id(LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    world.resource_scope::<EditorState, _>(|world, mut ui_state| {
        ui_state.ui(&mut ui, world);
    });
}

//update the cameras to render on the viewport tab
fn set_camera_viewport(
    game_view_rect: Res<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam_query: Query<&mut Camera, With<GameViewCam>>,
) {
    cam_query.iter_mut().for_each(|mut cam| {
        let scale_factor = window.scale_factor();
        let pos = game_view_rect.viewport_rect.left_top().to_vec2() * scale_factor;
        let size = game_view_rect.viewport_rect.size() * scale_factor;

        let physical_position = UVec2::new(pos.x.max(0.0) as u32, pos.y.max(0.0) as u32);
        let physical_size = UVec2::new(size.x.max(1.0) as u32, size.y.max(1.0) as u32);

        let bottom_right = physical_position.saturating_add(physical_size);
        let window_size = window.physical_size();
        if bottom_right.x <= window_size.x && bottom_right.y <= window_size.y {
            cam.viewport = Some(Viewport {
                physical_position,
                physical_size,
                depth: 0.0..1.0,
            });
        }
    });
}

//render gizmos on top of everything
fn update_gizmo_parameters(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        config.depth_bias = if config.depth_bias == 0. { -1. } else { 0. };
    }
}

//draw gizmo like in blender
fn draw_gizmo(
    objects_query: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
    selected_items: Res<SelectedItems>,
    editor_state: Res<EditorState>,
) {
    if !editor_state.free_cam {
        return;
    }
    for entity in selected_items.0.iter() {
        if let Ok(global_transform) = objects_query.get(*entity) {
            gizmos.arrow(
                global_transform.translation(),
                Vec3 {
                    x: global_transform.translation().x + 1.0,
                    y: global_transform.translation().y,
                    z: global_transform.translation().z,
                },
                Color::srgb(1.0, 0.0, 0.0),
            );
            gizmos.arrow(
                global_transform.translation(),
                Vec3 {
                    x: global_transform.translation().x,
                    y: global_transform.translation().y + 1.0,
                    z: global_transform.translation().z,
                },
                Color::srgb(0.0, 0.0, 1.0),
            );
            gizmos.arrow(
                global_transform.translation(),
                Vec3 {
                    x: global_transform.translation().x,
                    y: global_transform.translation().y,
                    z: global_transform.translation().z + 1.0,
                },
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
    }
}

//equivalent to meshpickingplugin bc the plugin cant work bc of the egui context using the click event of the mouse
fn pick_object_in_viewport(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    editor_state: Res<EditorState>,
    cam_query: Query<(&Camera, &GlobalTransform), With<FreeCam>>,
    rapier_context: ReadRapierContext,
    mut selected_items: ResMut<SelectedItems>,

    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !mouse.just_pressed(MouseButton::Left)
        || !editor_state.pointer_in_viewport
        || !editor_state.free_cam
    {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = cam_query.single() else {
        println!("pas de freecam trouvée");
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        println!("viewport to world pas marché");
        return;
    };

    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };

    if let Some((entity, _toi)) = rapier_context.cast_ray(
        ray.origin,
        *ray.direction,
        f32::MAX,
        true,
        QueryFilter::default(),
    ) {
        if keyboard.pressed(KeyCode::ControlLeft) {
            selected_items.0.add(entity);
        } else {
            selected_items.0.clear();
            selected_items.0.add(entity);
        }
    } else {
        selected_items.0.clear();
    }
}

fn pause_app(mut virtual_time: ResMut<Time<Virtual>>, mut editor_state: ResMut<EditorState>) {
    if editor_state.game_playing {
        editor_state.free_cam = false;
        virtual_time.unpause();
    } else {
        editor_state.free_cam = true;
        virtual_time.pause();
    }
}
