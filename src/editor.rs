use bevy::camera::Hdr;
use bevy::camera::visibility::RenderLayers;
use bevy::{camera::Viewport, prelude::*};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiGlobalSettings};
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::LayerId;
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector};
use bevy_window::PrimaryWindow;
use egui_dock::egui::UiBuilder;
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::player::PlayerVar;
use crate::player_ui::UICamera;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(EditorState::new())
            .add_systems(Startup, spawn_ui_cam)
            .add_systems(EguiPrimaryContextPass, show_ui_system)
            .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
    }
}

//enregistrer le type de chaque fenetre
#[derive(Debug)]
enum WindowType {
    GameView,
    Inspector,
    PlayerResourceInspector,
}

//global states de l'editor
#[derive(Resource)]
pub struct EditorState {
    state: DockState<WindowType>,
    viewport_rect: egui_dock::egui::Rect,
    pub pointer_in_viewport: bool,
}

impl EditorState {
    pub fn new() -> Self {
        //decoupage intial des tabs
        let mut state = DockState::new(vec![WindowType::GameView]);
        let tree = state.main_surface_mut();
        let [_game, inspector] =
            tree.split_left(NodeIndex::root(), 0.2, vec![WindowType::Inspector]);
        let [_inspector, _resource_inspector] =
            tree.split_below(inspector, 0.8, vec![WindowType::PlayerResourceInspector]);

        Self {
            state,
            viewport_rect: egui_dock::egui::Rect::from_min_size(
                egui_dock::egui::Pos2::ZERO,
                egui_dock::egui::vec2(1.0, 1.0),
            ),
            pointer_in_viewport: false,
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            pointer_in_viewport: &mut self.pointer_in_viewport,
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
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = WindowType;

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        match window {
            WindowType::GameView => *self.viewport_rect = ui.clip_rect(),
            WindowType::Inspector => {
                egui::ScrollArea::both().show(ui, |ui| {
                    bevy_inspector::ui_for_world(self.world, ui);
                });
            }
            WindowType::PlayerResourceInspector => {
                egui::ScrollArea::both().show(ui, |ui| {
                    bevy_inspector::ui_for_resource::<PlayerVar>(self.world, ui);
                });
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

#[derive(Component)]
pub struct GameViewCam;

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
