use amethyst::assets::{Handle, Loader};
use amethyst::core::nalgebra::{Matrix3, Vector3};
use amethyst::core::shrev::{EventChannel, ReaderId};
use amethyst::core::specs::world::Builder;
use amethyst::core::specs::Join;
use amethyst::core::{GlobalTransform, Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key_down};
use amethyst::renderer::{
    AmbientColor, Camera, DisplayConfig, DrawShaded, Light, Material, MaterialDefaults, MeshData,
    MeshHandle, Pipeline, PointLight, PosNormTex, RenderBundle, Rgba, ScreenDimensions, Shape,
    Stage, Texture, VirtualKeyCode,
};
use amethyst::{
    Application, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData, StateEvent, Trans,
};
use nphysics_ecs_dumb::ncollide::shape::{Ball, ShapeHandle};
use nphysics_ecs_dumb::nphysics::material::BasicMaterial as PhysicsMaterial;
use nphysics_ecs_dumb::nphysics::math::Velocity;
use nphysics_ecs_dumb::nphysics::volumetric::Volumetric;
use nphysics_ecs_dumb::*;
use num_traits::identities::One;
use std::time::Duration;
use amethyst::ecs::prelude::*;
use rand::{Rng, thread_rng};

extern crate rand;

#[derive(Default)]
struct GameState {
    pub collision_reader: Option<ReaderId<EntityContactEvent>>,
}

impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.register::<DynamicBody>();
        data.world.register::<MeshData>();
        data.world.register::<Handle<Texture>>();

        // Create a texture for using.
        let texture = data
            .world
            .read_resource::<Loader>()
            .load_from_data::<Texture, ()>(
                [170.0, 170.0, 255.0, 1.0].into(),
                (),
                &data.world.read_resource(),
            );

        let material = Material {
            albedo: texture,
            ..data.world.read_resource::<MaterialDefaults>().0.clone()
        };

        // Get resolution of the screen.
        let (x, y) = {
            let resolution = data.world.res.fetch::<ScreenDimensions>();
            (resolution.width(), resolution.height())
        };

        let camera_transform = Transform::from(Vector3::new(0.0, 5.0, 5.0));

        self.collision_reader = Some(
            data.world
                .write_resource::<EventChannel<EntityContactEvent>>()
                .register_reader(),
        );

        // Add Camera
        data.world
            .create_entity()
            .with(Camera::standard_3d(x, y))
            .with(camera_transform)
            .build();

        // Add Light
        data.world.add_resource(AmbientColor(Rgba::from([0.2; 3])));
        data.world
            .create_entity()
            .with(Light::Point(PointLight {
                intensity: 50.0,
                color: Rgba::white(),
                radius: 5.0,
                smoothness: 4.0,
            }))
            .with(Transform::from(Vector3::new(2.0, 2.0, -2.0)))
            .build();

        let sphere_shape = Shape::Sphere(32, 32).generate::<Vec<PosNormTex>>(None);
        let sphere_handle: MeshHandle = data.world.read_resource::<Loader>().load_from_data(
            sphere_shape,
            (),
            &data.world.read_resource(),
        );

        let ball = ShapeHandle::new(Ball::new(1.0));

        // Add Sphere (todo: add many, add rigidbodies and colliders)
        data.world
            .create_entity()
            .with(sphere_handle.clone())
            .with(material.clone())
            .with(Transform::from(Vector3::new(0.0, 15.0, -10.0)))
            .with(GlobalTransform::default())
            .with(DynamicBody::new_rigidbody_with_velocity(
                Velocity::linear(0.0, 1.0, 0.0),
                10.0,
                Matrix3::one(),
                ball.center_of_mass(),
            ))
            .with(
                ColliderBuilder::from(ball.clone())
                    .physics_material(PhysicsMaterial::default())
                    .build()
                    .unwrap(),
            )
            .build();

        // Add ground
        data.world
            .create_entity()
            .with(sphere_handle.clone())
            .with(material.clone())
            .with(Transform::from(Vector3::new(0.0, 0.0, -10.0)))
            .with(GlobalTransform::default())
            .with(
                //ColliderBuilder::from(ShapeHandle::new(Cuboid::new(Vector3::new(5.0, 1.0, 5.0))))
                ColliderBuilder::from(ball.clone())
                    .physics_material(PhysicsMaterial::default())
                    .build()
                    .unwrap(),
            )
            .build();

        data.world.create_entity()
            .with(Spawner {
                countdown: 20,
                remaining: 10,
                ball: ball.clone(),
                material: material,
                sphere: sphere_handle
            })
            .build();

        //---------------------------------------------------- nphysics's ball3.rs adapted

        /*let mut physics_world = data.world.write_resource::<PhysicsWorld>();
        physics_world.set_gravity(Vector3::new(0.0, -9.81, 0.0));

        // Material for all objects.
        let material = PhysicsMaterial::default();
        let ground_shape =
            ShapeHandle::new(Cuboid::new(Vector3::repeat(1.0 - 0.01)));
        let ground_pos = Isometry3::new(Vector3::new(0.0, -0.5, -15.0), nalgebra::zero());

        physics_world.add_collider(
            0.01,
            ground_shape,
            BodyHandle::ground(),
            ground_pos,
            material.clone(),
        );
        let geom = ShapeHandle::new(Ball::new(1.0 - 0.01));
        let inertia = geom.inertia(1.0);
        let center_of_mass = geom.center_of_mass();


        let pos = Isometry3::new(Vector3::new(0.0, 5.0, -15.0), nalgebra::zero());
        let handle = physics_world.add_rigid_body(pos, inertia, center_of_mass);
        physics_world.add_collider(
            0.01,
            geom.clone(),
            handle,
            Isometry3::identity(),
            material.clone(),
        );*/
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            for _ in data
                .world
                .read_resource::<EventChannel<EntityContactEvent>>()
                .read(self.collision_reader.as_mut().unwrap())
            {
                println!("Collision Event Detected.");
            }

            // Exit if user hits Escape or closes the window
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            //
            if is_key_down(&event, VirtualKeyCode::T) {
                *data.world.write_resource::<TimeStep>() = TimeStep::Fixed(1. / 120.);
                println!("Setting timestep to 1./120.");
            }

            if is_key_down(&event, VirtualKeyCode::Y) {
                *data.world.write_resource::<TimeStep>() = TimeStep::Fixed(1. / 60.);
                println!("Setting timestep to 1./60.");
            }

            if is_key_down(&event, VirtualKeyCode::S) {
                *data.world.write_resource::<TimeStep>() =
                    TimeStep::SemiFixed(TimeStepConstraint::new(
                        vec![1. / 240., 1. / 120., 1. / 60.],
                        0.4,
                        Duration::from_millis(50),
                        Duration::from_millis(500),
                    ))
            }

            // Reset the example
            if is_key_down(&event, VirtualKeyCode::Space) {
                *(
                    &mut data.world.write_storage::<Transform>(),
                    &data.world.read_storage::<DynamicBody>(),
                )
                    .join()
                    .next()
                    .unwrap()
                    .0
                    .translation_mut() = Vector3::new(0.0, 15.0, -10.0);
            }
        }
        Trans::None
    }
}

// A "countdown to deletion" component
struct TimeToLive(pub u32);
impl Component for TimeToLive {
    type Storage = DenseVecStorage<Self>;
}

// System that acts on `TimeToLive` by deleting entities when the countdown reaches 0
struct TimeToLiveSystem;
impl<'s> System<'s> for TimeToLiveSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, TimeToLive>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut ttl_storage,
        ) = data;
        for (mut ttl, entity) in (&mut ttl_storage, &entities).join() {
            if ttl.0 > 0 {
                ttl.0 -= 1;
            }
            if ttl.0 == 0 {
                println!("Deleting entity {:?}", entity);
                entities.delete(entity).unwrap();
            }
        }
    }
}

// Component for spawning a limited number of balls
struct Spawner {
    pub remaining: u32,
    pub countdown: u32,
    pub material: Material,
    pub sphere: MeshHandle,
    pub ball: ShapeHandle<f32>,
}
impl Component for Spawner {
    type Storage = HashMapStorage<Self>;
}

// System to run Spawners
struct SpawnerSystem;

impl<'s> System<'s> for SpawnerSystem {
    type SystemData = (
        WriteStorage<'s, Spawner>,
        Entities<'s>,
        // storages for spawning new entities
        WriteStorage<'s, Material>,
        WriteStorage<'s, MeshHandle>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, GlobalTransform>,
        WriteStorage<'s, DynamicBody>,
        WriteStorage<'s, Collider>,
        WriteStorage<'s, TimeToLive>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut spawners,
            entities,
            mut material_storage,
            mut mesh_storage,
            mut transform_storage,
            mut global_transform_storage,
            mut body_storage,
            mut collider_storage,
            mut ttl_storage,
        ) = data;

        for (mut spawner,) in (&mut spawners,).join() {
            if spawner.remaining > 0 {
                if spawner.countdown > 0 {
                    spawner.countdown -= 1;
                }
                if spawner.countdown == 0 {
                    spawner.countdown = 10;
                    spawner.remaining -= 1;

                    let mut rng = thread_rng();

                    // add a new ball at a random X position
                    entities.build_entity()
                        .with(spawner.sphere.clone(), &mut mesh_storage)
                        .with(spawner.material.clone(), &mut material_storage)
                        .with(Transform::from(Vector3::new(rng.gen_range(-5.0, 5.0), 15.0, -10.0)), &mut transform_storage)
                        .with(GlobalTransform::default(), &mut global_transform_storage)
                        .with(DynamicBody::new_rigidbody_with_velocity(
                            Velocity::linear(0.0, 1.0, 0.0),
                            10.0,
                            Matrix3::one(),
                            spawner.ball.center_of_mass(),
                        ), &mut body_storage)
                        .with(
                            ColliderBuilder::from(spawner.ball.clone())
                                .physics_material(PhysicsMaterial::default())
                                .build()
                                .unwrap(),
                            &mut collider_storage
                        )
                        .with(TimeToLive(200), &mut ttl_storage)
                        .build();
                }
            }
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_config = DisplayConfig {
        title: "Amethyst + Nphysics".to_string(),
        fullscreen: false,
        dimensions: Some((800, 400)),
        min_dimensions: Some((800, 400)),
        max_dimensions: None,
        icon: None,
        vsync: true,
        multisampling: 0, // Must be multiple of 2, use 0 to disable
        visibility: true,
        always_on_top: false,
        decorations: true,
        maximized: false,
        multitouch: true,
        resizable: true,
        transparent: false,
        loaded_icon: None,
    };
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.1, 0.1, 0.1, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            PhysicsBundle::new()
                .with_dep(&["transform_system"])
                .with_timestep_iter_limit(20),
        )?
        .with_bundle(RenderBundle::new(pipe, Some(display_config)))?
        .with(SpawnerSystem, "spawner_system", &[])
        .with(TimeToLiveSystem, "ttl_system", &[])
    ;

    let application = Application::new("./", GameState::default(), game_data);

    assert_eq!(application.is_ok(), true);

    application.ok().unwrap().run();

    Ok(())
}
