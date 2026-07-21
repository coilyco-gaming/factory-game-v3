use bevy::prelude::*;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "factory-game shell".into(),
        fit_canvas_to_parent: true,
        ..default()
      }),
      ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
    .add_systems(Startup, setup)
    .add_systems(Update, orbit_marker)
    .run();
}

#[derive(Component)]
struct Marker;

fn setup(mut commands: Commands) {
  commands.spawn(Camera2d);
  commands.spawn((
    Text2d::new("factory-game shell"),
    Transform::from_xyz(0.0, 150.0, 0.0),
  ));
  commands.spawn((
    Text2d::new("status: app shell only - sim integration deferred"),
    Transform::from_xyz(0.0, -150.0, 0.0),
  ));
  commands.spawn((
    Sprite::from_color(Color::srgb(0.9, 0.6, 0.2), Vec2::splat(24.0)),
    Transform::default(),
    Marker,
  ));
}

fn orbit_marker(time: Res<Time>, mut markers: Query<&mut Transform, With<Marker>>) {
  let t = time.elapsed_secs();
  for mut transform in &mut markers {
    transform.translation.x = t.cos() * 96.0;
    transform.translation.y = t.sin() * 96.0;
  }
}
