use std::f32::consts::PI;

use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        ElementState,
    },
    prelude::*,
    render::{camera::Camera, mesh::Indices, pipeline::PrimitiveTopology},
};

fn main() {
    App::build()
        .add_resource(MouseButtonState { pressed: false })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(mouse_events_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //let sphere_handle = meshes.add(Mesh::from(shape::Icosphere {
    //    radius: 1.0,
    //    subdivisions: 5,
    //}));
    let sphere_handle = meshes.add(sphere_mesh(2.0, 45, 180));
    //let sphere_handle = meshes.add(icosphere_mesh(2.0, 5));
    let texture_handle = asset_server.load("theworld.png");
    let material_handle = materials.add(StandardMaterial {
        albedo_texture: Some(texture_handle.clone()),
        shaded: false,
        ..Default::default()
    });
    commands
        // textured quad - normal
        .spawn(PbrComponents {
            mesh: sphere_handle.clone(),
            material: material_handle,
            transform: Transform::from_rotation(Quat::from_rotation_ypr(0.0, PI * 0.6, PI / 6.0)),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 6.0)),
            ..Default::default()
        });
}

struct MouseButtonState {
    pressed: bool,
}

#[derive(Default)]
struct State {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

fn mouse_events_system(
    mut state: Local<State>,
    mut btn: ResMut<MouseButtonState>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    //cursor_moved_events: Res<Events<CursorMoved>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    mut sphere_query: Query<(&Handle<Mesh>, Mut<Transform>)>,
    mut camera_query: Query<(&Camera, Mut<Transform>)>,
) {
    for event in state
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state,
            } => match state {
                ElementState::Pressed => btn.pressed = true,
                ElementState::Released => btn.pressed = false,
            },
            _ => {}
        }
    }

    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        if btn.pressed {
            let MouseMotion { delta } = event;
            if delta.length_squared() > 0.0 {
                for (_, mut transform) in sphere_query.iter_mut() {
                    let phi = delta.x() * PI / 720.0;
                    let theta = delta.y() * PI / 720.0;
                    let r = Quat::from_rotation_ypr(phi, theta, 0.0);
                    transform.rotation = r * transform.rotation;
                }
            }
        }
    }

    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        let MouseWheel { unit: _, x: _, y } = event;
        for (_, mut transform) in camera_query.iter_mut() {
            let delta = transform.translation.normalize() * *y;
            let new_translation = transform.translation - delta;
            if new_translation.length() > 3.0 {
                transform.translation = new_translation;
            }
        }
    }
}

#[allow(dead_code)]
fn icosphere_mesh(radius: f32, divisions: usize) -> Mesh {
    use hexasphere::IcoSphere;
    let hexasphere = IcoSphere::new(divisions, |point| {
        let inclination = if point.x() > 0.0 {
            point.z().acos()
        } else {
            PI * 2.0 - point.z().acos()
        };
        let azumith = Vec2::new(point.x(), point.z()).length().acos() * point.y().signum();
        //let azumith = point.y().atan2(point.x().abs());

        let norm_inclination = inclination / (PI * 2.0);
        let norm_azumith = (azumith / PI) + 0.5;

        [norm_inclination, norm_azumith]
    });
    let raw_points = hexasphere.raw_points();

    let points = raw_points
        .iter()
        .map(|&p| (p * radius).into())
        .collect::<Vec<[f32; 3]>>();

    let normals = raw_points
        .iter()
        .copied()
        .map(Into::into)
        .collect::<Vec<[f32; 3]>>();

    let uvs = hexasphere.raw_data().to_owned();

    let mut indices = Vec::with_capacity(hexasphere.indices_per_main_triangle() * 20);

    for i in 0..20 {
        hexasphere.get_indices(i, &mut indices);
    }

    let indices = Indices::U32(indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, points.into());
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals.into());
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs.into());
    mesh
}

fn sphere_mesh(radius: f32, lat_counts: u32, lon_counts: u32) -> Mesh {
    let lat_step = PI / lat_counts as f32;
    let lon_step = PI * 2.0 / lon_counts as f32;
    let vertex_count = ((lat_counts + 1) * (lon_counts + 1)) as usize;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    for lon in 0..=lon_counts {
        let theta = lon_step * lon as f32;
        for lat in 0..=lat_counts {
            let azu = -PI / 2.0 + lat_step * lat as f32;
            let pos = Vec3::new(
                radius * theta.cos() * azu.cos(),
                radius * theta.sin() * azu.cos(),
                radius * azu.sin(),
            );
            positions.push([pos.x(), pos.y(), pos.z()]);
            let n = pos.normalize();
            normals.push([n.x(), n.y(), n.z()]);
            uvs.push([
                1.0 - lon as f32 / lon_counts as f32,
                lat as f32 / lat_counts as f32,
            ])
        }
    }
    let mut indices = Vec::with_capacity((lon_counts * lat_counts) as usize);
    for lon in 0..lon_counts {
        let idx = lon * (lat_counts + 1);
        for lat in 0..lat_counts {
            let idx = idx + lat;
            if lat < lat_counts {
                indices.extend(vec![idx, idx + lat_counts + 1, idx + 1]);
            }
            if lat > 0 {
                indices.extend(vec![idx, idx + lat_counts, idx + lat_counts + 1]);
            }
        }
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions.into());
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals.into());
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs.into());
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
