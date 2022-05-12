use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

use crate::state::{BufferedState, GameState, OpeningGame};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::MainMenu).with_system(init_main_menu))
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(init_menu))
            .add_system_set(SystemSet::on_update(GameState::Menu).with_system(button_action))
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(term_menu));
    }
}

#[derive(AssetCollection)]
pub struct Fonts {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    font: Handle<Font>,
}

#[derive(Clone, Component)]
enum Action {
    Menu(MenuBuilder),
    Back,
    Rebuild,
    ImportGame,
    CreateWorld(PathBuf),
    Play(PathBuf),
    Set(Vec<Action>),
}

#[derive(Clone)]
enum MenuTitleSize {
    MainTitle,
    Normal,
}

#[derive(Clone)]
struct MenuTitle {
    text: String,
    size: MenuTitleSize,
}

#[derive(Clone)]
struct MenuButton {
    text: String,
    action: Action,
}

#[derive(Clone, Deref)]
struct MenuButtonRow(Vec<MenuButton>);

#[derive(Clone)]
enum AssetButtonAction {
    CreateWorld,
    Play,
}

impl AssetButtonAction {
    fn assets_path(&self) -> &'static Path {
        match self {
            AssetButtonAction::CreateWorld => Path::new("games"),
            AssetButtonAction::Play => Path::new("worlds"),
        }
    }

    fn action(&self, path: &Path) -> Action {
        let path = self.assets_path().join(path);
        match self {
            AssetButtonAction::CreateWorld => Action::CreateWorld(path),
            AssetButtonAction::Play => Action::Play(path),
        }
    }
}

#[derive(Clone)]
enum MenuButtonsBuilder {
    Row(MenuButtonRow),
    PerAsset { action: AssetButtonAction },
}

impl MenuButtonsBuilder {
    fn build(&self, asset_server: &AssetServer) -> Vec<MenuButtonRow> {
        match self {
            MenuButtonsBuilder::Row(row) => vec![row.clone()],
            MenuButtonsBuilder::PerAsset { action } => asset_server
                .asset_io()
                .read_directory(action.assets_path())
                .unwrap_or_else(|_| Box::new(Vec::default().into_iter()))
                .map(|path| {
                    MenuButtonRow(vec![MenuButton {
                        text: path.file_name().unwrap().to_string_lossy().to_string(),
                        action: action.action(&path),
                    }])
                })
                .collect(),
        }
    }
}

#[derive(Clone)]
struct Menu {
    title: MenuTitle,
    buttons: Vec<MenuButtonRow>,
}

const MENU_ITEM_MARGIN: Rect<Val> = Rect {
    left: Val::Percent(0.),
    right: Val::Percent(0.),
    top: Val::Px(10.),
    bottom: Val::Px(10.),
};
const MENU_TITLE_SIZE: f32 = 100.;
const MENU_HEADING_SIZE: f32 = 65.;
const MENU_TITLE_COLOR: Color = Color::WHITE;
const BUTTON_SIZE: Size<Val> = Size {
    width: Val::Percent(50.),
    height: Val::Px(50.),
};
const BUTTON_COLOR: Color = Color::WHITE;
const BUTTON_HOVER_COLOR: Color = Color::rgb(0.75, 0.75, 0.75);
const BUTTON_PRESS_COLOR: Color = Color::GRAY;
const BUTTON_TEXT_SIZE: f32 = 50.;
const BUTTON_TEXT_COLOR: Color = Color::BLACK;

impl Menu {
    fn spawn(&self, commands: &mut Commands, fonts: &Fonts) -> Entity {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    ..default()
                },
                color: Color::NONE.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    style: Style {
                        margin: MENU_ITEM_MARGIN.clone(),
                        ..default()
                    },
                    text: Text::with_section(
                        self.title.text.clone(),
                        TextStyle {
                            font: fonts.font.clone(),
                            font_size: match self.title.size {
                                MenuTitleSize::MainTitle => MENU_TITLE_SIZE,
                                MenuTitleSize::Normal => MENU_HEADING_SIZE,
                            },
                            color: MENU_TITLE_COLOR,
                        },
                        default(),
                    ),
                    ..default()
                });

                for row in &self.buttons {
                    // TODO make this display in rows
                    for button in &**row {
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    margin: MENU_ITEM_MARGIN.clone(),
                                    size: BUTTON_SIZE.clone(),
                                    ..default()
                                },
                                color: BUTTON_COLOR.into(),
                                ..default()
                            })
                            .insert(button.action.clone())
                            .with_children(|parent| {
                                parent.spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        button.text.clone(),
                                        TextStyle {
                                            font: fonts.font.clone(),
                                            font_size: BUTTON_TEXT_SIZE,
                                            color: BUTTON_TEXT_COLOR,
                                        },
                                        default(),
                                    ),
                                    ..default()
                                });
                            });
                    }
                }
            })
            .id()
    }
}

#[derive(Clone)]
struct MenuBuilder {
    title: MenuTitle,
    buttons: Vec<MenuButtonsBuilder>,
}

impl MenuBuilder {
    fn build(&self, asset_server: &AssetServer) -> Menu {
        Menu {
            title: self.title.clone(),
            buttons: self
                .buttons
                .iter()
                .flat_map(|buttons| buttons.build(asset_server))
                .collect(),
        }
    }
}

#[derive(Deref, DerefMut)]
struct MenuEs(Vec<Entity>);

#[derive(Deref)]
struct NextMenu(MenuBuilder);

fn init_main_menu(mut commands: Commands, mut state: ResMut<State<GameState>>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(NextMenu(MenuBuilder {
        title: MenuTitle {
            text: "voxmod".to_string(),
            size: MenuTitleSize::MainTitle,
        },
        buttons: vec![
            MenuButtonsBuilder::Row(MenuButtonRow(vec![MenuButton {
                text: "Play".to_string(),
                action: Action::Menu(MenuBuilder {
                    title: MenuTitle {
                        text: "Choose a world".to_string(),
                        size: MenuTitleSize::Normal,
                    },
                    buttons: vec![
                        MenuButtonsBuilder::PerAsset {
                            action: AssetButtonAction::Play,
                        },
                        MenuButtonsBuilder::Row(MenuButtonRow(vec![
                            MenuButton {
                                text: "Back".to_string(),
                                action: Action::Back,
                            },
                            MenuButton {
                                text: "New world".to_string(),
                                action: Action::Menu(MenuBuilder {
                                    title: MenuTitle {
                                        text: "New world".to_string(),
                                        size: MenuTitleSize::Normal,
                                    },
                                    buttons: vec![
                                        MenuButtonsBuilder::PerAsset {
                                            action: AssetButtonAction::CreateWorld,
                                        },
                                        MenuButtonsBuilder::Row(MenuButtonRow(vec![
                                            MenuButton {
                                                text: "Back".to_string(),
                                                action: Action::Back,
                                            },
                                            MenuButton {
                                                text: "Import game".to_string(),
                                                action: Action::Set(vec![
                                                    Action::ImportGame,
                                                    Action::Rebuild,
                                                ]),
                                            },
                                        ])),
                                    ],
                                }),
                            },
                        ])),
                    ],
                }),
            }])),
            MenuButtonsBuilder::Row(MenuButtonRow(vec![MenuButton {
                text: "Edit".to_string(),
                action: Action::Back,
            }])),
            MenuButtonsBuilder::Row(MenuButtonRow(vec![MenuButton {
                text: "Quit".to_string(),
                action: Action::Back,
            }])),
        ],
    }));
    state.push(GameState::Menu).unwrap();
}

fn init_menu(
    mut commands: Commands,
    mut menu_es: Option<ResMut<MenuEs>>,
    mut nodes: Query<&mut Style, With<Node>>,
    fonts: Res<Fonts>,
    next_menu: Res<NextMenu>,
    asset_server: Res<AssetServer>,
) {
    let menu_e = next_menu.build(&asset_server).spawn(&mut commands, &fonts);
    if let Some(menu_es) = &mut menu_es {
        nodes.get_mut(*menu_es.last().unwrap()).unwrap().display = Display::None;
        menu_es.push(menu_e);
    } else {
        commands.insert_resource(MenuEs(vec![menu_e]));
    }

    commands.remove_resource::<NextMenu>();
}

fn button_action(
    mut commands: Commands,
    mut interactions: Query<
        (&Interaction, &mut UiColor, &Action),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, mut color, action) in interactions.iter_mut() {
        *color = match interaction {
            Interaction::Clicked => {
                match action {
                    Action::Menu(menu) => {
                        commands.insert_resource(BufferedState(GameState::Menu));
                        commands.insert_resource(NextMenu(menu.clone()));
                        state.push(GameState::Buffer).unwrap();
                    }
                    Action::Back => state.pop().unwrap(),
                    Action::Play(_) => {
                        commands.insert_resource(OpeningGame);
                        state.replace(GameState::Game).unwrap()
                    }
                    _ => (), // TODO
                }
                BUTTON_PRESS_COLOR
            }
            Interaction::Hovered => BUTTON_HOVER_COLOR,
            Interaction::None => BUTTON_COLOR,
        }
        .into();
    }
}

fn term_menu(
    mut commands: Commands,
    mut nodes: Query<&mut Style, With<Node>>,
    mut menu_es: ResMut<MenuEs>,
) {
    commands.entity(menu_es.pop().unwrap()).despawn_recursive();
    if let Some(menu_e) = menu_es.last() {
        nodes.get_mut(*menu_e).unwrap().display = Display::Flex;
    } else {
        commands.remove_resource::<MenuEs>();
    }
}
