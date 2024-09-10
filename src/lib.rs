use bevy::{prelude::*, utils::HashSet,
    asset::embedded_asset,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
pub mod prelude {
    pub use crate::{Scoreboard, ScoreboardPlugin};
}

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scoreboard>()
            .init_state::<WinState>()
            .add_systems(Startup, setup)
            .add_systems(Update, update)    
            // So that any entities created by these systems are automatically despawned after leaving
            // the WinState.
            .enable_state_scoped_entities::<WinState>()
            .add_systems(PreUpdate, detect_win.run_if(in_state(WinState::Detecting)))
            .add_systems(Update, display_win.run_if(in_state(WinState::Displaying)));

        let omit_prefix = "src";
        embedded_asset!(app, omit_prefix, "winner_assets/KaushanScript-Regular.ttf");
    }
}

#[derive(Resource, Default)]
pub struct Scoreboard {
    entries: Vec<ScoreboardEntry>,
    player_ids: HashSet<usize>,
    winner_entity: Option<Entity>,
    winner_timer: Timer,
}



impl Scoreboard {
    pub fn add_player(&mut self, id: usize, name: impl Into<String>, color: Color) {
        if self.player_ids.contains(&id) {
            error!("Player id {} already exists - NOT ADDING TO SCOREBOARD", id);
            return;
        }
        self.player_ids.insert(id);
        let player_name: String = name.into();
        self.entries.push(ScoreboardEntry {
            score: 0,
            id,
            name: player_name,
            color,
        });
        self.sort();
    }

    pub fn show_winner_screen(&mut self, winner_entity: Entity) {
        self.winner_entity = Some(winner_entity);
    }

    pub fn remove_player(&mut self, id: usize) {
        self.player_ids.remove(&id);
        self.entries.retain(|entry| entry.id != id);
    }

    pub fn increment(&mut self, player_id: usize, amount: i32) {
        // Find & alter the existing entry (if it exists)
        for score_entry in &mut self.entries {
            if score_entry.id == player_id {
                score_entry.score += amount;
                break;
            }
        }
        self.sort();
    }

    pub fn get_score(&self, player_id: usize) -> i32 {
        for score_entry in &self.entries {
            if score_entry.id == player_id {
                return score_entry.score;
            }
        }
        0
    }

    pub fn high_score(&self) -> i32 {
        match self.entries.iter().max_by(|x, y| x.score.cmp(&y.score)) {
            Some(entry) => entry.score,
            None => 0,
        }
    }

    pub fn set_score(&mut self, player_id: usize, score: i32) {
        for score_entry in &mut self.entries {
            if score_entry.id == player_id {
                score_entry.score = score;
            }
        }
    }

    pub fn reset_scores(&mut self) {
        for score_entry in &mut self.entries {
            score_entry.score = 0;
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.player_ids.clear();
    }

    // Sort the score entries by score
    fn sort(&mut self) {
        self.entries
            .sort_by(|entry1, entry2| entry2.score.cmp(&entry1.score));
    }

    fn get_lines(&self) -> Vec<(String, Color)> {
        self.entries
            .iter()
            .map(|score_entry| {
                (
                    format!("{} - {}\n", score_entry.score, score_entry.name),
                    score_entry.color,
                )
            })
            .collect()
    }
}

struct ScoreboardEntry {
    score: i32,
    id: usize,
    name: String,
    color: Color,
}

#[derive(Component)]
struct ScoreboardText;

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Scoreboard Background"),
            NodeBundle {
                style: Style {
                    width: Val::Px(230.),
                    height: Val::Px(200.),
                    margin: UiRect::all(Val::Px(20.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((
                Name::new("Scoreboard Text"),
                TextBundle::from_section(
                    "Scores\n",
                    TextStyle {
                        font_size: 48.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Left)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(15.0),
                    left: Val::Px(15.0),
                    ..default()
                }),
                ScoreboardText,
            ));
        });
}

fn update(
    mut scoreboard_text_query: Query<&mut Text, With<ScoreboardText>>,
    scoreboard: Res<Scoreboard>,
) {
    let Ok(mut scoreboard_text) = scoreboard_text_query.get_single_mut() else {
        return;
    };
    for (mut index, (value, color)) in scoreboard.get_lines().into_iter().enumerate() {
        index += 1;
        if scoreboard_text.sections.len() <= index {
            scoreboard_text.sections.push(TextSection {
                value,
                style: TextStyle {
                    font_size: 32.0,
                    color,
                    ..default()
                },
            });
        } else {
            if scoreboard_text.sections[index].value != value {
                scoreboard_text.sections[index].value = value;
            }
            if scoreboard_text.sections[index].style.color != color {
                scoreboard_text.sections[index].style.color = color;
            }
        }
    }
}





#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum WinState {
    #[default]
    Detecting,
    Displaying,
}


#[allow(clippy::too_many_arguments)]
fn detect_win(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut next_state: ResMut<NextState<WinState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    texture_qry: Query<(&Sprite, &Handle<Image>)>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(winner_entity) = scoreboard.winner_entity else {
        return;
    };
    // Someone just won, get stuff set up!
    scoreboard.winner_timer = Timer::from_seconds(2.5, TimerMode::Repeating);
    next_state.set(WinState::Displaying);
    let font =
        asset_server.load("embedded://bevy_scoreboard/winner_assets/KaushanScript-Regular.ttf");
    commands.spawn((
        StateScoped(WinState::Displaying),
        Name::new("Winner Text"),
        Text2dBundle {
            text: Text::from_section(
                "Winner!!!",
                TextStyle {
                    font_size: 196.0,
                    font,
                    ..Default::default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 0.0, 999.0),
            ..Default::default()
        },
    ));
    virtual_time.pause();
    let (sprite, texture) = texture_qry.get(winner_entity).unwrap();
    let mut transform = Transform::from_xyz(0.0, 150.0, 999.0);
    transform.scale = Vec3::splat(2.5);
    commands.spawn((
        StateScoped(WinState::Displaying),
        Name::new("Winning Player Picture"),
        SpriteBundle {
            sprite: sprite.clone(),
            transform,
            texture: texture.clone(),
            ..Default::default()
        },
    ));

    commands.spawn((
        StateScoped(WinState::Displaying),
        Name::new("Winning Player Darkened Background"),
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(1280.0, 768.0))),
            material: materials.add(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            transform: Transform::from_xyz(0.0, 0.0, 998.0),
            ..Default::default()
        },
    ));
}

fn display_win(
    mut next_state: ResMut<NextState<WinState>>,
    time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    if scoreboard.winner_timer.tick(time.delta()).just_finished() {
        next_state.set(WinState::Detecting);
        scoreboard.winner_entity = None;
        scoreboard.reset_scores();
        virtual_time.unpause();
    }
}
