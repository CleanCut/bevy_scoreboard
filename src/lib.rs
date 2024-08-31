use bevy::{prelude::*, utils::HashSet};

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scoreboard>()
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

#[derive(Resource, Default)]
pub struct Scoreboard {
    entries: Vec<ScoreboardEntry>,
    player_ids: HashSet<usize>,
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
