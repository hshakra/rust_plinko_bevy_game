use bevy::{
    math::vec3,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{Window, WindowMode, WindowPlugin},
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 250.0, 0.0);
const BALL_FREEZE_POS: Vec3 = Vec3::new(0.0, -500.0, 0.0);
const BALL_DIAMETER: f32 = 10.0;
const BACKGROUND_COLOR: Color = Color::srgb(4.42 / 255.0, 5.72 / 255.0, 6.86 / 255.0);
const BALL_COLOR: Color = Color::srgb(187.0 / 255.0, 67.0 / 255.0, 67.0 / 255.0);
const FALL_VELO: f32 = 25.;
const COLLIDER_SIZE: f32 = 0.7;
const MULTI_TEXT_POS: Transform = Transform::from_xyz(-700.0, 400.0, 1.0);
const SHAKE_DURATION: f32 = 0.2;
const SHAKE_INTENSITY: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(BallArray(Vec::new()))
        .insert_resource(Money { money: 1000. })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, create_pyramid)
        .add_systems(Update, player_control)
        .add_systems(Update, modify_body_velocity)
        .add_systems(Update, update_ball)
        .add_systems(Update, multiplier_system)
        .add_systems(Update, text_update_system)
        .add_systems(Update, screen_shake_system)
        .run();
}

#[derive(Component)]
struct ScreenShake {
    timer: Timer,
    intensity: f32,
}

#[derive(Resource)]
struct Money {
    money: f32,
}

#[derive(Component)]
struct Ball {
    has_multiplied: bool,
}

#[derive(Component)]
struct BalanceTitle;

#[derive(Component)]
struct LatestMulti;

#[derive(Component)]
struct BoardBall;

#[derive(Resource)]
struct BallArray(pub Vec<Entity>);

fn setup(mut commands: Commands, balance: Res<Money>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: format!("$\t{:.0}", balance.money),
                    style: TextStyle {
                        font_size: 25.0,
                        color: Color::WHITE,
                        ..default()
                    },
                }],
                ..default()
            },
            transform: Transform::from_xyz(0.0, 400.0, 1.0),
            ..default()
        })
        .insert(BalanceTitle);

    // Initialize with empty lines for multipliers
    let initial_text = "LATEST MULTIS:\nx0\nx0\nx0\nx0\nx0\nx0\nx0\nx0\nx0";
    commands
        .spawn(Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: initial_text.to_string(),
                    style: TextStyle {
                        font_size: 15.0,
                        color: Color::WHITE,
                        ..default()
                    },
                }],
                justify: JustifyText::Left,
                ..default()
            },
            transform: MULTI_TEXT_POS,
            ..default()
        })
        .insert(LatestMulti);
}

fn modify_body_velocity(mut velocities: Query<&mut Velocity>) {
    for mut vel in velocities.iter_mut() {
        vel.linvel.y -= FALL_VELO;
        vel.angvel = 0.;
    }
}

fn update_ball(
    mut cmds: Commands,
    mut ball_query: Query<(Entity, &mut Transform), With<Ball>>,
    mut vec_res: ResMut<BallArray>,
) {
    let mut to_despawn = Vec::new();

    for (ball, mut ball_pos) in ball_query.iter_mut() {
        if ball_pos.translation.y < BALL_FREEZE_POS[1] {
            ball_pos.translation.y = BALL_FREEZE_POS[1];
            to_despawn.push(ball);
        }
    }

    for ball in to_despawn {
        cmds.entity(ball).despawn();
        vec_res.0.retain(|&b| b != ball);
    }
}

fn player_control(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut plink_balls: ResMut<BallArray>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut m: ResMut<Money>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        m.money = 3000.0;
    }
    if input.just_pressed(KeyCode::Space) || input.pressed(KeyCode::KeyW) {
        if m.money >= 100.0 {
            m.money -= 100.0;
            let spawn_pos: f32 = rand::thread_rng().gen_range(-10..=10) as f32;

            let new_ball: Ball = Ball {
                has_multiplied: false,
            };
            let ball_entity = commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::default()).into(),
                        material: materials.add(BALL_COLOR),
                        transform: Transform::from_translation(Vec3 {
                            x: BALL_STARTING_POSITION[0] + spawn_pos,
                            y: BALL_STARTING_POSITION[1],
                            z: 0.0,
                        })
                        .with_scale(Vec2::splat(BALL_DIAMETER).extend(0.0)),
                        ..Default::default()
                    },
                    new_ball,
                    RigidBody::Dynamic,
                    Collider::ball(COLLIDER_SIZE),
                ))
                .insert(Velocity {
                    linvel: Vec2::new(0.0, 0.0),
                    angvel: 0.,
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .id();
            plink_balls.0.push(ball_entity);
        } else {
            if let Ok(camera) = camera_query.get_single() {
                commands.entity(camera).insert(ScreenShake {
                    timer: Timer::from_seconds(SHAKE_DURATION, TimerMode::Once),
                    intensity: SHAKE_INTENSITY,
                });
            }
        }
    }
}

#[derive(Component)]
pub struct MultiplierBlock {
    power: f32,
}

fn create_pyramid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let horizontal_spacing = BALL_DIAMETER + 40.0;
    let vertical_spacing = BALL_DIAMETER + 25.0;
    let mut board_ball_pos: Vec3 = vec3(-horizontal_spacing, 200.0, 0.0);
    let mut balls_in_layer = 3;

    for a in 0..16 {
        let mut x: f32 = 8.0;
        for l in 0..balls_in_layer {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Circle::default()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(board_ball_pos)
                        .with_scale(Vec2::splat(BALL_DIAMETER).extend(0.0)),
                    ..default()
                },
                BoardBall,
                RigidBody::Fixed,
                Collider::ball(COLLIDER_SIZE),
            ));
            board_ball_pos.x += horizontal_spacing;

            let color = Color::srgb(57.0 / 255., 117.0 / 255.0, 57.0 / 255.0);
            if a >= 15 && l < balls_in_layer - 1 {
                let power_eq = (x * 0.7).powf(2.).round();

                commands
                    .spawn(Text2dBundle {
                        text: Text::from_section(
                            power_eq.to_string(),
                            TextStyle {
                                color: color,
                                font_size: 22.5,
                                ..default()
                            },
                        ),
                        transform: Transform {
                            translation: vec3(
                                board_ball_pos[0] - 25.,
                                board_ball_pos[1] - 20.,
                                board_ball_pos[2],
                            ),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(MultiplierBlock { power: power_eq })
                    .insert(RigidBody::Fixed)
                    .insert(Collider::cuboid(10., 10.))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(Sensor);

                if l >= 8 {
                    x += 1.0;
                } else {
                    x -= 1.0;
                }
            }
        }

        board_ball_pos.y -= vertical_spacing;
        balls_in_layer += 1;
        board_ball_pos.x = -((balls_in_layer as f32 - 1.0) * horizontal_spacing) / 2.0;
    }
}

fn multiplier_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut balance: ResMut<Money>,
    mut balls: Query<&mut Ball>,
    multipliers: Query<&MultiplierBlock>,
    mut multi_update_query: Query<&mut Text, With<LatestMulti>>,
) {
    for collision in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision {
            if let Ok(mut ball) = balls.get_mut(*e1) {
                if let Ok(multi) = multipliers.get(*e2) {
                    if !ball.has_multiplied {
                        balance.money += 100.0 * multi.power;
                        ball.has_multiplied = true;
                        update_multi_text(&mut multi_update_query, multi.power);
                    }
                }
            } else if let Ok(mut ball) = balls.get_mut(*e2) {
                if let Ok(multi) = multipliers.get(*e1) {
                    if !ball.has_multiplied {
                        balance.money += 100.0 * multi.power;
                        ball.has_multiplied = true;
                        update_multi_text(&mut multi_update_query, multi.power);
                    }
                }
            }
        }
    }
}

fn update_multi_text(multi_query: &mut Query<&mut Text, With<LatestMulti>>, power: f32) {
    if let Ok(mut text) = multi_query.get_single_mut() {
        let mut lines: Vec<String> = text.sections[0]
            .value
            .lines()
            .map(|s| s.to_string())
            .collect();

        // Keep the title
        let title = lines.remove(0);
        // Remove the oldest multiplier
        lines.pop();
        // Add new multiplier at the beginning (after title)
        lines.insert(0, format!("x{}", power));
        // Reconstruct text with title
        lines.insert(0, title);

        text.sections[0].value = lines.join("\n");
    }
}

fn text_update_system(mut text_query: Query<&mut Text, With<BalanceTitle>>, balance: Res<Money>) {
    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!("${}", balance.money);
    }
}

fn screen_shake_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ScreenShake), With<Camera2d>>,
) {
    for (entity, mut transform, mut shake) in query.iter_mut() {
        shake.timer.tick(time.delta());

        if shake.timer.finished() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            commands.entity(entity).remove::<ScreenShake>();
        } else {
            let progress = shake.timer.elapsed_secs() / SHAKE_DURATION;
            let shake_amount = shake.intensity * (1.0 - progress);
            transform.translation.x = rand::thread_rng().gen_range(-shake_amount..shake_amount);
            transform.translation.y = rand::thread_rng().gen_range(-shake_amount..shake_amount);
        }
    }
}
