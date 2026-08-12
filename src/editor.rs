use avian3d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::camera::Hdr;
use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use bevy::post_process::bloom::Bloom;
use bevy::{camera::Viewport, prelude::*};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiGlobalSettings};
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::LayerId;
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector};
use bevy_window::PrimaryWindow;
use egui_dock::egui::UiBuilder;
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::free_camera::FreeCam;
use crate::game_var::GameVar;
use crate::player::PlayerVar;
use crate::player_ui::UICamera;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(EditorState::new())
            .add_systems(Update, check_editor_state_change)
            .add_systems(
                Update,
                (update_transform_gizmo_settings, pause_app).run_if(check_if_in_editor),
            )
            //custom gizmo
            .add_systems(
                Update,
                (draw_gizmo, pick_object_in_viewport)
                    .run_if(check_if_in_editor)
                    .after(set_camera_viewport),
            )
            .add_systems(Startup, update_gizmo_parameters)
            .add_systems(Startup, spawn_ui_cam)
            .add_systems(
                EguiPrimaryContextPass,
                show_ui_system.run_if(check_if_in_editor),
            )
            .add_systems(
                PostUpdate,
                set_camera_viewport
                    .after(show_ui_system)
                    .run_if(check_if_in_editor),
            );
    }
}

//enregistrer le type de chaque fenetre
#[derive(Debug)]
enum WindowType {
    GameView,
    Inspector,
    PlayerResourceInspector,
    SelectedEntitieInspector,
    GameResourceInspector,
    GameManager,
}

//global states de l'editor
#[derive(Resource)]
pub struct EditorState {
    state: DockState<WindowType>,
    viewport_rect: egui_dock::egui::Rect,
    pub pointer_in_viewport: bool,
    pub game_playing: bool,
    pub pause_time: bool,
}

impl EditorState {
    pub fn new() -> Self {
        //decoupage intial des tabs
        let mut state = DockState::new(vec![WindowType::GameView]);
        let tree = state.main_surface_mut();
        let [game, inspector] =
            tree.split_left(NodeIndex::root(), 0.2, vec![WindowType::Inspector]);
        let [_inspector, _resource_inspector] =
            tree.split_below(inspector, 0.5, vec![WindowType::PlayerResourceInspector]);

        let [_game, selected_entity_inspector] =
            tree.split_right(game, 0.8, vec![WindowType::SelectedEntitieInspector]);
        let [_selected_entity_inspector, game_resource_inspector] = tree.split_below(
            selected_entity_inspector,
            0.5,
            vec![WindowType::GameResourceInspector],
        );

        let [_game_resource_inspector, _game_manager] =
            tree.split_below(game_resource_inspector, 0.5, vec![WindowType::GameManager]);

        Self {
            state,
            viewport_rect: egui_dock::egui::Rect::from_min_size(
                egui_dock::egui::Pos2::ZERO,
                egui_dock::egui::vec2(1.0, 1.0),
            ),
            pointer_in_viewport: false,
            game_playing: false,
            pause_time: false,
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            pointer_in_viewport: &mut self.pointer_in_viewport,
            game_playing: &mut self.game_playing,
            pause_time: &mut self.pause_time,
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
    pause_time: &'a mut bool,
    //free_cam: &'a mut bool,
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
                let mut query = self
                    .world
                    .query_filtered::<Entity, With<TransformGizmoFocus>>();

                let Ok(entity) = query.single(self.world) else {
                    ui.label("Aucune entité sélectionnée");
                    return;
                };

                egui::ScrollArea::both().show(ui, |ui| {
                    bevy_inspector::ui_for_entity(self.world, entity, ui);
                });
            }
            WindowType::GameResourceInspector => {
                bevy_inspector::ui_for_resource::<GameVar>(self.world, ui);
            }
            WindowType::GameManager => {
                ui.label("Game");
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
                    ui.label(String::from(format!("{:.1} FPS", value)));
                }

                if ui.button("Leave Editor").clicked() {
                    self.world.get_resource_mut::<GameVar>().unwrap().in_editor = false;
                };

                ui.label("Physics Time");
                let button_label = if *self.pause_time {
                    String::from("Play")
                } else {
                    String::from("Pause")
                };

                if ui.button(button_label).clicked() {
                    *self.pause_time = !*self.pause_time;
                };
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
        Bloom::NATURAL,
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
    selected_items: Query<Entity, With<TransformGizmoFocus>>,
    editor_state: Res<EditorState>,
) {
    if editor_state.game_playing {
        return;
    }
    for entity in selected_items.iter() {
        if let Ok(global_transform) = objects_query.get(entity) {
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
    existing: Query<Entity, With<TransformGizmoFocus>>,
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut last_selected: Local<Option<Entity>>,
) {
    if !mouse.just_pressed(MouseButton::Left)
        || !editor_state.pointer_in_viewport
        || editor_state.game_playing
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

    if let Some(ray_hit_data) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        f32::MAX,
        true,
        &SpatialQueryFilter::default(),
    ) {
        if *last_selected != Some(ray_hit_data.entity) {
            for e in &existing {
                commands.entity(e).remove::<TransformGizmoFocus>();
            }

            commands
                .entity(ray_hit_data.entity)
                .insert(TransformGizmoFocus);
            *last_selected = Some(ray_hit_data.entity)
        }
    } else {
        for e in &existing {
            commands.entity(e).remove::<TransformGizmoFocus>();
        }
        *last_selected = None;
    }
}

fn pause_app(mut virtual_time: ResMut<Time<Virtual>>, editor_state: ResMut<EditorState>) {
    if editor_state.pause_time {
        virtual_time.pause();
    } else {
        virtual_time.unpause();
    }
}

fn check_if_in_editor(game_var: Res<GameVar>) -> bool {
    game_var.in_editor
}

fn check_editor_state_change(
    mut virtual_time: ResMut<Time<Virtual>>,
    mut editor_state: ResMut<EditorState>,
    mut cam_query: Query<&mut Camera, With<GameViewCam>>,
    game_var: Res<GameVar>,
    mut last_state: Local<bool>,
) {
    if !game_var.in_editor {
        editor_state.game_playing = true;
        virtual_time.unpause();
        editor_state.game_playing = true;
        editor_state.pointer_in_viewport = true;
        editor_state.pause_time = false;

        //cam in fullscreen
        cam_query.iter_mut().for_each(|mut cam| {
            cam.viewport = None;
        });
    }
    if *last_state != game_var.in_editor && game_var.in_editor {
        editor_state.game_playing = false;
    }
    *last_state = game_var.in_editor;
}

fn update_transform_gizmo_settings(mut settings: ResMut<TransformGizmoSettings>) {
    settings.mode = TransformGizmoMode::Translate;
}
