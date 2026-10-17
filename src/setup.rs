use super::{
    BALL_DIAMETER, Ball, Brick, Collider, CollisionSound, Health, PADDLE_SIZE, Paddle,
    ScoreboardUi, Velocity,
};
use crate::wall;
use bevy::prelude::*;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub(crate) const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
pub(crate) const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);

pub(crate) const BALL_SPEED: f32 = 300.0;

pub(crate) const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

pub(crate) const BRICK_SIZE: Vec2 = Vec2::new(40., 30.);

// These values are exact
pub(crate) const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;

pub(crate) const GAP_BETWEEN_BRICKS: f32 = 45.0;

// These values are lower bounds, as the number of bricks is computed
pub(crate) const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;

pub(crate) const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

pub(crate) const INITIAL_BRICKS_DIRECTION: Vec2 = Vec2::new(0.5, 0.5);

pub(crate) const SCOREBOARD_FONT_SIZE: FontSize = FontSize::Px(33.0);

pub(crate) const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

pub(crate) const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);

pub(crate) const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

pub(crate) const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);

pub(crate) const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);

pub(crate) const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

// Add the game's entities to our world
pub(crate) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = wall::BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
        Transform {
            translation: Vec3::new(0.0, paddle_y, 0.0),
            scale: PADDLE_SIZE.extend(1.0),
            ..default()
        },
        Paddle,
        Collider,
    ));

    // Ball
    commands.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Scoreboard
    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::default(),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
    ));

    // Walls
    commands.spawn(wall::Wall::new(wall::WallLocation::Left));
    commands.spawn(wall::Wall::new(wall::WallLocation::Right));
    commands.spawn(wall::Wall::new(wall::WallLocation::Bottom));
    commands.spawn(wall::Wall::new(wall::WallLocation::Top));

    // Bricks
    let total_width_of_bricks =
        (wall::RIGHT_WALL - wall::LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks =
        wall::TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (wall::LEFT_WALL + wall::RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn((
                Sprite {
                    color: BRICK_COLOR,
                    ..default()
                },
                Transform {
                    translation: brick_position.extend(0.0),
                    scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                    ..default()
                },
                Brick,
                Collider,
                Velocity(INITIAL_BRICKS_DIRECTION.normalize() * 0.0),
                Health(2),
            ));
        }
    }
}
