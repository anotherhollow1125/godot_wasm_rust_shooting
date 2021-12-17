// use gdnative::api::RigidBody;
use gdnative::api::{Area, AudioStreamPlayer, CPUParticles, CollisionShape, RandomNumberGenerator};
use gdnative::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::PI;

struct Env {
    time: f32,
    theta: f32,
    left_limit: f32,
    right_limit: f32,
    up_limit: f32,
    down_limit: f32,
}

impl Env {
    pub fn new() -> Self {
        Env {
            time: 0.0,
            theta: 0.0,
            left_limit: 0.0,
            right_limit: 0.0,
            up_limit: 0.0,
            down_limit: 0.0,
        }
    }

    pub fn init(&mut self, owner: &Node) {
        let left_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/left_limit")
                .unwrap()
        };
        let right_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/right_limit")
                .unwrap()
        };
        let up_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/up_limit")
                .unwrap()
        };
        let down_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/down_limit")
                .unwrap()
        };

        self.left_limit = left_limit_node.translation().x;
        self.right_limit = right_limit_node.translation().x;
        self.up_limit = up_limit_node.translation().z;
        self.down_limit = down_limit_node.translation().z;
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register_signals)]
struct Stage {
    #[property(default = 100)]
    alien_num: i32,
    #[property]
    alien_scene: Ref<PackedScene>,
    aliens_magazine: Option<Magazine<Alien>>,

    alien_spawn_timer: Option<Ref<Timer>>,
    rng: Option<Ref<RandomNumberGenerator, Unique>>,

    env: Env,

    #[property(default = 3)]
    player_life: i32,
    beated_alien_num: i32,
    #[property(default = 1.0)]
    stage_heat: f32,
}

#[gdnative::methods]
impl Stage {
    fn new(_owner: &Node) -> Self {
        Stage {
            alien_num: 100,
            alien_scene: PackedScene::new().into_shared(),
            aliens_magazine: None,
            alien_spawn_timer: None,
            rng: None,
            env: Env::new(),

            player_life: 3,
            beated_alien_num: 0,
            stage_heat: 1.0,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "restart_player",
            args: &[],
        });
        builder.add_signal(Signal {
            name: "speed_up",
            args: &[SignalArgument {
                name: "speed",
                default: Variant::from_f64(1.0),
                export_info: ExportInfo::new(VariantType::F64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        let rng = RandomNumberGenerator::new();
        rng.randomize();
        let alien_spawn_timer = unsafe {
            let t = owner.get_node_as::<Timer>("alien_spawn_timer").unwrap();
            t.start(rng.randf_range(0.5, 1.0));
            t.claim()
        };
        self.rng = Some(rng);
        self.alien_spawn_timer = Some(alien_spawn_timer);
        self.aliens_magazine = Some(Magazine::new(&self.alien_scene, self.alien_num as usize));
        self.env.init(owner);

        let player = unsafe { owner.get_node_as::<Area>("/root/stage/PlayerRoot").unwrap() };
        owner
            .connect(
                "restart_player",
                player,
                "reset",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        owner.emit_signal("restart_player", &[]);

        // godot_print!("stage ready");
    }

    #[allow(unused)]
    fn get_magazine(&self) -> &Magazine<Alien> {
        self.aliens_magazine.as_ref().unwrap()
    }

    fn mut_magazine(&mut self) -> &mut Magazine<Alien> {
        self.aliens_magazine.as_mut().unwrap()
    }

    #[export]
    fn spawn_alien(&mut self, owner: &Node) {
        // godot_print!("spawn_alien");

        let (spawn_x, interval) = match self.rng {
            Some(ref mut rng) => {
                let interval = rng.randf_range(0.5, 3.0);
                let spawn_x =
                    rng.randf_range(self.env.left_limit as f64, self.env.right_limit as f64);
                (spawn_x, interval)
            }
            None => return,
        };
        let aliens_scene = match self.mut_magazine().hammer() {
            Some(a) => {
                a.map_mut(|t, owner| {
                    t.reset(&owner);
                    t.speed_up(&owner, self.stage_heat);
                    t.set_process(Alien::invasion_pattern);
                })
                .ok();
                a.into_base()
            }
            None => {
                // godot_print!("No aliens left");
                return;
            }
        };

        // godot_print!("spawn_alien beep");

        let pos = Vector3::new(spawn_x as f32, 0.0, -27.0);
        // let pos = Vector3::new(spawn_x as f32, 0.0, -10.0);

        aliens_scene.set_translation(pos);
        owner.add_child(aliens_scene, false);
        unsafe {
            self.alien_spawn_timer
                .as_ref()
                .unwrap()
                .assume_unique()
                .set_wait_time(interval);
        }

        // godot_print!("spawn_alien_beep");
    }

    #[export]
    fn collect_alien(&mut self, _owner: &Node, alien_var: Variant) {
        // godot_print!("stage collect alien");
        let alien_area: Ref<Area, Unique> =
            unsafe { alien_var.try_to_object().unwrap().assume_unique() };
        self.mut_magazine().charge_bullet(alien_area);
        /*
        godot_print!(
            "end stage collect alien: {}",
            self.get_magazine().get_left_num()
        );
        */
    }

    #[export]
    fn player_beated(&mut self, owner: &Node) {
        self.player_life -= 1;
        if self.player_life > 0 {
            godot_print!("Player Restart");
            unsafe {
                owner.call_deferred("player_restart", &[]);
            }
        }
    }

    #[export]
    fn player_restart(&mut self, owner: &Node) {
        owner.emit_signal("restart_player", &[]);
    }

    #[export]
    fn alien_beated(&mut self, owner: &Node) {
        self.beated_alien_num += 1;
        if self.beated_alien_num % 10 == 0 {
            self.stage_heat *= 1.1;
            owner.emit_signal("speed_up", &[Variant::from_f64(self.stage_heat as f64)]);
            godot_print!("Stage Heat: {}", self.stage_heat);
        }
        // godot_print!("beated_alien_num: {}", self.beated_alien_num);
    }
}

struct Magazine<T>
where
    T: NativeClass,
    <T as NativeClass>::Base:
        gdnative::object::GodotObject<RefKind = ManuallyManaged> + SubClass<Node> + QueueFree,
{
    bullets: VecDeque<Instance<T, Unique>>,
}

impl<T> Magazine<T>
where
    T: NativeClass,
    <T as NativeClass>::Base:
        gdnative::object::GodotObject<RefKind = ManuallyManaged> + SubClass<Node> + QueueFree,
{
    pub fn new(bullet_scene: &Ref<PackedScene, Shared>, bullet_num: usize) -> Self {
        let bullets: VecDeque<_> = (0..bullet_num)
            .filter_map(|_| {
                let r = instance_scene(bullet_scene)?;
                Instance::from_base(r)
            })
            .collect();

        Magazine { bullets }
    }

    pub fn charge_bullet(&mut self, bullet_base: Ref<T::Base, Unique>) {
        let bullet = match Instance::from_base(bullet_base) {
            Some(bullet) => bullet,
            None => return,
        };
        self.bullets.push_back(bullet);
    }

    pub fn get_left_num(&self) -> usize {
        self.bullets.len()
    }

    pub fn hammer(&mut self) -> Option<Instance<T, Unique>> {
        self.bullets.pop_front()
    }
}

impl<T> Drop for Magazine<T>
where
    T: NativeClass,
    <T as NativeClass>::Base:
        gdnative::object::GodotObject<RefKind = ManuallyManaged> + SubClass<Node> + QueueFree,
{
    fn drop(&mut self) {
        while let Some(bullet) = self.bullets.pop_front() {
            bullet.queue_free();
        }
    }
}

#[derive(NativeClass)]
#[inherit(Area)]
#[register_with(Self::register_signals)]
struct Player {
    #[property(default = 100)]
    bullet_num: i32,
    #[property(default = 5.0)]
    speed: f32,
    setted_speed: f32,
    #[property]
    bullet_scene: Ref<PackedScene>,

    magazine: Option<Magazine<Bullet>>,

    left_barrel: Option<Ref<Spatial>>,
    right_barrel: Option<Ref<Spatial>>,
    laser: Option<Ref<AudioStreamPlayer>>,

    env: Env,

    alive: bool,
    beated_sound: Option<Ref<AudioStreamPlayer, Unique>>,
    collision_shape: Option<Ref<CollisionShape, Unique>>,
    fighter: Option<Ref<Spatial, Unique>>,
    frag: Option<Ref<CPUParticles, Unique>>,
    destruct_timer: Option<Ref<Timer, Unique>>,
    blink_timer: Option<Ref<Timer, Unique>>,
    on_collision_timer: Option<Ref<Timer, Unique>>,
}

#[gdnative::methods]
impl Player {
    fn new(_owner: &Area) -> Self {
        Self {
            bullet_num: 100,
            speed: 5.0,
            setted_speed: 5.0,
            bullet_scene: PackedScene::new().into_shared(),

            magazine: None,

            left_barrel: None,
            right_barrel: None,
            laser: None,

            env: Env::new(),

            alive: true,
            beated_sound: None,
            collision_shape: None,
            fighter: None,
            frag: None,
            destruct_timer: None,
            blink_timer: None,
            on_collision_timer: None,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "player_beated",
            args: &[],
        });
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        self.setted_speed = self.speed;
        self.magazine = Some(Magazine::new(&self.bullet_scene, self.bullet_num as usize));

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

        self.env.init(owner);

        self.alive = false;
        self.beated_sound = Some(unsafe {
            owner
                .get_node_as::<AudioStreamPlayer>("beatedSound")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.collision_shape = Some(unsafe {
            owner
                .get_node_as::<CollisionShape>("CollisionShape")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.fighter = Some(unsafe {
            owner
                .get_node_as::<Spatial>("fighter")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.frag = Some(unsafe {
            owner
                .get_node_as::<CPUParticles>("frag")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.destruct_timer = Some(unsafe {
            owner
                .get_node_as::<Timer>("DestructTimer")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.blink_timer = Some(unsafe {
            owner
                .get_node_as::<Timer>("BlinkTimer")
                .unwrap()
                .claim()
                .assume_unique()
        });
        self.on_collision_timer = Some(unsafe {
            owner
                .get_node_as::<Timer>("OnCollisionTimer")
                .unwrap()
                .claim()
                .assume_unique()
        });

        self.reset(owner);

        let stage = unsafe { owner.get_node("/root/stage").unwrap().assume_safe() };
        owner
            .connect(
                "player_beated",
                stage,
                "player_beated",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        godot_print!("_ready@Player {}", env!("CARGO_PKG_VERSION"));
    }

    #[export]
    fn reset(&mut self, owner: &Area) {
        godot_print!("reset@Player");
        self.alive = true;
        owner.set_translation(Vector3::new(0.0, 0.0, 0.0));
        self.speed = self.setted_speed;
        if let Some(s) = self.fighter.as_ref() {
            s.set_visible(true);
            // godot_print!("reset visibility");
        }
        /*
        if let Some(c) = self.collision_shape.as_ref() {
            c.set_disabled(false);
            // godot_print!("reset collision");
        }
        */
        if let Some(t) = self.blink_timer.as_ref() {
            t.start(0.0);
        }
        if let Some(t) = self.on_collision_timer.as_ref() {
            t.start(0.0);
        }
    }

    #[export]
    fn blink(&mut self, _owner: &Area) {
        if let Some(fighter) = self.fighter.as_ref() {
            fighter.set_visible(!fighter.is_visible());
        }
    }

    #[export]
    fn enable_collision(&mut self, _owner: &Area) {
        if let Some(c) = self.collision_shape.as_ref() {
            c.set_disabled(false);
        }
        if let Some(fighter) = self.fighter.as_ref() {
            fighter.set_visible(true);
        }
        if let Some(t) = self.blink_timer.as_ref() {
            t.stop();
        }
    }

    #[export]
    fn _physics_process(&mut self, owner: &Area, delta: f64) {
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

    fn wave_move(&self, owner: &Area) {
        let d = self.env.theta.cos() * 0.005;
        let v = Vector3::new(0.0, d, 0.0);
        // owner.move_and_collide(v, false, false, false);
        owner.translate(v);
    }

    fn move_control(&self, owner: &Area, delta: f64, input: &Input, speed_weight: f32) {
        if !self.alive {
            return;
        }

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
        let fighter = self.fighter.as_ref().unwrap();
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

    fn coordinate_modifying(&self, owner: &Area) {
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

    fn get_magazine(&self) -> &Magazine<Bullet> {
        self.magazine.as_ref().unwrap()
    }

    fn mut_magazine(&mut self) -> &mut Magazine<Bullet> {
        self.magazine.as_mut().unwrap()
    }

    #[export]
    fn crash(&mut self, owner: &Area, other_area_var: Variant) {
        // godot_print!("crash beep");
        if !self.alive {
            return;
        }

        let area: TRef<Area, _> = unsafe { other_area_var.try_to_object().unwrap().assume_safe() };
        if area.collision_layer() != 2 && area.collision_layer() != 8 {
            // godot_print!("collision_layer: {}", area.collision_layer());
            return;
        }

        godot_print!("crash@Player");

        self.speed = 0.0;
        self.alive = false;
        self.frag.as_ref().unwrap().set_emitting(true);
        self.beated_sound.as_ref().unwrap().play(0.0);
        self.fighter.as_ref().unwrap().set_visible(false);
        self.destruct_timer.as_ref().unwrap().start(0.0);

        unsafe {
            owner.call_deferred("disable_collision", &[]);
        }
    }

    #[export]
    fn alert_beated(&self, owner: &Area) {
        owner.emit_signal("player_beated", &[]);
    }

    #[export]
    fn disable_collision(&mut self, _owner: &Area) {
        self.collision_shape.as_ref().unwrap().set_disabled(true);
    }

    fn shoot(&mut self, owner: &Area) {
        if !self.alive {
            return;
        }

        if self.get_magazine().get_left_num() < 1 {
            return;
        }

        let bullet_scene = match self.mut_magazine().hammer() {
            Some(b) => {
                b.map_mut(|bb, _| bb.flying = true).ok();
                b.into_base()
            }
            None => return,
        };

        let t = (self.env.time * 100.0) as u64;

        let barrel = unsafe {
            if t % 2 == 0 {
                self.left_barrel.as_ref().unwrap().assume_safe()
            } else {
                self.right_barrel.as_ref().unwrap().assume_safe()
            }
        };
        let pos = barrel.global_transform().origin;

        // let bullet_scene: Ref<Area, _> = instance_scene(&self.bullet_scene);
        bullet_scene.set_translation(pos);
        if let Some(parent) = owner.get_parent() {
            let parent = unsafe { parent.assume_safe() };
            parent.add_child(bullet_scene, false);
        }

        unsafe { self.laser.as_ref().unwrap().assume_safe() }.play(0.0);
    }

    #[export]
    fn collect_bullet(&mut self, _owner: &Area, bullet_var: Variant) {
        let bullet_area: Ref<Area, Unique> =
            unsafe { bullet_var.try_to_object().unwrap().assume_unique() };
        self.mut_magazine().charge_bullet(bullet_area);
    }
}

// https://github.com/godot-rust/godot-rust/blob/master/examples/dodge_the_creeps/src/main_scene.rs
fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Option<Ref<Root, Unique>>
where
    Root: gdnative::object::GodotObject<RefKind = ManuallyManaged> + SubClass<Node>,
{
    let scene = unsafe { scene.assume_safe() };

    let instance = scene.instance(PackedScene::GEN_EDIT_STATE_DISABLED)?;
    // .expect("should be able to instance scene");

    let instance = unsafe { instance.assume_unique() };

    Some(instance.try_cast::<Root>().ok()?)
    // .expect("root node type should be correct")
}

#[derive(NativeClass)]
#[inherit(Area)]
#[register_with(Self::register_signals)]
struct Bullet {
    #[property(default = 5.0)]
    speed: f32,
    flying: bool,
}

#[gdnative::methods]
impl Bullet {
    fn new(_owner: &Area) -> Self {
        Self {
            speed: 5.0,
            flying: false,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "collect",
            args: &[SignalArgument {
                name: "bullet",
                default: Variant::new(),
                export_info: ExportInfo::new(VariantType::Object),
                usage: PropertyUsage::DEFAULT,
            }],
        });
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        let player_root = unsafe {
            owner
                .get_node("/root/stage/PlayerRoot")
                .unwrap()
                .assume_safe()
        };
        owner
            .connect(
                "collect",
                player_root,
                "collect_bullet",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
    }

    #[export]
    fn _physics_process(&self, owner: &Area, delta: f64) {
        let d = Vector3::new(0.0, 0.0, -self.speed * delta as f32);
        owner.translate(d);
    }

    #[export]
    fn hit(&mut self, owner: &Area, _area: Variant) {
        if !self.flying {
            return;
        }
        unsafe {
            owner.call_deferred("cartridge_fallen", &[]);
        }
        self.flying = false;
    }

    #[export]
    fn cartridge_fallen(&self, owner: &Area) {
        unsafe {
            let parent = owner.get_parent().unwrap().assume_safe();
            parent.remove_child(owner.assume_shared());
        }
        let area = unsafe { owner.assume_unique() };
        owner.emit_signal("collect", &[Variant::from_object(area)]);
    }
}

#[derive(NativeClass)]
#[inherit(Area)]
#[register_with(Self::register_signals)]
struct Alien {
    alive: bool,
    #[property(default = 5.0)]
    speed: f32,
    setted_speed: f32,
    env: Env,
    attack_sound: Option<Ref<AudioStreamPlayer, Unique>>,
    collision_shape: Option<Ref<CollisionShape, Unique>>,
    alien_spatial: Option<Ref<Spatial, Unique>>,
    frag: Option<Ref<CPUParticles, Unique>>,
    destruct_timer: Option<Ref<Timer, Unique>>,
    alien_left_limit: f32,
    alien_right_limit: f32,
    alien_up_limit: f32,
    alien_down_limit: f32,
    process_pattern: fn(&mut Self, &Area, f64),
}

#[gdnative::methods]
impl Alien {
    fn new(_owner: &Area) -> Self {
        Self {
            alive: false,
            speed: 5.0,
            setted_speed: 5.0,
            env: Env::new(),
            alien_left_limit: 0.0,
            alien_right_limit: 0.0,
            alien_up_limit: 0.0,
            alien_down_limit: 0.0,
            attack_sound: None,
            collision_shape: None,
            alien_spatial: None,
            frag: None,
            destruct_timer: None,
            process_pattern: Self::default_process_pattern,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "collect_alien",
            args: &[SignalArgument {
                name: "alien",
                default: Variant::new(),
                export_info: ExportInfo::new(VariantType::Object),
                usage: PropertyUsage::DEFAULT,
            }],
        });
        builder.add_signal(Signal {
            name: "beated_alien",
            args: &[],
        });
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        self.alive = true;
        self.setted_speed = self.speed;
        self.env.init(owner);
        // // godot_print!("_ready@Alien {}", env!("CARGO_PKG_VERSION"));

        let stage = unsafe { owner.get_node("/root/stage").unwrap().assume_safe() };
        owner
            .connect(
                "collect_alien",
                stage,
                "collect_alien",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
        owner
            .connect(
                "beated_alien",
                stage,
                "alien_beated",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        let a_left_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_left_limit")
                .unwrap()
        };
        let a_right_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_right_limit")
                .unwrap()
        };
        let a_up_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_up_limit")
                .unwrap()
        };
        let a_down_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_down_limit")
                .unwrap()
        };

        self.alien_left_limit = a_left_limit_node.translation().x;
        self.alien_right_limit = a_right_limit_node.translation().x;
        self.alien_up_limit = a_up_limit_node.translation().z;
        self.alien_down_limit = a_down_limit_node.translation().z;

        unsafe {
            self.frag = Some(
                owner
                    .get_node_as::<CPUParticles>("frag")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
            self.attack_sound = Some(
                owner
                    .get_node_as::<AudioStreamPlayer>("attackSound")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
            self.alien_spatial = Some(
                owner
                    .get_node_as::<Spatial>("alien1")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
            self.collision_shape = Some(
                owner
                    .get_node_as::<CollisionShape>("CollisionShape")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
            self.destruct_timer = Some(
                owner
                    .get_node_as::<Timer>("DestructTimer")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
        }
    }

    pub fn reset(&mut self, _owner: &Area) {
        self.alive = true;
        self.speed = self.setted_speed;
        if let Some(s) = self.alien_spatial.as_ref() {
            s.set_visible(true);
            // godot_print!("reset visibility");
        }
        if let Some(c) = self.collision_shape.as_ref() {
            c.set_disabled(false);
            // godot_print!("reset collision");
        }
    }

    pub fn speed_up(&mut self, _owner: &Area, times: f32) {
        self.speed *= times;
    }

    pub fn set_process(&mut self, process: fn(&mut Self, &Area, f64)) {
        // godot_print!("set_process");
        self.process_pattern = process;
    }

    #[export]
    fn _physics_process(&mut self, owner: &Area, delta: f64) {
        (self.process_pattern)(self, owner, delta);
    }

    fn gone_far_away(&self, owner: &Area) -> bool {
        let pos = owner.translation();
        pos.x < self.alien_left_limit
            || pos.x > self.alien_right_limit
            || pos.z < self.alien_up_limit
            || pos.z > self.alien_down_limit
    }

    pub fn default_process_pattern(&mut self, owner: &Area, delta: f64) {
        let d = Vector3::new(-self.speed * delta as f32, 0.0, 0.0);
        owner.translate(d);

        if (owner.translation().x < self.env.left_limit && self.speed < 0.0)
            || (owner.translation().x > self.env.right_limit && self.speed > 0.0)
        {
            self.speed *= -1.0;
        }
    }

    pub fn invasion_pattern(&mut self, owner: &Area, delta: f64) {
        let d = Vector3::new(0.0, 0.0, -self.speed * delta as f32);
        owner.translate(d);

        if self.gone_far_away(owner) {
            self.destruct(owner);
        }
    }

    #[export]
    fn shooted(&mut self, owner: &Area, area: Variant) {
        if !self.alive {
            return;
        }

        let area: TRef<Area, _> = unsafe { area.try_to_object().unwrap().assume_safe() };
        if area.collision_layer() != 4 {
            return;
        }

        // godot_print!("shooted@Alien");

        self.speed = 0.0;
        self.alive = false;
        self.frag.as_ref().unwrap().set_emitting(true);
        self.attack_sound.as_ref().unwrap().play(0.0);
        self.alien_spatial.as_ref().unwrap().set_visible(false);
        // self.collision_shape.as_ref().unwrap().set_disabled(true);
        self.destruct_timer.as_ref().unwrap().start(0.0);

        owner.emit_signal("beated_alien", &[]);

        unsafe {
            owner.call_deferred("disable_collision", &[]);
        }
    }

    #[export]
    fn disable_collision(&mut self, _owner: &Area) {
        // godot_print!("disable_collision@Alien");
        self.collision_shape.as_ref().unwrap().set_disabled(true);
        // godot_print!("end disable_collision@Alien");
    }

    #[export]
    fn destruct(&mut self, owner: &Area) {
        self.alive = false;
        unsafe {
            owner.call_deferred("return_to_base", &[]);
        }
    }

    #[export]
    fn return_to_base(&self, owner: &Area) {
        // godot_print!("return_to_base");
        unsafe {
            let parent = owner.get_parent().unwrap().assume_safe();
            parent.remove_child(owner.assume_shared());
        }
        let area = unsafe { owner.assume_unique() };
        owner.emit_signal("collect_alien", &[Variant::from_object(area)]);
        // godot_print!("end collect alien");
    }
}

/*
impl BulletTarget for Alien {
    fn shooted(&self, owner: &Node, _damage: u32) {
        unsafe {
            owner
                .get_node_as::<AudioStreamPlayer>("../attackSound")
                .unwrap()
                .play(0.0);
        }

        unsafe {
            owner.assume_unique().queue_free();
        }

        /* // Copilot君が天才的なコード吐いた...多分healthは上に表示されるHPバー
        unsafe {
            owner.get_node_as::<Spatial>("../health").unwrap().set_scale(
                Vector3::new(
                    owner.get_node_as::<Spatial>("../health").unwrap().scale().x - damage as f32,
                    1.0,
                    1.0,
                ),
            );
        }

        if owner.get_node_as::<Spatial>("../health").unwrap().scale().x <= 0.0 {
            unsafe {
                owner.get_node_as::<AudioStreamPlayer>("../deathSound").unwrap().play(0.0);
            }

            unsafe {
                owner.assume_unique().queue_free();
            }
        }
        */
    }
}
*/

fn init(handle: InitHandle) {
    handle.add_class::<Stage>();
    handle.add_class::<Player>();
    handle.add_class::<Bullet>();
    handle.add_class::<Alien>();
}

godot_init!(init);
