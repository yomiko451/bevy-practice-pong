#![windows_subsystem = "windows"] 

use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 720.;
const WINDOW_HEIGHT: f32 = 480.;
const BLOCK_HEIGHT: f32 = 100.;
const BLOCK_WIDTH: f32 = 10.;
const PLAYER_MOVE_SPEED: f32 = 500.;
const LINE_WIDTH: f32 = 2.;
const BALL_RADIUS: f32 = 10.;
const BALL_SPEED: Vec3 = Vec3::new(250., 250., 0.);
const MAIN_COLOR: Color = Color::srgb(1., 1., 1.);
const BACKGROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const BALL_COLOR: Color = Color::srgb(1., 0., 0.);

#[derive(Debug, Default, States, Hash, PartialEq, Eq, Clone)]
enum GmaeState {
    #[default]
    WatingToStart,
    InProgress,
    GameOver,
}

#[derive(Debug, Default, Resource)]
struct ReStartTimer(Timer);

#[derive(Debug, Component, Default)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>)]
struct Player;

#[derive(Debug, Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>)]
struct Opponent;

#[derive(Debug, Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>)]
struct Ball {
    velocity: Vec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hello Bevy!".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, init)
        .add_systems(
            FixedUpdate,
            (
                (player_movement, ball_movement, opponent_movement, game_over)
                    .run_if(in_state(GmaeState::InProgress)),
                (game_start).run_if(in_state(GmaeState::WatingToStart)),
                (restart_game).run_if(in_state(GmaeState::GameOver)),
            ),
        )
        .init_state::<GmaeState>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ReStartTimer(Timer::from_seconds(1., TimerMode::Once)))
        .run();
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(LINE_WIDTH, WINDOW_HEIGHT))),
        MeshMaterial2d(materials.add(MAIN_COLOR)),
        Transform::from_translation(Vec3::ZERO),
    ));

    commands.spawn((
        Ball {
            velocity: BALL_SPEED,
        },
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_xyz(0., 0., 1.),
    ));

    commands.spawn((
        Player::default(),
        Mesh2d(meshes.add(Rectangle::new(10., BLOCK_HEIGHT))),
        MeshMaterial2d(materials.add(MAIN_COLOR)),
        Transform::from_translation(Vec3::new(-((WINDOW_WIDTH - BLOCK_WIDTH) / 2.), 0., 0.)),
    ));

    commands.spawn((
        Opponent,
        Mesh2d(meshes.add(Rectangle::new(10., BLOCK_HEIGHT))),
        MeshMaterial2d(materials.add(MAIN_COLOR)),
        Transform::from_translation(Vec3::new((WINDOW_WIDTH - BLOCK_WIDTH) / 2., 0., 0.)),
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let edge_y = (WINDOW_HEIGHT - BLOCK_HEIGHT) / 2.;
    let delta = time.delta().as_secs_f32();
    let mut transform = query.get_single_mut().unwrap();
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += PLAYER_MOVE_SPEED * delta;
        if transform.translation.y > edge_y {
            transform.translation.y = edge_y;
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= PLAYER_MOVE_SPEED * delta;
        if transform.translation.y < -edge_y {
            transform.translation.y = -edge_y;
        }
    }
}

fn ball_movement(
    mut query_ball: Query<(&mut Transform, &mut Ball), Without<Player>>,
    query_player: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let delta = time.delta().as_secs_f32();
    let (mut ball_transform, mut ball) = query_ball.get_single_mut().unwrap();
    if ball_transform.translation.y > (WINDOW_HEIGHT - BALL_RADIUS) / 2. {
        ball.velocity.y = -ball.velocity.y;
    } else if ball_transform.translation.y < -(WINDOW_HEIGHT - BALL_RADIUS) / 2. {
        ball.velocity.y = -ball.velocity.y;
    }
    let player_transform = query_player.get_single().unwrap();
    if ball_transform.translation.x - BALL_RADIUS
        < player_transform.translation.x + BLOCK_WIDTH / 2.
        && ball_transform.translation.x > -WINDOW_WIDTH / 2.
        && ball_transform.translation.y - BALL_RADIUS
            < player_transform.translation.y + BLOCK_HEIGHT / 2.
        && ball_transform.translation.y + BALL_RADIUS
            > player_transform.translation.y - BLOCK_HEIGHT / 2.
    {
        ball.velocity.x = -ball.velocity.x;
    } else if ball_transform.translation.x + BALL_RADIUS > (WINDOW_WIDTH / 2.) - BLOCK_WIDTH {
        ball.velocity.x = -ball.velocity.x;
    }

    ball_transform.translation += ball.velocity * delta;
}

fn opponent_movement(
    mut query_opponent: Query<&mut Transform, With<Opponent>>,
    query_ball: Query<&Transform, (With<Ball>, Without<Opponent>)>,
    time: Res<Time>,
) {
    let ball_transform = query_ball.get_single().unwrap();
    if ball_transform.translation.x < 0. {
        return;
    }
    let delta = time.delta().as_secs_f32();
    let mut opponent_transform = query_opponent.get_single_mut().unwrap();
    if ball_transform.translation.y > opponent_transform.translation.y {
        opponent_transform.translation.y += PLAYER_MOVE_SPEED * delta;
    } else if ball_transform.translation.y < opponent_transform.translation.y {
        opponent_transform.translation.y -= PLAYER_MOVE_SPEED * delta;
    }
}

fn game_start(
    mut timer: ResMut<ReStartTimer>,
    mut next_state: ResMut<NextState<GmaeState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GmaeState::InProgress);
        timer.0.reset();
    }
}

fn game_over(
    mut next_state: ResMut<NextState<GmaeState>>,
    query_ball: Query<&Transform, With<Ball>>,
) {
    let ball_transform = query_ball.get_single().unwrap();
    if ball_transform.translation.x < -WINDOW_WIDTH / 2. {
        next_state.set(GmaeState::GameOver);
    }
}

fn restart_game(
    mut timer: ResMut<ReStartTimer>,
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<&mut Transform, With<Opponent>>,
        Query<&mut Transform, With<Ball>>,
    )>,
    mut ball: Query<&mut Ball>,
    mut next_state: ResMut<NextState<GmaeState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        let mut query_player = query_set.p0();
        let mut player_transform = query_player.get_single_mut().unwrap();
        player_transform.translation = Vec3::new(-((WINDOW_WIDTH - BLOCK_WIDTH) / 2.), 0., 0.);
        let mut query_opponent = query_set.p1();
        let mut opponent_transform = query_opponent.get_single_mut().unwrap();
        opponent_transform.translation = Vec3::new((WINDOW_WIDTH - BLOCK_WIDTH) / 2., 0., 0.);
        let mut query_ball = query_set.p2();
        let mut ball_transform = query_ball.get_single_mut().unwrap();
        ball_transform.translation = Vec3::new(0., 0., 1.);
        let mut ball = ball.get_single_mut().unwrap();
        ball.velocity = BALL_SPEED;

        next_state.set(GmaeState::WatingToStart);
    }
}
