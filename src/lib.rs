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
    player_numbers: HashSet<usize>,
}

impl Scoreboard {
    pub fn add_player(
        &mut self,
        player_number: usize,
        player_name: impl Into<String>,
        player_color: Color,
    ) {
        if self.player_numbers.contains(&player_number) {
            error!(
                "Player number {} already exists - NOT ADDING TO THE SCOREBOARD",
                player_number
            );
            return;
        }
        self.player_numbers.insert(player_number);
        let player_name: String = player_name.into();
        self.entries.push(ScoreboardEntry {
            score: 0,
            player_number,
            player_name,
            player_color,
        });
        self.sort();
    }

    pub fn increment(&mut self, player_number: usize, amount: i32) {
        // Find & alter the existing entry (if it exists)
        for score_entry in &mut self.entries {
            if score_entry.player_number == player_number {
                score_entry.score += amount;
                break;
            }
        }
        self.sort();
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
                    format!("{} - {}\n", score_entry.score, score_entry.player_name),
                    score_entry.player_color,
                )
            })
            .collect()
    }
}

struct ScoreboardEntry {
    score: i32,
    player_number: usize,
    player_name: String,
    player_color: Color,
}

#[derive(Component)]
struct ScoreboardText;

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
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
        })
        .with_children(|builder| {
            builder.spawn((
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
