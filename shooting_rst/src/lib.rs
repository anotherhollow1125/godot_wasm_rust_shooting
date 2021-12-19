// use gdnative::api::RigidBody;
use gdnative::api::{Area, AudioStreamPlayer, CPUParticles, CollisionShape, RandomNumberGenerator};
use gdnative::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::PI;

pub fn rotate_xz_vec3(v: Vector3, theta: f32) -> Vector3 {
    let c = theta.cos();
    let s = theta.sin();
    Vector3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}

pub const BARRAGE_PLANS: [fn(Vector3) -> Vec<Vector3>; 7] = [
    // simple barrage
    |dir| -> Vec<Vector3> { vec![dir] },
    // simple barrage
    |dir| -> Vec<Vector3> { vec![dir] },
    // simple barrage
    |dir| -> Vec<Vector3> { vec![dir] },
    // simple barrage
    |dir| -> Vec<Vector3> { vec![dir] },
    // three way barrage
    |dir| -> Vec<Vector3> {
        let mut v = vec![dir];
        v.push(rotate_xz_vec3(dir, PI / 3.0));
        v.push(rotate_xz_vec3(dir, -PI / 3.0));
        v
    },
    // three way barrage
    |dir| -> Vec<Vector3> {
        let mut v = vec![dir];
        v.push(rotate_xz_vec3(dir, PI / 3.0));
        v.push(rotate_xz_vec3(dir, -PI / 3.0));
        v
    },
    // all range barrage
    |dir| -> Vec<Vector3> {
        let mut v = vec![];
        for i in 0..6 {
            v.push(rotate_xz_vec3(dir, PI * i as f32 / 3.0));
        }
        v
    },
];

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

struct AlienEnv {
    left_limit: f32,
    right_limit: f32,
    up_limit: f32,
    down_limit: f32,
}

impl AlienEnv {
    pub fn new() -> Self {
        AlienEnv {
            left_limit: 0.0,
            right_limit: 0.0,
            up_limit: 0.0,
            down_limit: 0.0,
        }
    }

    pub fn init(&mut self, owner: &Node) {
        let left_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_left_limit")
                .unwrap()
        };
        let right_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_right_limit")
                .unwrap()
        };
        let up_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_up_limit")
                .unwrap()
        };
        let down_limit_node = unsafe {
            owner
                .get_node_as::<Spatial>("/root/stage/alien_down_limit")
                .unwrap()
        };

        self.left_limit = left_limit_node.translation().x;
        self.right_limit = right_limit_node.translation().x;
        self.up_limit = up_limit_node.translation().z;
        self.down_limit = down_limit_node.translation().z;
    }

    pub fn gone_far_away(&self, owner: &Spatial) -> bool {
        let pos = owner.translation();
        pos.x < self.left_limit
            || pos.x > self.right_limit
            || pos.z < self.up_limit
            || pos.z > self.down_limit
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

    alien_spawn_timer: Option<Ref<Timer, Unique>>,
    rng: Option<Ref<RandomNumberGenerator, Unique>>,

    env: Env,

    #[property(default = 3)]
    player_life: i32,
    default_player_life: i32,
    beated_alien_num: i32,
    #[property(default = 1.0)]
    stage_heat: f32,

    #[property(default = 200)]
    alibullet_num: i32,
    #[property]
    alibullet_scene: Ref<PackedScene>,
    alibullets_magazine: Option<Magazine<AlienBullet>>,

    bgm: Option<Ref<AudioStreamPlayer, Unique>>,
    extend_sound: Option<Ref<AudioStreamPlayer, Unique>>,
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
            default_player_life: 3,
            beated_alien_num: 0,
            stage_heat: 1.0,

            alibullet_num: 200,
            alibullet_scene: PackedScene::new().into_shared(),
            alibullets_magazine: None,

            bgm: None,
            extend_sound: None,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        // godot_print!("register_signals@Stage");

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

        builder.add_signal(Signal {
            name: "set_score",
            args: &[SignalArgument {
                name: "score",
                default: Variant::from_i64(0),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
        builder.add_signal(Signal {
            name: "set_remain",
            args: &[SignalArgument {
                name: "remain",
                default: Variant::from_i64(0),
                export_info: ExportInfo::new(VariantType::I64),
                usage: PropertyUsage::DEFAULT,
            }],
        });
        builder.add_signal(Signal {
            name: "game_over",
            args: &[],
        });

        // godot_print!("end register_signals@Stage");
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
        // godot_print!("start _ready@Stage");
        self.default_player_life = self.player_life;
        let rng = RandomNumberGenerator::new();
        rng.randomize();
        let alien_spawn_timer = unsafe {
            let t = owner.get_node_as::<Timer>("alien_spawn_timer").unwrap();
            // t.start(rng.randf_range(0.5, 1.0));
            t.claim().assume_unique()
        };
        self.rng = Some(rng);
        self.alien_spawn_timer = Some(alien_spawn_timer);
        self.aliens_magazine = Some(Magazine::new(&self.alien_scene, self.alien_num as usize));
        self.env.init(owner);

        self.alibullets_magazine = Some(Magazine::new(
            &self.alibullet_scene,
            self.alibullet_num as usize,
        ));

        let bgm = unsafe {
            let bgm = owner.get_node_as::<AudioStreamPlayer>("BGM").unwrap();
            // bgm.play(0.0);
            bgm.claim().assume_unique()
        };
        self.bgm = Some(bgm);
        self.extend_sound = Some(unsafe {
            owner
                .get_node_as::<AudioStreamPlayer>("extend")
                .unwrap()
                .claim()
                .assume_unique()
        });

        // self.start_game(owner);

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

        // godot_print!("stage ready");
    }

    #[export]
    fn start_game(&mut self, owner: &Node) {
        godot_print!("game start"); // info log
        self.set_player_life(owner, self.default_player_life);
        self.set_beated_alien_num(owner, 0);
        self.set_heat(owner, 1.0);
        let rng = match self.rng.as_ref() {
            Some(rng) => rng,
            None => return,
        };
        match self.alien_spawn_timer.as_ref() {
            Some(timer) => timer.start(rng.randf_range(0.5, 1.0)),
            None => return,
        }
        match self.bgm.as_ref() {
            Some(bgm) => bgm.play(0.0),
            None => return,
        }

        owner.emit_signal("restart_player", &[]);
    }

    fn end_game(&mut self, owner: &Node) {
        godot_print!("game end"); // info log
        match self.alien_spawn_timer.as_ref() {
            Some(timer) => timer.stop(),
            None => return,
        }
        match self.bgm.as_ref() {
            Some(bgm) => bgm.stop(),
            None => return,
        }
        owner.emit_signal("game_over", &[]);
    }

    fn set_heat(&mut self, owner: &Node, heat: f32) {
        self.stage_heat = heat;
        owner.emit_signal("speed_up", &[Variant::from_f64(heat as f64)]);
    }

    fn set_player_life(&mut self, owner: &Node, val: i32) {
        self.player_life = val;
        owner.emit_signal("set_remain", &[Variant::from_i64(val as i64)]);
    }

    fn set_beated_alien_num(&mut self, owner: &Node, val: i32) {
        self.beated_alien_num = val;
        owner.emit_signal("set_score", &[Variant::from_i64(val as i64)]);
    }

    #[allow(unused)]
    fn get_alien_magazine(&self) -> &Magazine<Alien> {
        self.aliens_magazine.as_ref().unwrap()
    }

    fn mut_alien_magazine(&mut self) -> &mut Magazine<Alien> {
        self.aliens_magazine.as_mut().unwrap()
    }

    #[allow(unused)]
    fn get_alibul_magazine(&self) -> &Magazine<AlienBullet> {
        self.alibullets_magazine.as_ref().unwrap()
    }

    fn mut_alibul_magazine(&mut self) -> &mut Magazine<AlienBullet> {
        self.alibullets_magazine.as_mut().unwrap()
    }

    #[export]
    fn spawn_alien(&mut self, owner: &Node) {
        // godot_print!("spawn_alien");

        let (pos, process, dir, interval): (Vector3, AlienProcessPattern, Vector3, f64) =
            match self.rng {
                Some(ref mut rng) => {
                    let interval = rng.randf_range(0.5, 3.0) / self.stage_heat as f64;

                    let (pos, process, dir) = if rng.randi_range(0, 1) == 0 {
                        let spawn_x = rng
                            .randf_range(self.env.left_limit as f64, self.env.right_limit as f64);
                        (
                            Vector3::new(spawn_x as f32, 0.0, -27.0),
                            AlienProcessPattern::Invasion,
                            Vector3::new(0.0, 0.0, 1.0),
                        )
                    } else {
                        let mid = ((self.env.up_limit + self.env.down_limit) / 2.0) as f64;
                        let spawn_z = rng.randf_range(self.env.up_limit as f64, mid);
                        let t = [-1.0, 1.0][rng.randi_range(0, 1) as usize];
                        let x = t * 22.0;
                        let dir = Vector3::new(-t, 0.0, 0.0);
                        (
                            Vector3::new(x, 0.0, spawn_z as f32),
                            AlienProcessPattern::Dir,
                            dir,
                        )
                    };

                    (pos, process, dir, interval)
                }
                None => return,
            };

        let aliens_scene = match self.mut_alien_magazine().hammer() {
            Some(a) => {
                a.map_mut(|t, owner| {
                    t.reset(&owner);
                    t.speed_up(self.stage_heat);
                    t.set_dir_change_span_random(1.0 / self.stage_heat);
                    t.set_fire_span_random(1.0 / self.stage_heat);
                    t.set_dir(dir);
                    t.set_process(process);
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

        // let pos = Vector3::new(spawn_x as f32, 0.0, -27.0);
        // let pos = Vector3::new(spawn_x as f32, 0.0, -10.0);

        aliens_scene.set_translation(pos);
        owner.add_child(aliens_scene, false);
        self.alien_spawn_timer
            .as_ref()
            .unwrap()
            .set_wait_time(interval);

        // godot_print!("spawn_alien_beep");
    }

    #[export]
    fn collect_alien(&mut self, _owner: &Node, alien_var: Variant) {
        // godot_print!("stage collect alien");
        let alien_area: Ref<Area, Unique> =
            unsafe { alien_var.try_to_object().unwrap().assume_unique() };
        self.mut_alien_magazine().charge_bullet(alien_area);
        /*
        godot_print!(
            "end stage collect alien: {}",
            self.get_alien_magazine().get_left_num()
        );
        */
    }

    #[export]
    fn collect_alien_bullet(&mut self, _owner: &Node, alien_bullet_var: Variant) {
        // godot_print!("stage collect alien bullet");
        let alien_bullet_area: Ref<Area, Unique> =
            unsafe { alien_bullet_var.try_to_object().unwrap().assume_unique() };
        self.mut_alibul_magazine().charge_bullet(alien_bullet_area);
        /*
        godot_print!(
            "end stage collect alien bullet: {}",
            self.get_alibul_magazine().get_left_num()
        );
        */
    }

    #[export]
    fn player_beated(&mut self, owner: &Node) {
        // self.player_life -= 1;
        self.set_player_life(owner, self.player_life - 1);
        if self.player_life > 0 {
            godot_print!("Player Restart"); // info log
            unsafe {
                owner.call_deferred("player_restart", &[]);
            }
        } else {
            self.end_game(owner);
        }
    }

    #[export]
    fn player_restart(&mut self, owner: &Node) {
        owner.emit_signal("restart_player", &[]);
    }

    #[export]
    fn alien_beated(&mut self, owner: &Node) {
        // self.beated_alien_num += 1;
        self.set_beated_alien_num(owner, self.beated_alien_num + 1);
        if self.beated_alien_num % 10 == 0 {
            // self.stage_heat *= 1.1;
            // owner.emit_signal("speed_up", &[Variant::from_f64(self.stage_heat as f64)]);
            self.set_heat(owner, self.stage_heat * 1.1);
            godot_print!("Stage Heat: {}", self.stage_heat); // info log
        }
        if self.beated_alien_num % 20 == 0 {
            // self.player_life += 1;
            self.set_player_life(owner, self.player_life + 1);
            if let Some(sound) = self.extend_sound.as_ref() {
                sound.play(0.0);
            }
            godot_print!("Player Life Extended: {}", self.player_life); // info log
        }
        // godot_print!("beated_alien_num: {}", self.beated_alien_num);
    }

    #[export]
    fn alien_fire(
        &mut self,
        owner: &Node,
        pos: Variant,
        dir: Variant,
        speed: Variant,
        kind: Variant,
    ) {
        let pos: Vector3 = pos.to_vector3();
        let dir: Vector3 = dir.to_vector3();
        let speed: f64 = speed.to_f64();
        let kind: i64 = kind.to_i64();

        let dirs = BARRAGE_PLANS[(kind % BARRAGE_PLANS.len() as i64) as usize](dir);

        // godot_print!("alien_fire: {:?} {:?} {:?} {:?}", pos, dir, speed, kind);

        for dir in dirs {
            let bullet_scene = match self.mut_alibul_magazine().hammer() {
                Some(a) => {
                    a.map_mut(|t, _owner| {
                        t.set_speed(speed as f32);
                        // godot_print!("dir: {:?}", dir);
                        t.set_dir(dir);
                        t.flying = true;
                    })
                    .ok();
                    a.into_base()
                }
                None => {
                    // godot_print!("No bullets left");
                    return;
                }
            };
            bullet_scene.set_translation(pos);
            owner.add_child(bullet_scene, false);
        }
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
        // godot_print!("register_signals@Player");

        builder.add_signal(Signal {
            name: "player_beated",
            args: &[],
        });

        // godot_print!("end register_signals@Player");
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        // godot_print!("start _ready@Player");
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

        // self.reset(owner);
        self.alive = true; // for demonstration

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

        godot_print!("_ready@Player {}", env!("CARGO_PKG_VERSION")); // info log
    }

    #[export]
    fn reset(&mut self, owner: &Area) {
        // godot_print!("reset@Player");
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

        // godot_print!("crash@Player");

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
        // godot_print!("register_signals@Bullet");

        builder.add_signal(Signal {
            name: "collect",
            args: &[SignalArgument {
                name: "bullet",
                default: Variant::new(),
                export_info: ExportInfo::new(VariantType::Object),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        // godot_print!("end register_signals@Bullet");
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

        // godot_print!("ready@Bullet");
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

#[derive(PartialEq, Eq, Clone, Copy)]
enum AlienProcessPattern {
    Default,
    Invasion,
    Dir,
}

#[derive(NativeClass)]
#[inherit(Area)]
#[register_with(Self::register_signals)]
struct Alien {
    alive: bool,
    #[property(default = 5.0)]
    speed: f32,
    direction: Vector3,
    setted_speed: f32,
    env: Env,
    attack_sound: Option<Ref<AudioStreamPlayer, Unique>>,
    collision_shape: Option<Ref<CollisionShape, Unique>>,
    alien_spatial: Option<Ref<Spatial, Unique>>,
    frag: Option<Ref<CPUParticles, Unique>>,
    destruct_timer: Option<Ref<Timer, Unique>>,
    alien_env: AlienEnv,
    // process_pattern: fn(&mut Self, &Area, f64),
    process_pattern: AlienProcessPattern,
    rng: Option<Ref<RandomNumberGenerator, Unique>>,
    change_dir_timer: Option<Ref<Timer, Unique>>,

    fire_timer: Option<Ref<Timer, Unique>>,
    player_root: Option<Ref<Area, Shared>>,

    #[property(default = 5.0)]
    default_min_fire_interval: f32,
    #[property(default = 10.0)]
    default_max_fire_interval: f32,
}

#[gdnative::methods]
impl Alien {
    fn new(_owner: &Area) -> Self {
        Self {
            alive: false,
            speed: 5.0,
            direction: Vector3::new(0.0, 0.0, -1.0),
            setted_speed: 5.0,
            env: Env::new(),
            alien_env: AlienEnv::new(),
            attack_sound: None,
            collision_shape: None,
            alien_spatial: None,
            frag: None,
            destruct_timer: None,
            process_pattern: AlienProcessPattern::Default,
            rng: None,
            change_dir_timer: None,

            fire_timer: None,
            player_root: None,

            default_min_fire_interval: 5.0,
            default_max_fire_interval: 10.0,
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        // godot_print!("register_signals@Alien");

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
        builder.add_signal(Signal {
            name: "alien_fire",
            args: &[
                SignalArgument {
                    name: "pos",
                    default: Variant::from_vector3(&Vector3::new(0.0, 0.0, 0.0)),
                    export_info: ExportInfo::new(VariantType::Vector3),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "dir",
                    default: Variant::from_vector3(&Vector3::new(0.0, 0.0, 0.0)),
                    export_info: ExportInfo::new(VariantType::Vector3),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "speed",
                    default: Variant::from_f64(0.0),
                    export_info: ExportInfo::new(VariantType::F64),
                    usage: PropertyUsage::DEFAULT,
                },
                SignalArgument {
                    name: "bullet_type",
                    default: Variant::from_i64(0),
                    export_info: ExportInfo::new(VariantType::I64),
                    usage: PropertyUsage::DEFAULT,
                },
            ],
        });

        // godot_print!("end register_signals@Alien");
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        // godot_print!("start _ready@Alien");
        let rng = RandomNumberGenerator::new();
        rng.randomize();
        self.rng = Some(rng);

        self.alive = true;
        self.setted_speed = self.speed;
        // self.direction = Vector3::new(0.0, 0.0, 1.0);
        self.env.init(owner);

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
        owner
            .connect(
                "alien_fire",
                stage,
                "alien_fire",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        self.alien_env.init(owner);

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
            self.change_dir_timer = Some(
                owner
                    .get_node_as::<Timer>("ChangeDirTimer")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );
            self.fire_timer = Some(
                owner
                    .get_node_as::<Timer>("FireTimer")
                    .unwrap()
                    .claim()
                    .assume_unique(),
            );

            self.player_root = Some(
                owner
                    .get_node_as::<Area>("/root/stage/PlayerRoot")
                    .unwrap()
                    .claim(),
            );
        }

        self.set_dir_change_span_random(1.0);
        self.set_fire_span_random(1.0);
        // godot_print!("_ready@Alien {}", env!("CARGO_PKG_VERSION"));
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

    pub fn speed_up(&mut self, times: f32) {
        self.speed *= times;
    }

    pub fn set_dir(&mut self, dir: Vector3) {
        // godot_print!("pre dir : {:?}", dir);
        self.direction = dir.normalize();
        if !self.direction.is_finite() {
            self.direction = Vector3::new(0.0, 0.0, -1.0);
        }
        // godot_print!("post dir : {:?}", self.direction);
    }

    #[export]
    fn change_dir_random(&mut self, _owner: &Area) {
        // godot_print!("change_dir_random");

        if self.process_pattern != AlienProcessPattern::Dir {
            return;
        }

        let rng = match self.rng.as_ref() {
            Some(r) => r,
            None => return,
        };
        let theta = rng.randf_range((-PI / 4.0) as _, (PI / 4.0) as _);
        self.set_dir(rotate_xz_vec3(self.direction, theta as _));
    }

    pub fn set_dir_change_span_random(&mut self, ratio: f32) {
        let span = if let Some(rng) = self.rng.as_ref() {
            rng.randf_range(0.0, 3.0)
        } else {
            1.0
        };
        if let Some(t) = self.change_dir_timer.as_ref() {
            t.set_wait_time(span * ratio as f64);
            // godot_print!("reset change_dir_timer");
        }
    }

    pub fn set_fire_span_random(&mut self, ratio: f32) {
        // godot_print!("@@ set_fire_span_random");
        let span = if let Some(rng) = self.rng.as_ref() {
            rng.randf_range(
                self.default_min_fire_interval as _,
                self.default_max_fire_interval as _,
            )
        } else {
            1.0
        };
        if let Some(t) = self.fire_timer.as_ref() {
            // godot_print!("set to {}", span * ratio as f64);
            t.set_wait_time(span * ratio as f64);
            // godot_print!("reset fire_timer");
        }
    }

    #[export]
    fn timer_start(&self, _owner: &Area) {
        if let Some(t) = self.change_dir_timer.as_ref() {
            // godot_print!("change_dir_timer_start {}", t.wait_time());
            t.start(0.0);
            // godot_print!("start change_dir_timer");
        }
        if let Some(t) = self.fire_timer.as_ref() {
            // godot_print!("fire_timer_start {}", t.wait_time());
            t.start(0.0);
            // godot_print!("start fire_timer");
        }
    }

    #[export]
    fn _on_alien_tree_entered(&self, owner: &Area) {
        // godot_print!("_on_alien_tree_entered");
        unsafe {
            owner.call_deferred("timer_start", &[]);
        }
    }

    #[export]
    fn timer_stop(&self, _owner: &Area) {
        // godot_print!("change_dir_timer_stop");
        if let Some(t) = self.change_dir_timer.as_ref() {
            t.stop();
            // godot_print!("stop change_dir_timer");
        }
        if let Some(t) = self.fire_timer.as_ref() {
            t.stop();
            // godot_print!("stop fire_timer");
        }
    }

    pub fn set_process(&mut self, process: AlienProcessPattern) {
        // godot_print!("set_process");
        self.process_pattern = process;
    }

    #[export]
    fn _physics_process(&mut self, owner: &Area, delta: f64) {
        let f: fn(&mut Alien, &Area, f64) = match self.process_pattern {
            AlienProcessPattern::Default => Alien::default_process_pattern,
            AlienProcessPattern::Invasion => Alien::invasion_pattern,
            AlienProcessPattern::Dir => Alien::dir_pattern,
        };
        (f)(self, owner, delta);
    }

    pub fn default_process_pattern(&mut self, owner: &Area, delta: f64) {
        let d = Vector3::new(self.speed * delta as f32, 0.0, 0.0);
        owner.translate(d);

        if (owner.translation().x < self.env.left_limit && self.speed < 0.0)
            || (owner.translation().x > self.env.right_limit && self.speed > 0.0)
        {
            self.speed *= -1.0;
        }
    }

    pub fn invasion_pattern(&mut self, owner: &Area, delta: f64) {
        let d = Vector3::new(0.0, 0.0, self.speed * delta as f32);
        owner.translate(d);

        if self.alien_env.gone_far_away(owner) {
            self.destruct(owner);
        }
    }

    pub fn dir_pattern(&mut self, owner: &Area, delta: f64) {
        let d = self.direction * self.speed * delta as f32;
        owner.translate(d);

        if self.alien_env.gone_far_away(owner) {
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
        // self.change_dir_timer_stop(owner);
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

    #[export]
    fn fire(&self, owner: &Area) {
        if !self.alive {
            return;
        }

        let rng = match self.rng.as_ref() {
            Some(rng) => rng,
            None => return,
        };

        // godot_print!("fire@Alien");
        let pos = owner.translation();
        let dir = if rng.randi_range(0, 1) == 0 {
            // return its direction
            self.direction
        } else {
            let player_pos = unsafe {
                self.player_root
                    .as_ref()
                    .unwrap()
                    .assume_safe()
                    .translation()
            };
            // return self to player
            (player_pos - pos).normalize()
        };
        // godot_print!("{:?} | fire dir {:?}", self.direction, dir);
        let speed = self.speed * 1.3;
        let max_index = BARRAGE_PLANS.len() - 1;
        let kind = rng.randi_range(0, max_index as i64);

        owner.emit_signal(
            "alien_fire",
            &[
                Variant::from_vector3(&pos),
                Variant::from_vector3(&dir),
                Variant::from_f64(speed as f64),
                Variant::from_i64(kind),
            ],
        );
    }
}

#[derive(NativeClass)]
#[inherit(Area)]
#[register_with(Self::register_signals)]
struct AlienBullet {
    #[property(default = 5.0)]
    speed: f32,
    direction: Vector3,
    flying: bool,

    alien_env: AlienEnv,
}

#[gdnative::methods]
impl AlienBullet {
    fn new(_owner: &Area) -> Self {
        Self {
            speed: 5.0,
            direction: Vector3::new(0.0, 0.0, 1.0),
            flying: false,

            alien_env: AlienEnv::new(),
        }
    }

    fn register_signals(builder: &ClassBuilder<Self>) {
        // godot_print!("register_signals@AlienBullet");

        builder.add_signal(Signal {
            name: "collect_alien_bullet",
            args: &[SignalArgument {
                name: "bullet",
                default: Variant::new(),
                export_info: ExportInfo::new(VariantType::Object),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        // godot_print!("end register_signals@AlienBullet");
    }

    #[export]
    fn _ready(&mut self, owner: &Area) {
        let stage = unsafe { owner.get_node("/root/stage/").unwrap().assume_safe() };
        owner
            .connect(
                "collect_alien_bullet",
                stage,
                "collect_alien_bullet",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        self.alien_env.init(owner);

        // godot_print!("ready@AlienBullet");
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_dir(&mut self, dir: Vector3) {
        self.direction = dir.normalize();
        if !self.direction.is_finite() {
            self.direction = Vector3::new(0.0, 0.0, 1.0);
        }
    }

    #[export]
    fn _physics_process(&mut self, owner: &Area, delta: f64) {
        let d = self.direction * self.speed * delta as f32;
        owner.translate(d);

        if self.alien_env.gone_far_away(owner) {
            self.destruct(owner);
        }
    }

    #[export]
    fn hit(&mut self, owner: &Area, _: Variant) {
        self.destruct(owner);
    }

    fn destruct(&mut self, owner: &Area) {
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
        owner.emit_signal("collect_alien_bullet", &[Variant::from_object(area)]);
    }
}

fn init(handle: InitHandle) {
    // godot_print!("beep1");
    handle.add_class::<Stage>();
    // godot_print!("beep2");
    handle.add_class::<Player>();
    // godot_print!("beep3");
    handle.add_class::<Bullet>();
    // godot_print!("beep4");
    handle.add_class::<Alien>();
    // godot_print!("beep5");
    handle.add_class::<AlienBullet>();
}

godot_init!(init);
