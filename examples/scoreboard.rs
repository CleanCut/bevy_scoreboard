use bevy::prelude::*;
use bevy_scoreboard::{Scoreboard, ScoreboardPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ScoreboardPlugin))
        .insert_resource(ScoreTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Startup, startup)
        .add_systems(Update, update_scores)
        .run();
}

#[derive(Resource)]
struct ScoreTimer(Timer);

fn update_scores(
    mut score_timer: ResMut<ScoreTimer>,
    mut scoreboard: ResMut<Scoreboard>,
    time: Res<Time>,
) {
    score_timer.0.tick(time.delta());
    if score_timer.0.just_finished() {
        scoreboard.increment(1, 1);
        scoreboard.increment(2, 2);
        scoreboard.increment(3, 4);
        scoreboard.increment(4, 8);
    }
}

fn startup(mut commands: Commands, mut scoreboard: ResMut<Scoreboard>) {
    scoreboard.add_player(1, "Ferris", Color::srgb(1., 0., 0.));
    scoreboard.add_player(2, "Paul", Color::srgb(0., 1., 0.));
    scoreboard.add_player(3, "George", Color::srgb(0., 0., 1.));
    scoreboard.add_player(4, "Ringo", Color::srgb(1., 1., 0.));
    commands.spawn(Camera2dBundle::default());
}
