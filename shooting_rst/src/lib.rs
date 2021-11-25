// use gdnative::api::RigidBody;
use gdnative::api::AudioStreamPlayer;
use gdnative::prelude::*;
use std::f32::consts::PI;

struct Env {
    time: f32,
    theta: f32,
    left_limit: f32,
    right_limit: f32,
    up_limit: f32,
    down_limit: f32,
}

#[derive(NativeClass)]
#[inherit(KinematicBody)]
struct Player {
    #[property(default = 5.0)]
    speed: f32,
    #[property]
    bullet: Ref<PackedScene>,

    left_barrel: Option<Ref<Spatial>>,
    right_barrel: Option<Ref<Spatial>>,
    laser: Option<Ref<AudioStreamPlayer>>,

    env: Env,
}

#[gdnative::methods]
impl Player {
    fn new(_owner: &KinematicBody) -> Self {
        Self {
            speed: 5.0,
            bullet: PackedScene::new().into_shared(),

            left_barrel: None,
            right_barrel: None,
            laser: None,

            env: Env {
                time: 0.0,
                theta: 0.0,
                left_limit: 0.0,
                right_limit: 0.0,
                up_limit: 0.0,
                down_limit: 0.0,
            },
        }
    }

    #[export]
    fn _ready(&mut self, owner: &KinematicBody) {
        let left_limit_node = unsafe { owner.get_node_as::<Spatial>("../left_limit").unwrap() };
        let right_limit_node = unsafe { owner.get_node_as::<Spatial>("../right_limit").unwrap() };
        let up_limit_node = unsafe { owner.get_node_as::<Spatial>("../up_limit").unwrap() };
        let down_limit_node = unsafe { owner.get_node_as::<Spatial>("../down_limit").unwrap() };

        self.left_barrel = Some(unsafe {
            owner
                .get_node_as::<Spatial>("fighter/LeftBarrel")
                .unwrap()
                .claim()
        });
        self.right_barrel = Some(unsafe {
            owner
                .get_node_as::<Spatial>("fighter/RightBarrel")
                .unwrap()
                .claim()
        });
        self.laser = Some(unsafe {
            owner
                .get_node_as::<AudioStreamPlayer>("laser")
                .unwrap()
                .claim()
        });

        self.env.left_limit = left_limit_node.translation().x;
        self.env.right_limit = right_limit_node.translation().x;
        self.env.up_limit = up_limit_node.translation().z;
        self.env.down_limit = down_limit_node.translation().z;

        godot_print!("_ready@Player {}", env!("CARGO_PKG_VERSION"));
    }

    #[export]
    fn _process(&mut self, owner: &KinematicBody, delta: f64) {
        /*
        self.time += delta as f32;
        self.theta = self.time.sin() * 90.0;
        owner.set_rotation(Vector3::new(0.0, self.theta, 0.0));
        */
        self.env.time += delta as f32;
        self.env.theta = self.env.time / 0.8 * PI; // 0.8sで一周
        if self.env.theta > 2.0 * PI {
            self.env.theta -= 2.0 * PI;
        }

        let input = Input::godot_singleton();

        let sp_weight = if input.is_action_pressed("shoot") {
            self.shoot(owner);
            0.5
        } else {
            1.0
        };

        self.wave_move(owner);
        self.move_control(owner, delta, input, sp_weight);
        self.coordinate_modifying(owner);
    }

    fn wave_move(&self, owner: &KinematicBody) {
        let d = self.env.theta.cos() * 0.005;
        let v = Vector3::new(0.0, d, 0.0);
        // owner.move_and_collide(v, false, false, false);
        owner.translate(v);
    }

    fn move_control(&self, owner: &KinematicBody, delta: f64, input: &Input, speed_weight: f32) {
        let mut v = Vector3::new(0.0, 0.0, 0.0);
        if input.is_action_pressed("ui_right") {
            v.x += 1.0;
        }
        if input.is_action_pressed("ui_left") {
            v.x -= 1.0;
        }
        if input.is_action_pressed("ui_up") {
            v.z -= 1.0;
        }
        if input.is_action_pressed("ui_down") {
            v.z += 1.0;
        }
        if v.length() > 0.0 {
            v = v.normalize() * self.speed * speed_weight;
        }
        let fighter = unsafe { owner.get_node_as::<Spatial>("fighter").unwrap() };
        #[allow(non_upper_case_globals)]
        const max: f32 = PI / 16.0;
        if v.x > 0.0 {
            fighter.rotate(Vector3::new(0.0, 0.0, 1.0), -delta);
            if fighter.rotation().z < -max {
                fighter.set_rotation(Vector3::new(0.0, 0.0, -max));
            }
        } else if v.x < 0.0 {
            fighter.rotate(Vector3::new(0.0, 0.0, 1.0), delta);
            if fighter.rotation().z > max {
                fighter.set_rotation(Vector3::new(0.0, 0.0, max));
            }
        } else {
            fighter.set_rotation(Vector3::new(0.0, 0.0, 0.0));
        }
        owner.translate(v * delta as f32);
        // owner.move_and_collide(v * delta as f32, false, false, false);
    }

    fn coordinate_modifying(&self, owner: &KinematicBody) {
        if owner.translation().x < self.env.left_limit {
            owner.set_translation(Vector3::new(
                self.env.left_limit,
                owner.translation().y,
                owner.translation().z,
            ));
        }
        if owner.translation().x > self.env.right_limit {
            owner.set_translation(Vector3::new(
                self.env.right_limit,
                owner.translation().y,
                owner.translation().z,
            ));
        }
        if owner.translation().z > self.env.down_limit {
            owner.set_translation(Vector3::new(
                owner.translation().x,
                owner.translation().y,
                self.env.down_limit,
            ));
        }
        if owner.translation().z < self.env.up_limit {
            owner.set_translation(Vector3::new(
                owner.translation().x,
                owner.translation().y,
                self.env.up_limit,
            ));
        }
    }

    fn shoot(&self, owner: &KinematicBody) {
        let t = (self.env.time * 100.0) as u64;

        if t % 5 != 0 {
            return;
        }

        let barrel = unsafe {
            if t % 2 == 0 {
                self.left_barrel.as_ref().unwrap().assume_safe()
            } else {
                self.right_barrel.as_ref().unwrap().assume_safe()
            }
        };
        let pos = barrel.global_transform().origin;

        let bullet_scene: Ref<KinematicBody, _> = instance_scene(&self.bullet);
        bullet_scene.set_translation(pos);
        if let Some(parent) = owner.get_parent() {
            let parent = unsafe { parent.assume_safe() };
            parent.add_child(bullet_scene, false);
        }

        unsafe { self.laser.as_ref().unwrap().assume_safe() }.play(0.0);

        /*
        let bullet = bullet_scene.cast_instance::<Bullet>().unwrap();
        bullet.map(|_, owner| owner.set_liner_velocity());
        */
    }
}

// https://github.com/godot-rust/godot-rust/blob/master/examples/dodge_the_creeps/src/main_scene.rs
fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
where
    Root: gdnative::object::GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
{
    let scene = unsafe { scene.assume_safe() };

    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .expect("should be able to instance scene");

    let instance = unsafe { instance.assume_unique() };

    instance
        .try_cast::<Root>()
        .expect("root node type should be correct")
}

#[derive(NativeClass)]
#[inherit(KinematicBody)]
struct Bullet {
    #[property(default = 5.0)]
    speed: f32,
}

#[gdnative::methods]
impl Bullet {
    fn new(_owner: &KinematicBody) -> Self {
        Self { speed: 5.0 }
    }

    #[export]
    fn _ready(&mut self, _owner: &KinematicBody) {
        // godot_print!("_ready@Bullet {}", env!("CARGO_PKG_VERSION"));
    }

    #[export]
    fn _process(&self, owner: &KinematicBody, delta: f64) {
        let d = Vector3::new(0.0, 0.0, -self.speed * delta as f32);
        let c = owner.move_and_collide(d, false, false, false);
        // owner.translate(d);
        if c.is_some() {
            unsafe {
                owner.assume_unique().queue_free();
            }
        }
    }
}

#[derive(NativeClass)]
#[inherit(KinematicBody)]
struct Alien {
    #[property(default = 5.0)]
    speed: f32,

    env: Env,
}

#[gdnative::methods]
impl Alien {
    fn new(_owner: &KinematicBody) -> Self {
        Self {
            speed: 5.0,

            env: Env {
                time: 0.0,
                theta: 0.0,
                left_limit: 0.0,
                right_limit: 0.0,
                up_limit: 0.0,
                down_limit: 0.0,
            },
        }
    }

    #[export]
    fn _ready(&mut self, owner: &KinematicBody) {
        let left_limit_node = unsafe { owner.get_node_as::<Spatial>("../left_limit").unwrap() };
        let right_limit_node = unsafe { owner.get_node_as::<Spatial>("../right_limit").unwrap() };
        let up_limit_node = unsafe { owner.get_node_as::<Spatial>("../up_limit").unwrap() };
        let down_limit_node = unsafe { owner.get_node_as::<Spatial>("../down_limit").unwrap() };

        self.env.left_limit = left_limit_node.translation().x;
        self.env.right_limit = right_limit_node.translation().x;
        self.env.up_limit = up_limit_node.translation().z;
        self.env.down_limit = down_limit_node.translation().z;
        // godot_print!("_ready@Alien {}", env!("CARGO_PKG_VERSION"));
    }

    #[export]
    fn _process(&mut self, owner: &KinematicBody, delta: f64) {
        let d = Vector3::new(-self.speed * delta as f32, 0.0, 0.0);
        let c = owner.move_and_collide(d, false, false, false);
        // owner.translate(d);
        if c.is_some() {
            unsafe {
                owner
                    .get_node_as::<AudioStreamPlayer>("../attackSound")
                    .unwrap()
                    .play(0.0);
            }

            unsafe {
                owner.assume_unique().queue_free();
            }

            return;
        }

        if (owner.translation().x < self.env.left_limit && self.speed > 0.0)
            || (owner.translation().x > self.env.right_limit && self.speed < 0.0)
        {
            self.speed *= -1.0;
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Player>();
    handle.add_class::<Bullet>();
    handle.add_class::<Alien>();
}

godot_init!(init);
