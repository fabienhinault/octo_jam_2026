//! A simplified implementation of the classic game "Breakout".
//!
//! Demonstrates Bevy's stepping capabilities if compiled with the `bevy_debug_stepping` feature.

use bevy::prelude::*;

const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;
const BALL_DIAMETER: f32 = 30.;
const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

mod setup;
mod stepping;
mod wall;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .at(percent(35), percent(50)),
        )
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup::setup)
        // Add our simulation systems to the update schedule
        // which is called once per frame.
        .add_systems(
            Update,
            (
                apply_velocity,
                move_paddle,
                check_for_ball_collisions,
                check_for_brick_collisions,
            )
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Update, update_scoreboard)
        .add_observer(play_collision_sound)
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Event)]
struct BallCollided;

#[derive(Component)]
struct Brick;

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// Default must be implemented to define this as a required component for the Wall component below
#[derive(Component, Default)]
struct Collider;

#[derive(Component)]
struct Health(u8);

// This resource tracks the game's score
#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi;

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_transform: Single<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound =
        wall::LEFT_WALL + wall::WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound =
        wall::RIGHT_WALL - wall::WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = score.to_string();
}

fn check_for_brick_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut collindings: ParamSet<(
        Query<(&mut Velocity, &Transform), With<Brick>>,
        Query<(Entity, &Transform, Option<&mut Velocity>), With<Collider>>,
    )>,
) {
    let brick_query = collindings.p0();
    //let (mut brick_velocity, brick_transform) = brick_query.into_inner();
    let collider_query = collindings.p1();
}

fn check_for_ball_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<
        (
            Entity,
            &Transform,
            Option<&mut Health>,
            Option<&mut Velocity>,
        ),
        (With<Collider>, Without<Ball>),
    >,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();

    for (collider_entity, collider_transform, maybe_health, v) in collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Trigger observers of the "BallCollided" event
            commands.trigger(BallCollided);

            if let Some(mut health) = maybe_health {
                (*health).0 -= 1;
                if health.0 == 0 {
                    commands.entity(collider_entity).despawn();
                    **score += 1;
                } else {
                    if let Some(mut velocity) = v {
                        **velocity = ball_velocity.clone();
                        **velocity *= 0.5;
                    }
                }
            }

            // Reflect the ball's velocity when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // Reflect only if the velocity is in the opposite direction of the collision
            // This prevents the ball from getting stuck inside the bar
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // Reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // Reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn play_collision_sound(
    _collided: On<BallCollided>,
    mut commands: Commands,
    sound: Res<CollisionSound>,
) {
    commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}
