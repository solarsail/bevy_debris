use std::collections::BTreeMap;
use std::f32::consts::{FRAC_1_SQRT_2, PI};
use std::fmt;

use bevy::prelude::*;
use bevy::render::render_graph::base::MainPass;
use bevy_prototype_lyon::prelude::*;
use ordered_float::OrderedFloat;
use rand::prelude::*;

const POI_WIDTH: f32 = 30.0;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = materials.add(Color::rgb(0.8, 0.0, 0.0).into());
    let font = asset_server.load("arial.ttf");

    let cmd = commands
        .spawn(Camera2dComponents::default())
        .spawn(origin(material.clone(), &mut meshes));

    let mut targets = test_data(20);
    targets.sort_unstable_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
    let rings = arrange_targets(&targets, POI_WIDTH);
    for (ring_ord, ring) in rings.iter().enumerate() {
        cmd.spawn(ref_ring(material.clone(), &mut meshes, POI_WIDTH, ring_ord));
        for (azi, target) in ring {
            let r = ring_radius(POI_WIDTH, ring_ord);
            let trans = Vec3::new(r * azi.cos(), r * azi.sin(), 0.0);
            let (line, poi, text) = poi(
                material.clone(),
                &mut meshes,
                trans,
                font.clone(),
                target.text.clone(),
            );
            cmd.spawn(line).spawn(poi).spawn(text).with(MainPass);
        }
    }
}

fn origin(
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
) -> SpriteComponents {
    primitive(
        material,
        meshes,
        ShapeType::Circle(5.0),
        TessellationMode::Fill(&FillOptions::default()),
        Vec3::new(0.0, 0.0, 0.0).into(),
    )
}

fn ref_ring(
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    poi_width: f32,
    ring_ord: usize,
) -> SpriteComponents {
    let r = ring_radius(poi_width, ring_ord);
    primitive(
        material,
        meshes,
        ShapeType::Circle(r),
        TessellationMode::Stroke(&StrokeOptions::default()),
        Vec3::new(0.0, 0.0, 0.0).into(),
    )
}

fn poi(
    material: Handle<ColorMaterial>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    translation: Vec3,
    font: Handle<Font>,
    text: String,
) -> (SpriteComponents, SpriteComponents, TextComponents) {
    let square = primitive(
        material.clone(),
        meshes,
        ShapeType::Rectangle {
            width: POI_WIDTH,
            height: POI_WIDTH,
        },
        TessellationMode::Stroke(&StrokeOptions::default()),
        translation - Vec3::new(POI_WIDTH / 2.0, POI_WIDTH / 2.0, 0.0),
    );
    let line = primitive(
        material,
        meshes,
        ShapeType::Polyline {
            points: vec![point(0.0, 0.0), point(translation.x(), translation.y())],
            closed: false,
        },
        TessellationMode::Stroke(&StrokeOptions::default()),
        Vec3::new(0.0, 0.0, 0.0),
    );
    let textc = TextComponents {
        //style: Style {
        //    margin: Rect::all(Val::Px(1.0)),
        //    ..Default::default()
        //},
        text: Text {
            value: text,
            font,
            style: TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
            },
        },
        transform: Transform::from_translation(translation),
        ..Default::default()
    };
    (line, square, textc)
}

fn test_data(num: usize) -> Vec<Target> {
    let mut rng = rand::thread_rng();
    (0..num)
        .map(|id| {
            let text = format!("{}", id);
            Target {
                id: id as i32,
                text,
                azimuth: rng.gen_range(0.0, PI * 2.0),
                dist: rng.gen_range(10.0, 100.0),
            }
        })
        .collect()
}

#[derive(Clone)]
struct Target {
    pub id: i32,
    pub text: String,
    pub azimuth: f32,
    pub dist: f32,
}

impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Target")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("azimuth(deg)", &self.azimuth.to_degrees())
            .field("(rad)", &self.azimuth)
            .field("dist", &self.dist)
            .finish()
    }
}

fn arrange_targets(targets: &[Target], poi_width: f32) -> Vec<BTreeMap<OrderedFloat<f32>, Target>> {
    let mut rings = Vec::new();
    targets.iter().for_each(|t| {
        println!("{:?}", t);
        let mut ring_ord = 0;
        loop {
            let min_azi = min_angle(poi_width, ring_ord);
            println!(
                "\tring {}, min_azi(deg|rad): {}|{}",
                ring_ord,
                min_azi.to_degrees(),
                min_azi
            );
            if rings.len() == ring_ord {
                rings.push(BTreeMap::<OrderedFloat<f32>, Target>::new());
            }
            let ring = &mut rings[ring_ord];
            if ring.len() > 0 {
                let mut nearest = ring.range(OrderedFloat(t.azimuth)..);
                if let Some((azi, _)) = nearest.next() {
                    if **azi - t.azimuth < min_azi {
                        println!(
                            "\t\tnearest ge azimuth(deg|rad): {}|{}, overlap",
                            azi.to_degrees(),
                            azi
                        );
                        ring_ord += 1;
                        continue;
                    }
                } else if **ring.keys().next().unwrap() + PI * 2.0 - t.azimuth < min_azi {
                    let azi = ring.keys().next().unwrap();
                    println!(
                        "\t\tminimum azimuth(deg|rad): {}|{}, overlap",
                        azi.to_degrees(),
                        azi
                    );
                    ring_ord += 1;
                    continue;
                }
                let mut nearest = ring.range(..OrderedFloat(t.azimuth));
                if let Some((azi, _)) = nearest.next_back() {
                    if t.azimuth - **azi < min_azi {
                        println!(
                            "\t\tnearest lt azimuth(deg|rad): {}|{}, overlap",
                            azi.to_degrees(),
                            azi
                        );
                        ring_ord += 1;
                        continue;
                    }
                } else if t.azimuth + PI * 2.0 - **ring.keys().next_back().unwrap() < min_azi {
                    let azi = ring.keys().next_back().unwrap();
                    println!(
                        "\t\tmaximum azimuth(deg|rad): {}|{}, overlap",
                        azi.to_degrees(),
                        azi
                    );
                    ring_ord += 1;
                    continue;
                }
            }
            println!("\t\tno overlap, insert");
            ring.insert(OrderedFloat(t.azimuth), t.clone());
            break;
        }
    });
    rings
}

fn ring_radius(poi_width: f32, ring_ord: usize) -> f32 {
    (ring_ord + 1) as f32 * poi_width * 2.0
}

fn min_angle(poi_width: f32, ring_ord: usize) -> f32 {
    const SCATTER_COEF: f32 = 1.2;
    let r = ring_radius(poi_width, ring_ord);
    (poi_width * FRAC_1_SQRT_2 / r).asin() * 2.0 * SCATTER_COEF
}
