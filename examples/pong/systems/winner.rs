use {Ball, ScoreBoard, Serve, Side};
use std::time::Duration;
use amethyst::assets::AssetStorage;
use amethyst::audio::Source;
use amethyst::audio::output::Output;
use amethyst::core::transform::Transform;
use amethyst::ecs::prelude::{Entity, Join, Read, ReadExpect, System, Write, WriteStorage};
use amethyst::ui::UiText;
use audio::Sounds;

/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        Write<'s, Serve>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        ReadExpect<'s, ScoreText>,
        Read<'s, Option<Output>>,
    );

    fn run(
        &mut self,
        (
            mut balls,
            mut transforms,
            mut text,
            mut score_board,
            mut serve,
            storage,
            sounds,
            score_text,
            audio_output,
        ): Self::SystemData,
    ) {
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            use {ARENA_WIDTH, BALL_VELOCITY_X, BALL_VELOCITY_Y};
            if serve.stopwatch.elapsed() > Duration::new(2, 0) {
                ball.velocity[0] = BALL_VELOCITY_X;
                ball.velocity[1] = BALL_VELOCITY_Y;

                if serve.serve_to == Side::Left{
                    ball.velocity[0] = -ball.velocity[0];
                }
                serve.stopwatch.reset();
            }

            let ball_x = transform.translation[0];

            let hit_side = if ball_x <= ball.radius {
                // Right player scored on the left side.
                score_board.score_right += 1;
                if let Some(text) = text.get_mut(score_text.p2_score) {
                    text.text = score_board.score_right.to_string();
                }
                Some(Side::Left)
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                // Left player scored on the right side.
                score_board.score_left += 1;
                if let Some(text) = text.get_mut(score_text.p1_score) {
                    text.text = score_board.score_left.to_string();
                }
                Some(Side::Right)
            } else {
                None
            };

            if let Some(hit_side) = hit_side {
                // Reset the ball.
                ball.velocity[0] = 0.0;
                ball.velocity[1] = 0.0;
                transform.translation[0] = ARENA_WIDTH / 2.0;
                transform.translation[1] = 2.0 * ball.radius;

                serve.stopwatch.restart();
                serve.serve_to = hit_side;

                // Print the score board.
                println!(
                    "Score: | {:^3} | {:^3} |",
                    score_board.score_left, score_board.score_right
                );

                // Play audio.
                if let Some(ref output) = *audio_output {
                    if let Some(sound) = storage.get(&sounds.score_sfx) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        }
    }
}

/// Stores the entities that are displaying the player score with UiText.
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}
