//! Rust/Wasm reconstruction with Ghidra-verified original data and contact rules. This is not a clean-room implementation.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
const TILE: f64 = 32.;
mod original_stats;
pub mod original_combat;
mod original_grenades;
mod original_initiative;
mod original_terrain;
mod original_motion;
mod original_upstream_reactions;
mod original_clock;
mod native_math;
mod original_fire;
mod original_formation;
mod original_ai;
mod original_bullets;
mod original_projectiles;
mod original_fragment_blast;
mod original_bridge_sound;
mod original_hardware;
mod original_groups;
mod original_contacts;
mod native_ai_scoring;
mod native_ai_tasks;
mod original_strategy;
mod original_transport;
mod original_transport_game;
mod original_idle;
mod original_reactions;
mod original_animation_game;
mod original_route;
mod original_collision;
mod original_dynamic_collision;
mod original_route_game;
mod original_route_pool;
mod original_road_scratch;
mod original_path_contact;
mod original_hardware_point;
mod original_audio;
mod original_listener_audio;
mod original_unit_preamble;
mod original_retired_unit_preamble;
mod original_vehicle_spacing;
mod original_building_display;
mod original_building_fragments;
mod original_flag_display;
mod original_display;
mod original_factory;
mod original_factory_game;
mod original_factory_policy_game;
mod original_factory_placement;
mod original_factory_placement_game;
mod original_death;
mod original_death_game;
mod original_ambient;
mod original_ambient_game;
mod original_point_fire;
mod original_entry;
mod original_entry_game;
mod original_smoke20;
mod original_bullet_impacts20;
mod original_ambient_emitters;
mod original_ambient_animations;
mod original_lava;
mod original_lava_game;
mod original_group_lifecycle;
mod original_group_game;
mod original_orders_game;
mod original_selection;
mod original_selection_game;
mod original_attack_click;
mod original_attack_click_game;
mod original_attack_command_game;
mod original_attack_order;
mod original_attack_approach;
mod original_feedback;
mod original_ground_fire;
mod original_walk;
mod original_move_order;
mod original_projectile_warning;
mod original_world_contact;
mod original_retired_workshop;
mod original_world_contact_game;
mod native_weapon_data;
mod terrain;
mod walls;
mod workshops;
mod pursuit;
mod original_pursuit;
mod orders;
mod gun_placement;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tile_size: f64,
    pub world: usize,
    pub tiles: Vec<u8>,
    pub territory: Vec<usize>,
    pub bridge_base: Vec<i32>,
    pub bridge_hp: Vec<f64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bridge_max_hp: Vec<f64>,
    pub seed: u32,
    pub revision: u32,
    #[serde(default)] pub original:bool,
    #[serde(default)] pub original_tiles:Vec<u16>,
    #[serde(default)] pub terrain_codes:Vec<u8>,
    #[serde(default)] pub water_tile:u16,
    #[serde(default)] pub wall_tile_lookup:Value,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_road_graph:Option<original_route::RoadGraph>,
    #[serde(default)] pub native_level_record:i16,
    #[serde(default,skip_serializing)] pub native_terrain_plane:Vec<u8>,
    #[serde(default,skip_serializing)] pub native_tile_plane:Vec<u8>,
    #[serde(default, skip_serializing)] pub wall_ground_tiles:Vec<u16>,
    #[serde(default, skip_serializing)] pub wall_ground_codes:Vec<u8>,
    #[serde(default, skip_serializing)] pub wall_ground_classes:Vec<u8>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Sector {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub owner: i32,
    pub home: bool,
    #[serde(default="production_sector")] pub counts_for_production:bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub kind: String,
    pub team: i32,
    pub sector: usize,
    pub stars: usize,
    pub hp: f64,
    pub max_hp: f64,
    pub product: Option<String>,
    pub progress: f64,
    #[serde(default)]
    pub ready_guns: Vec<String>,
    #[serde(default)] pub gate:Option<(f64,f64)>,
    #[serde(default)] pub orientation:String,
    #[serde(default)] pub remaining_work:Option<f64>,
    /// Native building +0x36: completion hold, decremented before each factory callback.
    #[serde(default)] pub completion_ticks:u32,
    #[serde(default)] pub raw_base:Option<(f64,f64)>,
}
impl Building {
    fn alive(&self,original:bool)->bool { if original&&self.kind=="fort" { self.hp>=0. } else { self.hp>0. } }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    pub kind: String,
    pub hp: f64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: usize,
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub team: i32,
    pub hp: f64,
    pub max_hp: f64,
    pub angle: f64,
    pub aim: f64,
    pub moving: bool,
    pub squad: usize,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub formation:Option<original_formation::Follow>,
    #[serde(default)] pub motion_bonus:i8,
    #[serde(default)] pub formation_layout:u8,
    #[serde(default)] pub group_bonus_ticks:u8,
    #[serde(skip)] pub group_activity_ticks:u8,
    #[serde(skip)] pub native_group_order_pending:bool,
    #[serde(skip)] pub native_group_motion_bonus:u8,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_hardware:Option<original_hardware::State>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_contact:Option<original_contacts::State>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_idle:Option<original_idle::State>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_cover:Option<original_reactions::Cover>,
    #[serde(default)] pub native_move_frame:i16,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_transport:Option<original_transport::Animation>,
    #[serde(skip)] pub native_transport_slot:Option<usize>,
    #[serde(skip)] pub transport_carrier:Option<usize>,
    #[serde(skip)] pub transport_actor:Option<usize>,
    #[serde(skip)] pub native_route_flip:std::cell::Cell<bool>,
    #[serde(skip)] pub native_route_road_replaced:std::cell::Cell<bool>,
    #[serde(skip)] pub native_secondary_route:std::cell::Cell<bool>,
    #[serde(skip)] pub native_route_pending:std::cell::Cell<Option<(f64,f64)>>,
    #[serde(skip)] pub native_counter6e:u8,
    #[serde(default)] pub native_frame32:i16,
    #[serde(default)] pub native_timer3c:i8,
    #[serde(skip)] pub native_remaining48:i16,
    #[serde(default)] pub native_phase35:u8,
    #[serde(default)] pub native_class7c:u8,
    #[serde(default)] pub native_order5c:i8,
    #[serde(default)] pub native_range4:Option<i8>,
    #[serde(default)] pub native_counter97:i8,
    #[serde(default)] pub native_mask98:u8,
    #[serde(default)] pub native_contact6f:u8,
    #[serde(default)] pub native_contact70:u8,
    #[serde(default)] pub native_wait72:i16,
    #[serde(default)] pub native_contact_pausebc:u8,
    #[serde(default)] pub native_terrain_tag63:i8,
    #[serde(default)] pub native_target46:i16,
    #[serde(default)] pub native_counter68:i8,
    #[serde(default)] pub native_flags9c:u8,
    #[serde(default)] pub native_protectionaa:u8,
    #[serde(default)] pub native_control06:i8,
    #[serde(default)] pub native_trait7a:i8,
    #[serde(default)] pub native_field62:u8,
    #[serde(default)] pub native_cyclea0:u8,
    #[serde(default)] pub native_axis81:u8,
    #[serde(default)] pub native_speed8b:i8,
    #[serde(default)] pub native_counter9f:u8,
    #[serde(default)] pub native_cell10:i32,
    #[serde(default)] pub native_target42:i16,
    #[serde(default)] pub native_target44:i16,
    #[serde(default)] pub native_elevation76:i8,
    #[serde(default)] pub native_bucket38:u8,
    #[serde(default)] pub native_counter67:i8,
    #[serde(default)] pub native_counter66:u8,
    #[serde(default)] pub native_reload7b:i8,
    #[serde(default)] pub native_directiona2:i8,
    #[serde(default)] pub native_directiona3:i8,
    #[serde(default)] pub native_counter84:i16,
    #[serde(default)] pub native_pose82:i16,
    #[serde(default)] pub native_direction34:u8,
    #[serde(default)] pub native_field9e:i8,
    #[serde(default)] pub native_fieldb3:i8,
    #[serde(default)] pub native_footprint18:u32,
    #[serde(default)] pub native_permission1c:u32,
    #[serde(default)] pub native_pose7e:i16,
    #[serde(default)] pub native_pose80:i8,
    #[serde(default)] pub native_transition88:i16,
    #[serde(skip)] pub native_hardware_slot:Option<usize>,
    #[serde(default)] pub native_flags64:u8,
    #[serde(default)] pub native_initiative_scan:original_initiative::ScanState,
    #[serde(skip)] pub native_dodge:Option<original_animation_game::Dodge>,
    #[serde(skip)] pub native_factory_exit:Option<original_factory::FactoryExitState>,
    #[serde(default,skip_serializing_if="Option::is_none")] pub native_walk:Option<original_motion::Walk>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_slot: Option<usize>,
    #[serde(skip)] pub native_pattern20:Option<u32>,
    #[serde(skip)] pub native_word90:i16,
    /// Present only for a gun initialized in an original fort's gun socket.
    /// Ownership and crew changes do not detach the hardware from its fort.
    #[serde(default)]
    pub fort_id: Option<usize>,
    pub grenades: usize,
    pub crew: Option<Crew>,
    pub cargo: Vec<Crew>,
    pub order: String,
    pub target: Option<usize>,
    pub dest: Option<(f64, f64)>,
    pub cooldown: f64,
    #[serde(default)]
    pub original_combat: Option<original_combat::CombatState>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub native_fire: Option<original_fire::State>,
    #[serde(default)]
    pub initiative: Option<original_initiative::Initiative>,
    /// A nearby object temporarily interrupts this ordinary march.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub march_resume: Option<(f64, f64)>,
    #[serde(default)]
    pub hold_position: bool,
    #[serde(default)] pub explicit_stop:bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workshop: Option<workshops::State>,
    #[serde(skip)]
    pub(crate) pursuit: Option<pursuit::State>,
    #[serde(skip)]
    pub(crate) original_motion: Option<original_motion::Segment>,
    #[serde(skip_deserializing)]
    pub path: VecDeque<(f64, f64)>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Fx {
    pub x: f64,
    pub y: f64,
    pub age: f64,
    pub life: f64,
    pub kind: String,
    pub radius: f64,
    pub x2: f64,
    pub y2: f64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Crate {
    pub x: f64,
    pub y: f64,
    pub taken: bool,
}
#[derive(Clone, Copy)]
pub struct Stats {
    pub hp: f64,
    pub speed: f64,
    pub range: f64,
    pub damage: f64,
    pub reload: f64,
    pub seconds: f64,
    pub members: usize,
    pub stars: usize,
    pub cls: u8,
}
pub fn stats(k: &str) -> Stats {
    if let Some(d)=original_stats::data(k){return d;}
    let (hp, speed, range, damage, reload, seconds, members, stars, cls) = match k {
        "psycho" => (42., 42., 130., 6., 0.28, 30., 3, 2, 0),
        "tough" => (64., 31., 160., 30., 1.8, 42., 2, 3, 0),
        "sniper" => (36., 40., 260., 18., 1.7, 46., 2, 3, 0),
        "pyro" => (62., 37., 75., 9., 0.3, 48., 3, 4, 0),
        "laser" => (56., 43., 210., 15., 0.65, 58., 3, 5, 0),
        "jeep" => (150., 72., 150., 9., 0.48, 35., 1, 1, 1),
        "light" => (290., 46., 190., 36., 1.9, 49., 1, 1, 1),
        "medium" => (480., 35., 215., 58., 2.4, 66., 1, 2, 1),
        "heavy" => (720., 28., 230., 92., 2.9, 85., 1, 4, 1),
        "apc" => (320., 55., 0., 0., 1., 48., 1, 2, 1),
        "crane" => (250., 36., 0., 0., 1., 54., 1, 2, 1),
        "gatling" => (280., 0., 175., 8., 0.25, 39., 1, 1, 2),
        "cannon" => (430., 0., 230., 52., 2.2, 64., 1, 2, 2),
        "missile" => (400., 0., 300., 100., 3.8, 92., 1, 4, 2),
        _ => (40., 40., 120., 8., 0.75, 24., 3, 1, 0),
    };
    Stats {
        hp,
        speed,
        range,
        damage,
        reload,
        seconds,
        members,
        stars,
        cls,
    }
}
pub const TYPES: [&str; 19] = [
    "grunt", "psycho", "tough", "sniper", "pyro", "laser", "jeep", "light", "medium", "heavy",
    "apc", "crane", "gatling", "cannon", "missile", "launcher", "howitzer", "truck", "firetruck",
];
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub map: Map,
    pub sectors: Vec<Sector>,
    pub buildings: Vec<Building>,
    pub units: Vec<Unit>,
    pub effects: Vec<Fx>,
    pub bullets: Vec<original_bullets::Bullet>,
    pub projectiles: Vec<original_projectiles::Projectile>,
    pub blasts: Vec<original_projectiles::Blast>,
    pub fires: Vec<original_projectiles::Fire>,
    pub crates: Vec<Crate>,
    pub events: Vec<Value>,
    #[serde(default)] pub objects:Vec<Value>,
    #[serde(default)] pub walls:Vec<walls::Wall>,
    #[serde(skip)] wall_owners:Vec<usize>,
    pub time: f64,
    pub winner: Option<i32>,
    pub difficulty: usize,
    pub mission: usize,
    /// Two human players share one authoritative simulation; difficulty must not favor a side.
    #[serde(default)]
    pub multiplayer: bool,
    #[serde(skip)]
    next_id: usize,
    #[serde(skip)]
    ai_clock: f64,
    #[serde(skip)] original_ai:Option<original_ai::Clock>,
    #[serde(skip)] native_strategy:Option<original_strategy::State>,
    #[serde(skip)] pub(crate) native_group_rosters:original_group_game::State,
    #[serde(skip)] pub(crate) native_world:Option<original_world_contact_game::State>,
    #[serde(skip)]
    rng: u32,
    #[serde(skip)] original_rng:original_combat::OriginalRng,
    #[serde(skip)]
    ai_teams: Vec<i32>,
    #[serde(skip)] production_clock:f64,
    #[serde(skip)] native_pressure:std::collections::BTreeMap<i32,(f32,f32)>,
    #[serde(skip)] native_flag_capture:original_initiative::FlagState,
    #[serde(skip)] native_factory_flags6:std::collections::BTreeMap<usize,u8>,
    #[serde(skip)] native_factory_work:std::collections::BTreeMap<usize,original_factory::Work>,
    #[serde(skip)] native_production_limit:i8,
    #[serde(skip)] native_blast_sound_gate:bool,
    #[serde(skip)] native_blast_sound_cycle:u8,
    #[serde(skip)] native_blast_terrain_class:i8,
    #[serde(skip)] native_factory_placement:original_factory_placement_game::State,
    #[serde(skip)] native_deaths:original_death::State,
    #[serde(skip)] native_view:Option<([i16;2],[i16;2])>,
    #[serde(skip)] native_ambient:Option<original_ambient_game::State>,
    #[serde(skip)] native_ambient_animations:original_ambient_animations::State,
    #[serde(skip)] native_lava:original_lava::State,
    #[serde(skip)] native_cosmetic_rng:u32,
    #[serde(skip)] native_lava_sound_tick:i16,
    #[serde(skip)] native_hardware_pool:Vec<original_orders_game::HardwarePoolSlot>,
    #[serde(skip)] native_fire_base_strength:u8,
    #[serde(skip)] native_smoke20:original_smoke20::State,
    #[serde(skip)] native_bullet_impacts20:original_bullet_impacts20::State,
    #[serde(skip)] native_order_arguments:[i16;3],
    #[serde(skip)] native_attack_reports:Vec<u8>,
    #[serde(skip)] native_attack_ray_building:bool,
    #[serde(skip)] native_attack_walk_marked:bool,
    #[serde(skip)] native_feedback:original_feedback::State,
    #[serde(skip)] native_callback_mode:u8,
    #[serde(skip)] native_wait_clock:i8,
    #[serde(skip)] native_flag_frame:i16,
    #[serde(skip)] native_object_frame:i8,
    #[serde(skip)] native_frame16:i16,
    #[serde(skip)] native_seasonal_mode:u8,
    #[serde(skip)] native_path_contact:original_path_contact::State,
    #[serde(skip)] native_upstream_reactions:original_upstream_reactions::State,
    #[serde(skip)] native_routes:std::cell::RefCell<original_route_pool::State>,
    #[serde(skip)] native_route_boundary:Option<original_path_contact::game::Boundary>,
    #[serde(skip)] native_route_mode:bool,
    #[serde(skip)] native_display:original_display::State,
    #[serde(skip)] native_query_shape:u32,
    #[serde(skip)] native_audio:original_audio::State,
    #[serde(skip)] native_listener_audio:original_listener_audio::State,
    #[serde(skip)] native_protected_order_seen:bool,
    #[serde(skip)] native_group_descriptors48:[u8;19],
    #[serde(skip)] native_collision_masks:original_dynamic_collision::Masks,
    #[serde(skip)] native_work_factor:i16,
    #[serde(skip)] native_team_rates:[i8;4],
    #[serde(skip)] native_workshop_units:original_world_contact_game::workshop::UnitSlots,
    #[serde(skip)] native_selected_slot:Option<usize>,
    #[serde(skip)] native_selection:original_selection::State,
    #[serde(skip)] native_hardware_dummy:original_orders_game::HardwareOrderRecord,
    #[serde(skip)] native_hardware_dummy_attributes:original_entry::HardwareAttributes,
    #[serde(skip)] initiative_clock:f64,
    #[serde(skip)] motion_clock:f64,
    #[serde(skip)] native_ui_clock:original_clock::UiClock,
    #[serde(skip)] motion_tick:u64,
    #[serde(skip)] native_vehicle_positions:Vec<(i32,i32)>,
    #[serde(skip)] native_transports:Vec<original_transport::Boarding>,
    #[serde(skip)] native_transport_commands:Vec<(usize,Value)>,
}
impl Game {
    fn random(&mut self) -> f64 {
        advance_random(&mut self.rng) as f64 / u32::MAX as f64
    }
    pub fn new(mission: usize, difficulty: usize, seed: u32) -> Self {
        let (w, h) = (56, 40);
        let n = w * h;
        let mut g = Self {
            map: Map {
                width: w,
                height: h,
                tile_size: TILE,
                world: (mission / 4) % 5,
                tiles: vec![0; n],
                territory: vec![0; n],
                bridge_base: vec![-1; n],
                bridge_hp: vec![],
                bridge_max_hp: vec![],
                seed,
                revision: 0, original:false,original_tiles:vec![],terrain_codes:vec![],water_tile:0,wall_tile_lookup:Value::Null,native_road_graph:None,native_level_record:0,native_terrain_plane:vec![],native_tile_plane:vec![],wall_ground_tiles:vec![],wall_ground_codes:vec![],wall_ground_classes:vec![],
            },
            sectors: vec![],
            buildings: vec![],
            units: vec![],
            effects: vec![],
            bullets: vec![],
            projectiles: vec![],blasts:vec![],fires:vec![],
            crates: vec![],
            events: vec![], objects:vec![], walls:vec![],wall_owners:vec![],
            time: 0.,
            native_ui_clock:Default::default(),
            winner: None,
            difficulty: difficulty.min(3),
            mission,
            multiplayer: false,
            next_id: 1,
            ai_clock: 3.,original_ai:None,native_strategy:None,native_group_rosters:Default::default(),native_world:None,
            rng: seed.max(1),
            original_rng:original_combat::OriginalRng::seeded(seed),
            ai_teams: vec![1], production_clock:0.,native_factory_work:Default::default(),native_production_limit:50,native_blast_sound_gate:false,native_blast_sound_cycle:0,native_blast_terrain_class:0,native_factory_placement:Default::default(),native_deaths:Default::default(),native_view:None,native_ambient:None,native_ambient_animations:Default::default(),native_lava:Default::default(),native_cosmetic_rng:1,native_lava_sound_tick:0,native_hardware_pool:(0..240).map(|_|original_orders_game::HardwarePoolSlot{live_marker:-1,..Default::default()}).collect(),native_fire_base_strength:0,native_smoke20:Default::default(),native_bullet_impacts20:Default::default(),native_order_arguments:[0;3],native_attack_reports:vec![],native_attack_ray_building:false,native_attack_walk_marked:false,native_feedback:Default::default(),native_callback_mode:0,native_wait_clock:0,native_flag_frame:0,native_object_frame:0,native_frame16:0,native_seasonal_mode:0,native_path_contact:Default::default(),native_upstream_reactions:Default::default(),native_routes:Default::default(),native_route_boundary:None,native_route_mode:false,native_display:Default::default(),native_query_shape:0,native_audio:Default::default(),native_listener_audio:Default::default(),native_protected_order_seen:false,native_group_descriptors48:[0;19],native_collision_masks:Default::default(),native_work_factor:8,native_team_rates:[0;4],native_workshop_units:Default::default(),native_selected_slot:None,native_selection:Default::default(),native_hardware_dummy:Default::default(),native_hardware_dummy_attributes:Default::default(),native_factory_flags6:Default::default(),native_flag_capture:Default::default(),native_pressure:Default::default(), initiative_clock:0.,motion_clock:0.,motion_tick:0,native_vehicle_positions:Vec::new(),native_transports:Vec::new(),native_transport_commands:Vec::new(),
        };
        // Own nine-territory layout. Voronoi edges and a meandering river avoid a square sector grid.
        let sites = [
            (7, 7),
            (25, 6),
            (48, 7),
            (10, 20),
            (28, 20),
            (45, 19),
            (7, 32),
            (30, 33),
            (48, 32),
        ];
        for (id, (x, y)) in sites.iter().enumerate() {
            g.sectors.push(Sector {
                id,
                x: (*x as f64 + 0.5) * TILE,
                y: (*y as f64 + 0.5) * TILE,
                owner: if id == 6 {
                    0
                } else if id == 2 {
                    1
                } else {
                    -1
                },
                home: id == 6 || id == 2, counts_for_production:true,
            });
        }
        for y in 0..h {
            for x in 0..w {
                let j = y * w + x;
                let river = 28. + (y as f64 / 5.).sin() * 2.;
                let mut t = if (x as f64 - river).abs() < 1.4 { 3 } else { 0 };
                if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                    t = 2;
                }
                if t == 0 && g.random() < 0.07 {
                    t = 4;
                }
                if t == 0
                    && x > 3
                    && x < w - 3
                    && y > 3
                    && y < h - 3
                    && ((x as f64 / 3.).sin() + (y as f64 / 2.8).cos() > 1.62)
                {
                    t = 2;
                }
                if (y.abs_diff(13) < 2 || y.abs_diff(28) < 2) && x > 5 && x < 51 {
                    t = if t == 3 { 5 } else { 1 };
                }
                g.map.tiles[j] = t;
                g.map.territory[j] = g
                    .sectors
                    .iter()
                    .min_by_key(|s| {
                        let dx = x as i32 - (s.x / TILE) as i32;
                        let dy = y as i32 - (s.y / TILE) as i32;
                        dx * dx + dy * dy
                    })
                    .unwrap()
                    .id;
            }
        }
        for j in 0..n / 2 {
            g.map.tiles[n - 1 - j] = g.map.tiles[j];
        }
        for s in g.sectors.clone() {
            let kind = if s.home {
                "fort"
            } else {
                match s.id {
                    0 | 4 | 8 => "robot",
                    1 | 7 => "vehicle",
                    3 | 5 => {
                        if mission % 2 == 0 {
                            "repair"
                        } else {
                            "radar"
                        }
                    }
                    _ => "radar",
                }
            };
            let (bw, bh) = if s.home { (144., 128.) } else { (96., 80.) };
            let x = s.x - bw / 2.;
            let y = s.y - bh - 18.;
            for ty in ((y / TILE) as i32 - 1)..=((s.y / TILE) as i32 + 2) {
                for tx in ((x / TILE) as i32 - 2)..=(((x + bw) / TILE) as i32 + 1) {
                    if tx > 0 && ty > 0 && tx < w as i32 - 1 && ty < h as i32 - 1 {
                        g.map.tiles[ty as usize * w + tx as usize] = 0;
                    }
                }
            }
            let stars = if s.home {
                1 + (mission / 5).min(4)
            } else {
                1 + (s.id.min(8 - s.id) + mission) % 5
            };
            let hp = if s.home { 2400. } else { 1200. };
            g.buildings.push(Building {
                id: g.buildings.len(),
                x,
                y,
                w: bw,
                h: bh,
                kind: kind.into(),
                team: s.owner,
                sector: s.id,
                stars,
                hp,
                max_hp: hp,
                product: if s.home { Some("grunt".into()) } else { None },
                progress: 0., ready_guns: Vec::new(), gate:None,orientation:String::new(),remaining_work:None,raw_base:None,completion_ticks:0,
            });
        }
        for j in 0..n {
            if g.map.tiles[j] == 5 && g.map.bridge_base[j] < 0 {
                let id = g.map.bridge_hp.len() as i32;
                g.map.bridge_hp.push(400.);
                let mut q = VecDeque::from([j]);
                g.map.bridge_base[j] = id;
                while let Some(k) = q.pop_front() {
                    for v in [k.wrapping_sub(1), k + 1, k.wrapping_sub(w), k + w] {
                        if v < n && g.map.tiles[v] == 5 && g.map.bridge_base[v] < 0 {
                            g.map.bridge_base[v] = id;
                            q.push_back(v);
                        }
                    }
                }
            }
        }
        for team in [0, 1] {
            let s = g.sectors[if team == 0 { 6 } else { 2 }].clone();
            for squad in 0..2 {
                g.spawn_group("grunt", team, s.x - 38. + squad as f64 * 56., s.y + 22.);
            }
            if mission > 0 {
                g.spawn_group("light", team, s.x, s.y + 68.);
            }
            g.spawn_group("gatling", team, s.x - 58., s.y - 114.);
            g.spawn_group("gatling", team, s.x + 58., s.y - 114.);
            if difficulty == 0 && team == 0 {
                g.spawn_group("grunt", 0, s.x + 8., s.y + 56.);
            }
        }
        for id in [0, 4, 8] {
            let s = g.sectors[id].clone();
            g.spawn_group(
                if id == 4 { "light" } else { "jeep" },
                -1,
                s.x + 72.,
                s.y + 32.,
            );
            g.crates.push(Crate {
                x: s.x - 64.,
                y: s.y + 24.,
                taken: false,
            });
        }
        g.ensure_production();
        g
    }
    fn spawn_group(&mut self, k: &str, team: i32, x: f64, y: f64) {
        let d = stats(k);
        let squad = self.next_id;
        for i in 0..d.members {
            self.spawn(
                k,
                team,
                x + (i % 3) as f64 * 17.,
                y + (i / 3) as f64 * 18.,
                squad,
                None,
            );
        }
    }
    fn spawn(&mut self, k: &str, team: i32, x: f64, y: f64, squad: usize, crew: Option<Crew>) {
        let d = stats(k);if self.map.original&&original_combat::runtime_class(k)==15{self.native_fire_base_strength=native_ai_scoring::default_input(k).map_or(0,|v|v.damage as u8);}
        let crew = if d.cls > 0 && team >= 0 {
            crew.or(Some(Crew {
                kind: "grunt".into(),
                hp: if self.map.original {original_combat::initial_crew_hp(k)} else {40.},
            }))
        } else {
            crew
        };
        let native_slot=if self.map.original&&team>=0 {(0..240).find(|slot|!self.units.iter().any(|u|
            u.team>=0&&original_combat::alive(true,u.hp)&&u.native_slot==Some(*slot)))}else{None};
        //0041b210 returns allocation failure after the shared240-slot pool.
        // Hidden passengers retain slots; the driver's adapter duplicate does not.
        if self.map.original&&team>=0&&native_slot.is_none(){return;}
        let id = self.next_id;
        self.next_id += 1;
        if self.map.original&&self.native_collision_masks.world!=Some(self.map.world as u8){self.native_collision_masks.initialize(self.map.world as u8);}
        self.units.push(Unit {
            id,
            kind: k.into(),
            x,
            y,
            team,
            hp: d.hp,
            max_hp: d.hp,
            angle: if self.map.original{native_math::angle(0)}else{0.},
            aim: if self.map.original{native_math::angle(0)}else{0.},
            moving: false,
            squad,
            formation:None,motion_bonus:0,formation_layout:0,group_bonus_ticks:0,group_activity_ticks:0,native_group_order_pending:false,native_group_motion_bonus:0,native_hardware:None,native_contact:None,native_idle:None,native_cover:None,native_move_frame:0,native_walk:None,native_transport:None,native_transport_slot:None,transport_carrier:None,transport_actor:None,
            native_slot,native_route_flip:std::cell::Cell::new(false),native_route_road_replaced:std::cell::Cell::new(false),native_secondary_route:std::cell::Cell::new(false),native_route_pending:std::cell::Cell::new(None),native_factory_exit:None,native_dodge:None,native_initiative_scan:Default::default(),native_counter6e:0,native_contact6f:if native_slot.is_some(){255}else{0},native_contact70:0,native_wait72:0,native_contact_pausebc:0,native_terrain_tag63:0,native_frame32:0,native_timer3c:if team>=0&&d.cls>0{15}else{0},native_remaining48:0,native_order5c:0,native_range4:None,native_counter97:0,native_mask98:if native_slot.is_some(){7}else{0},native_target46:0,native_counter68:0,native_flags9c:0,native_protectionaa:0,native_control06:native_ai_scoring::default_input(k).map_or(0,|v|v.traits[3]),native_trait7a:native_ai_scoring::default_input(k).map_or(0,|v|v.traits[6]),native_field62:0,native_cyclea0:0,native_axis81:0,native_speed8b:0,native_pattern20:None,native_word90:0,native_counter9f:0,native_cell10:0,native_target42:0,native_target44:0,native_elevation76:0,native_bucket38:0,native_counter67:0,native_counter66:0,native_reload7b:0,native_directiona2:0,native_directiona3:0,native_counter84:0,native_pose82:0,native_direction34:0,native_field9e:0,native_fieldb3:if native_slot.is_some(){-1}else{0},native_footprint18:0,native_permission1c:0,native_pose7e:0,native_pose80:0,native_transition88:0,native_hardware_slot:None,native_class7c:original_combat::runtime_class(k) as u8,native_phase35:if team>=0{original_animation_game::initial_phase(k)}else{0},native_flags64:if team>=0{(team as u8&3)|16}else{0},
            fort_id: None,
            grenades: 0,
            crew,
            cargo: vec![],
            order: "guard".into(),
            target: None,
            dest: None,
            cooldown: 0.,
            original_combat: self.map.original.then(||original_combat::CombatState::new(k)),
            native_fire: None,
            initiative: None,
            march_resume: None,
            hold_position: false,explicit_stop:false,
            workshop: None,
            pursuit: None,
            original_motion: None,
            path: VecDeque::new(),
        });
        if self.map.original{if let Some(slot)=native_slot{self.initialize_native_selection_context(slot);}}
        if self.map.original{self.initialize_native_walk_fields(self.units.len()-1);self.refresh_native_pattern_step(self.units.len()-1,self.motion_tick);if self.native_route_mode{if let Some(slot)=native_slot{let paths=self.native_routes.get_mut();paths.managed[slot]=true;paths.pool.links[slot]=Default::default();}}}
        if self.map.original&&team>=0&&d.cls>0{self.allocate_native_hardware_record(self.units.len()-1,d.cls==1);if d.cls==2&&native_slot.is_some(){let u=self.units.last_mut().unwrap();u.native_counter84=-1;u.angle=native_math::angle(6);}}
        if self.map.original&&d.cls>0{if let Some(slot)=native_slot{self.native_selection.contexts[slot].link28=if k=="jeep"{Some(0x2084)}else{None};}}
        if self.map.original&&native_slot.is_some(){self.initialize_native_collision_tables(self.units.len()-1);if let Err(e)=self.initialize_native_terrain_tag(self.units.len()-1){self.native_route_boundary=Some(e);}}
        if self.map.original&&native_slot.is_some(){self.record_native_workshop_ctor(self.units.len()-1);self.record_native_upstream_ctor(self.units.len()-1);self.record_native_crane_ctor(self.units.len()-1);}
    }
    pub fn products(&self,b:&Building)->Vec<&'static str>{
      if !b.alive(self.map.original){return vec![];}
      if self.map.original{return original_stats::MENU_ORDER.iter().filter_map(|id|original_stats::PRODUCTS.iter().find(|r|r.1==*id)).filter(|r|r.6&&original_stats::required(r.0,&b.kind)>0&&original_stats::required(r.0,&b.kind)<=b.stars).map(|r|r.0).collect();}
      TYPES.iter().copied().filter(|k|{let d=stats(k);d.stars<=b.stars&&(b.kind=="fort"||b.kind=="robot"&&d.cls==0||b.kind=="vehicle"&&d.cls!=0)}).collect()
    }
    // A factory always has a valid repeating order once it belongs to a team.
    // Prefer robots over the original menu's leading gun entries; preserve a
    // player's valid selection and the work inherited when a sector changes hands.
    fn default_product(&self, b: &Building) -> Option<String> {
        let choices = self.products(b);
        let choice = if choices.contains(&"grunt") {
            Some("grunt")
        } else {
            choices.iter().copied().find(|k| stats(k).cls == 0)
                .or_else(|| choices.first().copied())
        };
        choice.map(str::to_string)
    }
    fn ensure_production(&mut self) {
        for i in 0..self.buildings.len() {
            let b = &self.buildings[i];
            if b.team < 0 || !b.alive(self.map.original) { continue; }
            let choices = self.products(b);
            if b.product.as_deref().map(|k| choices.contains(&k)).unwrap_or(false) { continue; }
            let product = self.default_product(b);
            if product != b.product {
                if self.map.original{if let Some(kind)=product.as_deref(){if let Some(work)=self.native_factory_work.get_mut(&self.buildings[i].id){work.animation=0;work.credit=0;work.total=original_stats::workload(kind);work.remaining=self.buildings[i].remaining_work.unwrap_or(f64::from(work.total))as i32;}}}
        self.buildings[i].product = product;
                self.buildings[i].progress = 0.;
                self.buildings[i].remaining_work = None;
            }
        }
    }
    pub fn apply_original(&mut self,v:&Value)->Result<(),String>{
      let map:Map=serde_json::from_value(v["map"].clone()).map_err(|e|e.to_string())?;
      if map.width<1||map.height<1||map.width>128||map.height>128||map.tiles.len()!=map.width*map.height{return Err("Invalid original campaign map".into());}
      let sectors:Vec<Sector>=serde_json::from_value(v["sectors"].clone()).map_err(|e|e.to_string())?;
      let buildings:Vec<Building>=serde_json::from_value(v["buildings"].clone()).map_err(|e|e.to_string())?;
      let crates:Vec<Crate>=serde_json::from_value(v["crates"].clone()).map_err(|e|format!("Invalid original crates: {e}"))?;
      self.native_group_descriptors48=v["nativeGroupDescriptors48"].as_array().filter(|a|a.len()==19).and_then(|a|a.iter().map(|n|n.as_u64().and_then(|n|u8::try_from(n).ok())).collect::<Option<Vec<_>>>()).and_then(|a|a.try_into().ok()).unwrap_or([0;19]);
      self.native_work_factor=v["nativeProduction"]["factor"].as_i64().and_then(|n|i16::try_from(n).ok()).filter(|&n|n!=0).unwrap_or(12); //Existing browser normal-mode adapter until raw17b30 settings are transported.
      self.map=map;self.native_collision_masks.initialize(self.map.world as u8);self.objects=v["originalObjects"].as_array().cloned().unwrap_or_default();self.sectors=sectors;self.buildings=buildings;self.units.clear();self.crates=crates;self.effects.clear();self.bullets.clear();self.projectiles.clear();self.blasts.clear();self.fires.clear();self.native_transports.clear();self.native_transport_commands.clear();self.native_group_rosters.clear_for_mission_preserving_strategy_values();self.motion_clock=0.;self.motion_tick=0;self.native_callback_mode=0;self.native_flag_frame=0;self.native_object_frame=0;self.native_frame16=0;self.native_routes.get_mut().reset_for_mission();self.native_workshop_units.clear_for_mission();self.native_upstream_reactions.clear_for_mission();self.native_route_boundary=None;self.native_route_mode=false;self.native_display.clear_for_mission();self.production_clock=0.;self.native_factory_work.clear();self.native_factory_placement.clear_for_mission();self.native_deaths=Default::default();self.native_ambient_animations.clear_for_mission();self.native_lava.clear_for_mission();self.native_selection.clear_rows_for_mission();self.native_attack_reports.clear();self.native_selected_slot=None;self.native_smoke20.clear_for_mission();self.native_bullet_impacts20.clear_for_mission();original_orders_game::clear_hardware_pool_for_mission(&mut self.native_hardware_pool);self.native_view=None;self.native_flag_capture=Default::default();self.initiative_clock=0.;self.next_id=1;
      if !self.map.bridge_max_hp.is_empty() && (self.map.bridge_max_hp.len()!=self.map.bridge_hp.len() || self.map.bridge_max_hp.iter().any(|hp|!hp.is_finite()||*hp<=0.)) {return Err("Invalid bridge capacities".into());}
      for j in 0..self.map.tiles.len(){if let Some(id)=self.map.bridge_base.get(j).copied().filter(|id|*id>=0){if self.map.bridge_hp.get(id as usize).is_some_and(|hp|*hp<=0.){self.map.tiles[j]=3;}}}
      self.import_walls(v)?;
      self.map.water_tile=self.map.terrain_codes.iter().position(|c|[9,39,40].contains(c)).and_then(|i|self.map.original_tiles.get(i).copied()).unwrap_or(0);
      for spawn in v["spawns"].as_array().ok_or("Missing spawns")?{let kind=spawn["type"].as_str().ok_or("Missing unit type")?;let kind=if kind=="mobile_missile"{"launcher"}else{kind};if !TYPES.contains(&kind){return Err(format!("Unsupported original unit {kind}"));}let x=spawn["x"].as_f64().ok_or("Missing x")?;let y=spawn["y"].as_f64().ok_or("Missing y")?;let team=spawn["team"].as_i64().unwrap_or(-1)as i32;let squad=spawn["squad"].as_u64().unwrap_or(self.next_id as u64)as usize;self.spawn(kind,team,x,y,squad,None);
        let fort_id=spawn["fortId"].as_u64().map(|id|id as usize).filter(|id|stats(kind).cls==2&&self.buildings.iter().any(|b|b.id==*id&&b.kind=="fort"));
        self.units.last_mut().unwrap().fort_id=fort_id;
        self.units.last_mut().unwrap().formation_layout=spawn["formation"].as_u64().filter(|n|*n<5||*n==255).unwrap_or(0)as u8;
      }
      for b in &mut self.buildings { b.ready_guns.retain(|k| gun_placement::is_gun(k)); b.ready_guns.truncate(gun_placement::READY_GUN_CAPACITY); }
      self.native_factory_flags6.clear();for b in v["buildings"].as_array().into_iter().flatten(){if let Some(id)=b["id"].as_u64(){self.native_factory_flags6.insert(id as usize,b["nativeFlags6"].as_u64().unwrap_or(0)as u8);}}
      for b in v["nativeWorldBuildings"].as_array().into_iter().flatten(){if let Some(id)=b["buildingId"].as_u64(){self.native_factory_flags6.insert(id as usize,b["flags6"].as_u64().unwrap_or(0)as u8);}}
      self.ensure_production();
      self.original_ai=v["nativeAi"]["decisionControl"].as_i64().filter(|n|(0..=255).contains(n)).map(|n|original_ai::Clock::new(n as i8 as i16));
      self.native_production_limit=v["nativeProduction"]["teamUnitLimit"].as_u64().and_then(|n|u8::try_from(n).ok()).map(|n|n as i8).unwrap_or(50);self.native_strategy=original_strategy::State::from_mission(v);self.import_native_world(v);self.initialize_native_crane_constructors();self.initialize_native_factory_constructors(v);self.initialize_native_scene_display(v);self.initialize_native_ambient()?;self.maintain_native_group_rosters();for group in v["startingGroups"].as_array().into_iter().flatten(){if let (Some(team),Some(slot),Some(gid))=(group["team"].as_i64(),group["slot"].as_u64(),group["groupId"].as_u64()){if (0..4).contains(&team)&&gid<19{self.initialize_native_group_descriptor(team as i32,team as usize*100+slot as usize+1,gid as usize);}}}self.initialize_native_feedback();self.native_route_mode=self.map.native_road_graph.is_some()&&self.native_world.is_some();if self.native_route_mode{for u in &self.units{if let Some(slot)=u.native_slot{self.native_routes.get_mut().managed[slot]=true;}}}
      Ok(())
    }
    pub fn remaining_seconds(&self,b:&Building)->Option<u32>{let k=b.product.as_deref()?;if self.map.original&&b.completion_ticks>0{return Some(0);}if (!self.map.original||self.map.native_road_graph.is_none())&&gun_placement::is_gun(k)&&b.ready_guns.len()>=gun_placement::READY_GUN_CAPACITY{return None;}if !self.map.original{return Some(((1.-b.progress)*stats(k).seconds)as u32);}let owned=self.sectors.iter().filter(|s|s.owner==b.team&&s.counts_for_production).count();let total=self.sectors.iter().filter(|s|s.counts_for_production).count();let(health,maximum)=self.native_work_health(b);let rate=original_stats::rate(health,maximum,owned,total);if rate==0{return None;}Some((b.remaining_work.unwrap_or(original_stats::workload(k)as f64).max(0.)/(rate*17)as f64)as u32)}

    fn los(&self, x: f64, y: f64, dx: f64, dy: f64) -> bool {
        let steps = ((dx - x).hypot(dy - y) / 12.) as usize;
        for i in 1..steps {
            let a = i as f64 / steps as f64;
            let (tx, ty) = (
                ((x + (dx - x) * a) / TILE) as usize,
                ((y + (dy - y) * a) / TILE) as usize,
            );
            if self.map.tiles.get(ty * self.map.width + tx).is_some_and(|t|matches!(t,2|6)) {
                return false;
            }
        }
        true
    }
    pub fn command(&mut self, v: &Value) {
        let ids: Vec<usize> = v["ids"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|n| n.as_u64().and_then(|x| usize::try_from(x).ok()))
                    .collect()
            })
            .unwrap_or_default();
        let kind = v["kind"].as_str().unwrap_or("move");
        let Ok(team) = request_team(v) else { return; };
        if self.map.original{self.sync_native_world();self.maintain_native_group_rosters();}
        let ids=self.native_transport_command_ids(&ids,team,v);
        if self.try_native_human_attack_command(v,&ids,team){return;}
        //An actual UI click supplies native waypoint coordinates. Internal AI
        //and older callers retain their explicit anchor-coordinate contract.
        let native_ui_move=self.map.original&&self.map.native_road_graph.is_some()&&kind=="move"&&v["nativeEvent"]!=false&&v.get("nativeClick").is_some()&&ids.iter().all(|id|self.units.iter().find(|u|u.id==*id).and_then(|u|u.native_slot).and_then(|slot|self.native_group_rosters.members[slot].as_ref()).is_none_or(|m|m.role<=0||self.native_group_rosters.pool.get(usize::from(m.group)).and_then(|g|g.member(0)).is_some_and(|slot|self.units.iter().any(|u|u.native_slot==Some(usize::from(slot))&&ids.contains(&u.id)))));
        let mut native_move_goals=std::collections::BTreeMap::new();
        if native_ui_move{
            let Some(click)=v["nativeClick"].as_array().filter(|p|p.len()==2).and_then(|p|Some([i16::try_from(p[0].as_i64()?).ok()?,i16::try_from(p[1].as_i64()?).ok()?]))else{return;};
            if click[0]<0||click[1]<0||usize::try_from(click[0]).unwrap()>=self.map.width*16||usize::try_from(click[1]).unwrap()>=self.map.height*16||v.get("target").is_some_and(|t|!t.is_null()){return;}
            let eligible=ids.iter().filter_map(|id|self.units.iter().position(|u|u.id==*id&&u.team==team&&original_combat::alive(true,u.hp)&&stats(&u.kind).speed>0.&&!workshops::hidden(u))).collect::<Vec<_>>();
            let Some(primary)=v["nativePrimary"].as_u64().and_then(|id|eligible.iter().copied().find(|&i|self.units[i].id==id as usize)).filter(|&i|self.units[i].native_slot.and_then(|s|self.native_group_rosters.role(s)).is_some_and(|r|r<=0)).or_else(||eligible.iter().copied().find(|&i|self.units[i].native_slot.and_then(|s|self.native_group_rosters.role(s)).is_some_and(|r|r<=0)))else{return;};
            let flag_mode=v["nativeFlagMode"].as_bool().unwrap_or(false);
            if self.classify_native_human_move(primary,click,flag_mode).is_none(){return;}
            let mut groups=Vec::new();for &i in &eligible{if let Some(group)=self.units[i].native_slot.and_then(|slot|self.native_group_rosters.members[slot].as_ref()).map(|m|m.group).filter(|&g|g!=255){if !groups.contains(&group){groups.push(group);}}}
            self.select_native_groups(&groups);let Some(prepared)=self.prepare_native_human_move_batch(primary,click,flag_mode)else{return;};
            for movement in prepared{let actor=&self.units[movement.index];if actor.team!=team||!ids.contains(&actor.id)||!original_combat::alive(true,actor.hp){continue;}let goal=(f64::from(movement.anchor[0])*2.,f64::from(movement.anchor[1])*2.);native_move_goals.insert(actor.squad,goal);self.record_native_order_event(movement.index,movement.event);}
        }

        if self.map.original&&kind=="move" {
            if let Some(layout)=v["formation"].as_u64().filter(|n|*n<5){for u in self.units.iter_mut().filter(|u|u.team==team&&ids.contains(&u.id)){u.formation_layout=layout as u8;}}
            if native_ui_move{for(&squad,&goal)in &native_move_goals{let members=ids.iter().copied().filter(|id|self.units.iter().any(|u|u.id==*id&&u.squad==squad&&u.team==team)).collect::<Vec<_>>();self.prepare_native_formations(&members,team,goal);}}else if let Some((x,y))=v["x"].as_f64().zip(v["y"].as_f64()).filter(|(x,y)|x.is_finite()&&y.is_finite()) {self.prepare_native_formations(&ids,team,(x,y));}
        }
        if self.map.original{self.maintain_native_group_rosters();}
        for (n, id) in ids.iter().enumerate() {
            let Some(i) = self
                .units
                .iter()
                .position(|u| u.id == *id && u.team == team && original_combat::alive(self.map.original,u.hp))
            else {
                continue;
            };
            if native_ui_move&&!native_move_goals.contains_key(&self.units[i].squad){continue;}
            let actor=&self.units[i];let target=v["target"].as_u64().and_then(|n|usize::try_from(n).ok());
            if v.get("target").is_some_and(|value|!value.is_null())&&target.is_none(){continue;}
            if !matches!(kind,"move"|"attack"|"attackBuilding"|"attackWall"|"attackBridge"|"repair"|"repairBridge"|"repairVehicle"|"enter"|"board"|"stop"|"unload"){continue;}
            if kind=="move"&&stats(&actor.kind).speed<=0.{continue;}
            if matches!(kind,"repair"|"repairBridge")&&actor.kind!="crane"{continue;}
            if kind=="repair"&&!target.is_some_and(|id|self.buildings.iter().any(|b|b.id==id&&b.team==team&&b.hp<b.max_hp)){continue;}
            if kind=="attackBuilding"&&(!(stats(&actor.kind).damage>0.||self.map.original&&walls::apc_point_capable(actor))||!target.is_some_and(|id|self.buildings.iter().any(|b|b.id==id&&b.team!=team&&b.alive(self.map.original)))){continue;}
            if matches!(kind,"enter"|"board")&&(!matches!(stats(&actor.kind).cls,0)||!target.is_some_and(|id|self.units.iter().any(|u|u.id==id&&original_combat::alive(self.map.original,u.hp)&&!workshops::hidden(u)&&if kind=="enter"{u.team<0&&u.crew.is_none()&&stats(&u.kind).cls>0}else{u.team==team&&u.kind=="apc"&&self.apc_has_passenger_room(u)}))){continue;}
            if matches!(kind,"repairBridge"|"attackBridge") {
                let bridge=v["x"].as_f64().zip(v["y"].as_f64()).and_then(|(x,y)|self.bridge_at_point(x,y));
                if !bridge.is_some_and(|id|if kind=="repairBridge"{self.map.bridge_hp[id]<self.bridge_capacity(id)}else{self.map.bridge_hp[id]>0.&&(stats(&actor.kind).damage>0.||self.map.original&&walls::apc_point_capable(actor))}){continue;}
            }
            if kind=="attackWall" && (!walls::capable(&self.units[i]) || target.and_then(|id| self.wall(id)).is_none()) { continue; }
            if kind=="repairVehicle" {
                if let Some(target)=v["target"].as_u64().and_then(|n|usize::try_from(n).ok()) { self.begin_workshop(i,target); }
                continue;
            }
            if !native_ui_move&&(kind=="move"||kind=="attack"&&target.is_none())&&!v["x"].as_f64().zip(v["y"].as_f64()).is_some_and(|(x,y)|x.is_finite()&&y.is_finite()){continue;}
            if kind=="attack"&&target.is_some_and(|id|!self.units.iter().any(|u|u.id==id&&u.team>=0&&u.team!=team&&original_combat::alive(self.map.original,u.hp)&&!workshops::hidden(u))){continue;}
            if self.map.original{let u=&self.units[i];self.native_group_rosters.set_current_roster(u.team,u.squad);}
            if kind=="repair"{if let Some(target)=target{let click=v["x"].as_f64().zip(v["y"].as_f64()).map(|(x,y)|[(x/2.).floor()as i16,(y/2.).floor()as i16]);if self.begin_native_crane_building(i,target,click).is_some(){continue;}}}
            if self.map.original&&v["nativeEvent"]!=false&&!native_ui_move{
                let event=match kind{
                    "move"=>Some(original_orders_game::Event::classified(1,[((v["x"].as_f64().unwrap()/2.).floor()as i16).wrapping_add(8),((v["y"].as_f64().unwrap()/2.).floor()as i16).wrapping_add(8)],0)),
                    "stop"|"unload"=>Some(original_orders_game::Event::classified(if kind=="stop"{4}else{13},[self.native_order_arguments[0],self.native_order_arguments[1]],self.native_order_arguments[2])),
                    "attack" if target.is_some()=>target.and_then(|id|self.units.iter().position(|u|u.id==id)).and_then(|ti|self.native_ui_target_order_event(ti,matches!(self.units[i].native_class7c,1|2))),
                    "attackBuilding" if v["impactX"].is_null()&&v["x"].is_null()=>target.and_then(|id|self.native_building_target_event(id)),
                    "attack"|"attackBuilding"=>v["impactX"].as_f64().or_else(||v["x"].as_f64()).zip(v["impactY"].as_f64().or_else(||v["y"].as_f64())).filter(|(x,y)|x.is_finite()&&y.is_finite()).map(|(x,y)|original_orders_game::building_target_event([(x/2.).floor()as i16,(y/2.).floor()as i16])),
                    _=>None,
                };if let Some(event)=event{self.record_native_order_event(i,event);}
            }
            //45f070 records MOVE now;54160 executes53730 at the next
            //59ms callback. Existing line/heads remain intact until dispatch.
            if self.map.original&&self.native_route_mode&&kind=="move"&&(native_ui_move||v["nativeEvent"]!=false)&&self.units[i].native_order5c==1{
                let goal=if native_ui_move{native_move_goals.get(&self.units[i].squad).copied()}else{v["x"].as_f64().zip(v["y"].as_f64())};let u=&mut self.units[i];u.order="move".into();u.target=None;if let Some(goal)=goal{u.dest=Some(goal);}u.hold_position=false;u.explicit_stop=false;continue;
            }
            //578b0 needs the classified approach goal before its claim. The
            //target-only web payload has no original43a600 click/dispersal result.
            if self.map.original&&kind=="enter"{if let Some(u)=target.and_then(|id|self.units.iter().find(|u|u.id==id)){let goal=[(u.x/2.).floor()as i16+8,(u.y/2.).floor()as i16+8];let actor=&self.units[i];self.native_group_rosters.record_group_order(team,actor.squad,1,goal,0);}}

            let native_claim=if self.map.original&&kind=="enter"&&self.native_world.is_some(){
                self.release_original_claim(i);let Some(claim)=target.and_then(|id|self.claim_native_hardware_order(i,id))else{continue;};Some(claim)
            }else{if self.map.original{self.release_original_claim(i);}None};
            self.cancel_workshop(i);
            self.units[i].initiative=None;
            self.units[i].march_resume=None;
            self.units[i].pursuit=None;self.units[i].native_route_pending.set(None);
            self.units[i].native_factory_exit=None;self.units[i].native_dodge=None;self.units[i].native_flags64&=!0x40;if stats(&self.units[i].kind).cls==0{self.units[i].native_phase35=0;}
            self.units[i].original_motion=None;
            self.units[i].native_fire=None;self.units[i].native_idle=None;self.units[i].native_cover=None;
            if self.units[i].native_contact.as_ref().is_some_and(|s|s.phase==2){self.units[i].native_contact=None;}
            self.units[i].native_group_order_pending=self.map.original&&!matches!(kind,"stop"|"unload")&&stats(&self.units[i].kind).speed>0.;
            if kind!="move"{self.units[i].formation=None;if native_claim.is_none(){self.units[i].motion_bonus=0;}}
            self.units[i].hold_position=self.map.original&&kind=="stop";
            self.units[i].explicit_stop=self.map.original&&kind=="stop";
            if kind == "unload" {
                if self.map.original&&self.unload_native_hardware(i){continue;}
                let u = self.units[i].clone();
                if u.kind == "apc" {
                    let cargo = std::mem::take(&mut self.units[i].cargo);
                    for (c, k) in cargo.into_iter().enumerate() {
                        self.spawn(
                            &k.kind,
                            team,
                            u.x + 24. + c as f64 * 9.,
                            u.y + 28.,
                            self.next_id,
                            None,
                        );
                        let last = self.units.len() - 1;
                        self.units[last].hp = k.hp;
                        self.units[last].hold_position=self.map.original;
                    }
                } else if stats(&u.kind).cls > 0 {
                    if let Some(c) = self.units[i].crew.take() {
                        if self.map.original {
                            let (dx,dy)=original_tank_origin_offset(&u.kind,false);
                            self.units[i].x+=dx;self.units[i].y+=dy;
                        }
                        self.units[i].team = -1;
                        self.units[i].native_slot=None;
                        self.finish_order(i);
                        self.spawn(&c.kind, team, u.x + 26., u.y + 24., self.next_id, None);
                        let last = self.units.len() - 1;
                        self.units[last].hp = c.hp;
                        self.units[last].hold_position=self.map.original;
                        if self.map.original{
                            self.units[last].native_slot=u.native_slot;self.units[last].native_route_flip.set(u.native_route_flip.get());self.units[last].native_route_road_replaced.set(u.native_route_road_replaced.get());self.units[last].native_secondary_route.set(u.native_secondary_route.get());
                            self.sync_native_world();if let Some(slot)=self.native_object_slot(u.id){self.units[last].native_initiative_scan.excluded9d=slot;}
                        }
                    }
                }
                continue;
            }
            self.units[i].order = kind.into();
            self.units[i].target = target;
            self.units[i].path.clear();
            self.units[i].dest = None;
            if let Some(claim)=native_claim{let point=(f64::from(claim.destination[0]-8)*2.,f64::from(claim.destination[1]-8)*2.);self.units[i].dest=Some(point);self.units[i].path.clear();self.units[i].native_route_pending.set(Some(point));continue;}
            if kind == "attackWall" {
                let wall=self.wall(self.units[i].target.unwrap()).unwrap().clone();
                self.units[i].dest=Some(wall.center());
                self.units[i].path=if stats(&self.units[i].kind).speed>0.{self.route_to_wall_attack(&self.units[i],&wall,stats(&self.units[i].kind).range)}else{VecDeque::new()};
                continue;
            }
            if kind == "stop" {
                self.units[i].order = "guard".into();
                continue;
            }
            let command_goal=if native_ui_move{native_move_goals.get(&self.units[i].squad).copied()}else{v["x"].as_f64().zip(v["y"].as_f64())};
            if let Some((x,y))=command_goal {
                let formation=matches!(kind,"move"|"attack");
                let d=if self.map.original {self.native_formation_destination(i,(x,y))}else{(x+if formation{(n%4)as f64*12.}else{0.},y+if formation{(n/4)as f64*14.}else{0.})};
                self.units[i].dest = Some(d);
                self.units[i].path = if stats(&self.units[i].kind).speed<=0.{VecDeque::new()}else if matches!(kind,"attackBridge"|"repairBridge"){self.route_to_attack(&self.units[i],d.0,d.1,if kind=="repairBridge"{110.}else{stats(&self.units[i].kind).range})}else{self.route_unit(&self.units[i],d.0,d.1)};
            }
        }
    }
    pub fn produce(&mut self, id: usize, k: Option<&str>) {
        self.produce_for_team(id, k, 0);
    }
    pub fn produce_for_team(&mut self, id: usize, k: Option<&str>, team: i32) -> bool {
        if !(0..=1).contains(&team) { return false; }
        let Some(i) = self.buildings.iter().position(|b| b.id == id && b.team == team && b.alive(self.map.original)) else { return false; };
        if k.is_some_and(|kind| !self.products(&self.buildings[i]).contains(&kind)) { return false; }
        let product = k.map(str::to_string).or_else(|| self.default_product(&self.buildings[i]));
        if self.buildings[i].product != product {
            self.buildings[i].progress = 0.;self.buildings[i].remaining_work=if self.map.original{product.as_deref().map(|k|original_stats::workload(k)as f64)}else{None};
            if self.map.original{if let Some(kind)=product.as_deref(){if let Some(work)=self.native_factory_work.get_mut(&id){work.animation=0;work.credit=0;work.total=original_stats::workload(kind);work.remaining=work.total;}}}
        }
        self.buildings[i].product = product;
        true
    }
    fn ai(&mut self) {
        if self.map.original&&self.native_strategy.is_some(){
            self.update_original_strategy(false);return;
        }
        let teams = self.ai_teams.clone();
        for team in teams {
            self.place_ai_guns(team);
            for bi in 0..self.buildings.len() {
                let b = self.buildings[bi].clone();
                if b.team != team || !b.alive(self.map.original) {
                    continue;
                }
                if b.product.is_none() {
                    self.buildings[bi].product = self.default_product(&b);
                }
            }
            let available: Vec<usize> = self
                .units
                .iter()
                .filter(|u| {
                    u.team == team && original_combat::alive(self.map.original,u.hp) && stats(&u.kind).speed > 0. && u.path.is_empty() && u.initiative.is_none() && (!u.hold_position||self.map.original&&!u.explicit_stop) && u.workshop.is_none()
                        && matches!(u.order.as_str(),"guard"|"move")
                })
                .map(|u| u.id)
                .collect();
            let mut commanded_squads=std::collections::HashSet::new();
            for id in available {
                let u = self.units.iter().find(|u| u.id == id).unwrap().clone();
                if self.map.original&&(u.formation.is_some()||!commanded_squads.insert(u.squad)){continue;}
                let goal = self
                    .sectors
                    .iter()
                    .filter(|s| s.owner != team && s.counts_for_production)
                    .min_by(|a, b| {
                        if let Some(clock)=self.original_ai.as_ref().filter(|_|self.map.original) {
                            let score=|s:&Sector|{let dx=((s.x-u.x)/2.)as i32;let dy=((s.y-u.y)/2.)as i32;original_ai::perturb(dx*dx+dy*dy,clock.control,&self.original_rng)};
                            score(a).cmp(&score(b))
                        }else{((a.x - u.x).hypot(a.y - u.y)).total_cmp(&(b.x - u.x).hypot(b.y - u.y))}
                    });
                if let Some(s) = goal {
                    let(x,y)=if s.home{self.buildings.iter().find(|b|b.sector==s.id&&b.kind=="fort").and_then(|b|b.gate).unwrap_or((s.x,s.y))}else{(s.x,s.y)};
                    let ids=if self.map.original {self.units.iter().filter(|v|v.team==team&&v.squad==u.squad&&original_combat::alive(true,v.hp)&&stats(&v.kind).speed>0.).map(|v|v.id).collect::<Vec<_>>()}else{vec![id]};
                    self.command(&json!({"team":team,"ids":ids,"kind":"move","x":x,"y":y}));
                }
            }
        }
    }
    pub fn step(&mut self, dt: f64) {
        self.native_ui_clock.reset();
        if !self.map.original {self.step_update(dt);return;}
        self.motion_clock+=dt.clamp(0.,0.1);
        while self.motion_clock+1e-10>=original_motion::TICK_SECONDS {
            self.motion_clock-=original_motion::TICK_SECONDS;
            // The original mission clock counts17 callbacks as one second.
            // Wall scheduling and simulation time are distinct quantities.
            self.step_update(1./17.);
            if self.winner.is_some(){break;}
        }
    }
    fn update_production(&mut self,dt:f64){
        self.ensure_production();
        if self.map.original{self.update_original_production(dt);return;}
        self.production_clock+=dt*17.;let production_ticks=self.production_clock.floor()as i32;self.production_clock-=production_ticks as f64;
        let total=self.sectors.iter().filter(|s|s.counts_for_production).count();
        let mut completed = vec![];
        for b in &mut self.buildings {
            if b.team < 0 || !b.alive(self.map.original) { continue; }
            if let Some(k)=b.product.clone() {
                if gun_placement::is_gun(&k) && b.ready_guns.len()>=gun_placement::READY_GUN_CAPACITY { continue; }
                let owned=self.sectors.iter().filter(|s|s.owner==b.team).count();
                let mult=if b.team==1 {[0.55,1.,1.45,1.85][self.difficulty]}else{1.};
                b.progress+=dt*(0.65+owned as f64*0.32)*(b.hp/b.max_hp).max(0.15)*mult/stats(&k).seconds;
                if b.progress>=1. {b.progress-=1.;completed.push((k,b.team,b.x+b.w/2.,b.y+b.h+30.,b.id));}
            }
        }
        for (k, team, x, y, factory) in completed {
            if gun_placement::is_gun(&k) {
                if let Some(b) = self.buildings.iter_mut().find(|b| b.id == factory) {
                    if b.ready_guns.len() < gun_placement::READY_GUN_CAPACITY {
                        b.ready_guns.push(k.clone());
                        self.events.push(json!({"kind":"ready","team":team,"type":k,"factory":factory}));
                    }
                }
                continue;
            }
            if self.units.iter().filter(|u|original_combat::alive(false,u.hp)&&u.team==team).count()<120 {
                self.spawn_group(&k, team, x, y);
                self.events
                    .push(json!({"kind":"ready","team":team,"type":k}));
            }
        }
    }

    fn capture_sector(&mut self,team:i32,id:usize){
        let Some(s)=self.sectors.iter_mut().find(|s|s.id==id&&s.owner!=team)else{return;};let previous=s.owner;s.owner=team;
        if let Some(strategy)=self.native_strategy.as_mut(){strategy.captured(team,id);}
        for b in self.buildings.iter_mut().filter(|b|b.sector==id){b.team=team;}
        self.events.push(json!({"kind":"capture","team":team,"previous":previous,"sector":id}));self.clear_native_hardware_restrictions(id);if self.map.original{self.initialize_native_capture_defaults(id);}else{self.ensure_production();}self.rebuild_native_scene_flags();
    }
    fn step_update(&mut self, dt: f64) {
        if self.winner.is_some() {
            return;
        }
        let dt = dt.clamp(0., 0.1);
        self.time += dt;
        let first_motion_tick=self.motion_tick+1;
        let motion_ticks=if self.map.original {self.motion_tick+=1;1}else{0};
        if self.map.original{self.native_blast_sound_gate=false;self.native_frame16=self.native_frame16.wrapping_add(1)&15;self.native_object_frame=self.native_object_frame.wrapping_add(1);if self.native_object_frame==6{self.native_object_frame=0;}self.native_callback_mode=if self.native_callback_mode==0{255}else{0};self.native_wait_clock=self.native_wait_clock.wrapping_add(1)&7;if self.native_callback_mode!=0{self.native_flag_frame=self.native_flag_frame.wrapping_add(1)&3;}}
        self.effects.iter_mut().for_each(|e| e.age += dt);
        self.effects.retain(|e| e.age < e.life);
        self.advance_original_group_cooldowns();
        self.maintain_native_transports();self.sync_native_world();
        if self.map.original{self.update_native_track_effects();self.begin_native_display_frame();self.begin_native_owner4_main();self.capture_native_building_display();self.native_protected_order_seen=false;}
        self.update_initiative(dt);
        self.ai_clock -= dt;
        if !self.map.original&&self.ai_clock<=0. {
            self.ai();
            self.ai_clock = [8., 4.5, 2.5, 1.3][self.difficulty];
        }
        let snapshot = if self.map.original{Vec::new()}else{self.units.clone()};
        let mut damage: Vec<(usize, f64, bool)> = vec![];
        let mut bdamage: Vec<(usize, f64, i32)> = vec![];
        //4b280 consumes the pre-update UNIT mirrors, once per callback.
        self.maintain_native_group_rosters();
        self.update_native_groups();
        if let Some(state)=&mut self.native_strategy{state.begin_callback();}
        for group in self.native_group_rosters.pool.iter_mut(){group.strategy[16]&=!1;}
        self.prepare_native_ambient_queries();
        if self.map.original{self.prepare_native_path_contacts();self.native_vehicle_positions=self.units.iter().filter(|u|u.team>=0&&stats(&u.kind).cls==1&&original_combat::alive(true,u.hp)&&!workshops::hidden(u)).map(|u|((u.x/2.).floor()as i32,(u.y/2.).floor()as i32)).collect();}
        //The native loop visits240 fixed UNIT addresses. Resolve the current
        //Web identity at each slot, because takeover may change it mid-frame.
        let update_order:Vec<(bool,usize)>=if self.map.original{
            (0..240).map(|slot|(true,slot)).chain((0..self.units.len()).filter(|&i|!self.native_route_mode&&self.units[i].native_slot.is_none()).map(|i|(false,i))).collect()
        }else{(0..self.units.len()).map(|i|(false,i)).collect()};
        'native_units: for (native_slot_callback,address) in update_order {
          let i=if native_slot_callback{let Some(i)=self.units.iter().rposition(|u|u.native_slot==Some(address))else{continue;};if self.native_group_rosters.members[address].as_ref().is_none_or(|m|m.flags31&4==0){continue;}i}else{if self.map.original&&self.units[address].native_slot.is_some(){continue;}address};
          let native_display_slot=self.native_unit_display_slot(i);
          'unit: {
            // Native unit slots run sequentially. Later slots observe the
            // movement and boarding already performed by earlier slots.
            //Native target queries read the live pool; clone only the selected
            //target, avoiding a deep copy of every path for every unit slot.
            match self.advance_native_unit_preamble(i){
                original_unit_preamble::game::Outcome::Continue=>{},
                original_unit_preamble::game::Outcome::Frozen=>continue 'native_units,
                original_unit_preamble::game::Outcome::Retired(_)=>continue 'native_units,
                original_unit_preamble::game::Outcome::Removed=>{if self.native_route_mode&&self.native_route_boundary.is_none(){self.native_route_boundary=Some(original_path_contact::game::Boundary::RetiredUnitPreamble(address));}continue 'native_units;},
                original_unit_preamble::game::Outcome::Unavailable=>{
                    if self.native_route_mode&&native_slot_callback&&self.units[i].team>=0&&original_combat::alive(true,self.units[i].hp){self.native_route_boundary=Some(original_path_contact::game::Boundary::UnitPreamble(address,self.units[i].native_phase35));continue 'native_units;}
                    if self.map.original{if let Some(slot)=self.units[i].native_slot{self.native_group_rosters.observe_unit_context(slot);}}
                    self.advance_native_order_preamble(i);self.advance_original_group_leader(i);
                }
            }
            self.refresh_native_pattern_step(i,first_motion_tick);self.native_grenade_contact(i);self.native_workshop_contact(i);if let Some(sector)=self.native_flag_contact(i){let team=self.units[i].team;self.capture_sector(team,sector);self.finish_native_flag_contact(i);}
            if native_display_slot.is_some()&&self.units[i].native_reload7b!=0{self.units[i].native_reload7b=self.units[i].native_reload7b.wrapping_sub(1);}
            self.update_native_factory_exit(i);
            if self.map.original&&self.units[i].native_contact.is_none() {self.update_native_follower(i);}
            if self.update_workshop(i,dt) { break 'unit; }
            let mut u = self.units[i].clone();
            if !original_combat::alive(self.map.original,u.hp) || u.team < 0 {
                break 'unit;
            }
            let mut d = self.native_unit_stats(&u);
            if self.map.original {
                d.speed=original_combat::movement_speed(&u.kind);
                if let Some(state)=&u.original_combat {
                    d.speed*=state.speed_scale(&u.kind);
                    d.reload=state.reload_seconds();
                }
            }
            self.units[i].moving = false;
            if !u.moving{self.units[i].native_walk=None;}
            self.units[i].cooldown = (self.units[i].cooldown-dt).max(0.);
            if self.units[i].cooldown<1e-9 {self.units[i].cooldown=0.;}
            if self.units[i].native_factory_exit.is_none(){self.dispatch_recorded_native_order(i);}
            if self.map.original&&self.units[i].native_slot.is_some()&&!self.owns_native_crane_handler(i){match self.dispatch_native_occupancy_state(i){Ok(true)=>break 'unit,Ok(false)=>{},Err(e)=>{self.native_route_boundary=Some(e);break 'unit;}}}
            if self.update_native_feedback_pose(i){break 'unit;}
            if self.update_native_idle_pose(i){break 'unit;}
            if self.update_native_ambient_point_fire(i){break 'unit;}
            if self.update_native_ground_fire(i).is_some(){break 'unit;}
            self.start_native_group_order(i);
            if self.update_native_dodge(i,first_motion_tick,motion_ticks){break 'unit;}
            if self.update_native_transport(i){break 'unit;}
            if self.update_native_hardware_entry(i){break 'unit;}
            if self.update_native_contact_state(i){break 'unit;}
            if self.update_native_cover(i){break 'unit;}
            if self.update_native_crane_building(i,first_motion_tick,motion_ticks){break 'unit;}
            if self.update_native_route_wait(i)||self.update_native_attack_phase(i)||self.update_native_walk(i)||self.update_native_walk_idle_entry(i){break 'unit;}
            if self.units[i].native_route_pending.get().is_none()&&self.update_native_idle_handler(i){break 'unit;}
            self.update_native_hardware(i);
            if self.update_native_hardware_fire(i){break 'unit;}
            let pursuit_hold=self.update_pursuit(i,&snapshot,d.range,d.speed);
            let wall_hold=self.update_wall_attack(i,d.range,d.damage,d.reload,d.speed);
            let building_hold=self.update_building_attack(i,d.range,d.speed);
            u=self.units[i].clone();
            let mut bridge_hold=false;
            if matches!(u.order.as_str(),"attackBridge"|"repairBridge") {
                let target=u.dest.and_then(|(x,y)|self.bridge_at_point(x,y).map(|id|(x,y,id)));
                if let Some((x,y,id))=target {
                    let repairing=u.order=="repairBridge";
                    let capacity=self.bridge_capacity(id);
                    let valid=if repairing{u.kind=="crane"&&self.map.bridge_hp[id]<capacity}else{(d.damage>0.||self.map.original&&walls::apc_point_capable(&u))&&self.map.bridge_hp[id]>0.&&(!self.map.original||walls::capable(&u)||walls::apc_point_capable(&u))};
                    if !valid{self.finish_order(i);break 'unit;}
                    let range=if repairing{110.}else{d.range};
                    bridge_hold=(u.x-x).hypot(u.y-y)<=range&&self.unit_los(&u,x,y);
                    if bridge_hold {
                        self.units[i].path.clear();
                        if repairing {
                            self.map.bridge_hp[id]=(self.map.bridge_hp[id]+dt*65.).min(capacity);
                            if self.map.bridge_hp[id]>=capacity {
                                for k in 0..self.map.tiles.len(){if self.map.bridge_base[k]==id as i32{self.map.tiles[k]=5;}}
                                self.map.revision=self.map.revision.wrapping_add(1);
                                self.events.push(json!({"kind":"bridgeRepaired","team":u.team}));self.finish_order(i);
                            }
                        }else if self.map.original {
                            self.units[i].aim=(y-u.y).atan2(x-u.x);
                            self.fire_native_ground(i,(x,y),id.wrapping_add(1000000),!walls::primary_capable_unit(&u)&&!walls::apc_point_capable(&u));
                        }else if self.units[i].cooldown<=0. {
                            self.units[i].cooldown=d.reload;self.map.bridge_hp[id]=(self.map.bridge_hp[id]-d.damage).max(0.);
                            self.events.push(json!({"kind":"shot","type":u.kind,"x":u.x,"y":u.y,"team":u.team}));
                            if self.map.bridge_hp[id]<=0. {
                                for k in 0..self.map.tiles.len(){if self.map.bridge_base[k]==id as i32{self.map.tiles[k]=3;}}
                                for v in &mut self.units {
                                    let col=(v.x/TILE).floor()as usize;let row=(v.y/TILE).floor()as usize;
                                    if col<self.map.width&&row<self.map.height&&self.map.bridge_base.get(row*self.map.width+col)==Some(&(id as i32)){v.hp=if self.map.original{-1.}else{0.};}
                                }
                                self.map.revision=self.map.revision.wrapping_add(1);
                                self.events.push(json!({"kind":"bridgeDestroyed","team":u.team}));self.finish_order(i);
                            }
                        }
                    }else if self.units[i].path.is_empty()&&d.speed>0.{self.units[i].path=self.route_to_attack(&u,x,y,range);}
                }else{self.finish_order(i);break 'unit;}
                if !original_combat::alive(self.map.original,self.units[i].hp){break 'unit;}
            }
            // Resolve stable identifiers and discard lost interaction targets.
            if ["enter", "board", "repair"].contains(&u.order.as_str()) {
                let goal;
                if u.order=="repair" {
                    if u.kind!="crane" {self.finish_order(i);break 'unit;}
                    if let Some((bi,b))=u.target.and_then(|id|self.buildings.iter().enumerate().find(|(_,b)|b.id==id&&b.team==u.team)).map(|(bi,b)|(bi,b.clone())) {
                        if b.hp>=b.max_hp {self.finish_order(i);break 'unit;}
                        let point=(b.x+b.w/2.,b.y+b.h+22.);goal=Some(point);
                        if (u.x-point.0).hypot(u.y-point.1)<70.&&self.movement_segment_clear_for(u.x,u.y,point.0,point.1,&u.kind) {
                            self.buildings[bi].hp=(b.hp+dt*48.).min(b.max_hp);
                            if self.buildings[bi].hp>=b.max_hp {self.finish_order(i);self.events.push(json!({"kind":"repair","team":u.team,"factory":b.id}));}
                        }
                    }else{self.finish_order(i);break 'unit;}
                }else{
                    let target=u.target.and_then(|id|self.units.iter().find(|v|v.id==id&&original_combat::alive(self.map.original,v.hp)&&!workshops::hidden(v))).cloned();
                    let valid=target.as_ref().is_some_and(|v|if u.order=="enter"{v.team<0&&v.crew.is_none()&&stats(&v.kind).cls>0}else{v.team==u.team&&v.kind=="apc"&&self.apc_has_passenger_room(v)});
                    if d.cls!=0||!valid{
                        if u.initiative.is_some(){self.resume_march(i,u.march_resume);}else{self.finish_order(i);}
                        break 'unit;
                    }
                    let v=target.unwrap();let claimed=self.map.original&&u.native_slot.and_then(|slot|self.native_group_rosters.members[slot].as_ref()).is_some_and(|m|m.flags31&8!=0&&m.contact_order!=0);
                    goal=if claimed{u.dest}else{Some((v.x,v.y))};
                    if if self.map.original{let dx=(u.x/2.).floor()as i32-(v.x/2.).floor()as i32;let dy=(u.y/2.).floor()as i32-(v.y/2.).floor()as i32;dx*dx+dy*dy<401}else{(u.x-v.x).hypot(u.y-v.y)<28.} {
                        let vi=self.units.iter().position(|v0|v0.id==v.id).unwrap();
                        if u.order=="enter" {
                            if self.map.original {
                                let (dx,dy)=original_tank_origin_offset(&v.kind,true);
                                self.units[vi].x+=dx;self.units[vi].y+=dy;
                                // Ordinary enter00450db0 invokes00450cd0,
                                // which initializes both body and aim to6 (west).
                                if matches!(v.kind.as_str(),"light"|"medium"|"heavy") {
                                    self.units[vi].angle=std::f64::consts::PI;self.units[vi].aim=std::f64::consts::PI;
                                }
                            }
                            self.units[vi].team=u.team;self.units[vi].crew=Some(Crew{kind:u.kind.clone(),hp:u.hp});
                            self.units[vi].native_slot=u.native_slot;self.units[vi].native_route_flip.set(u.native_route_flip.get());self.units[vi].native_route_road_replaced.set(u.native_route_road_replaced.get());
                            self.units[vi].native_hardware=None;self.units[vi].native_phase35=original_animation_game::initial_phase(&v.kind);
                            if self.map.original{self.units[vi].squad=if v.kind=="apc"{u.squad}else{self.units.iter().map(|p|p.squad).max().unwrap_or(0)+1};}
                            if let Some(state)=&mut self.units[vi].original_combat{state.crew_boarded(&u.kind);}
                            if u.initiative==Some(original_initiative::Initiative::Vehicle(v.id)) {
                                self.resume_march(vi,u.march_resume);
                            }
                        }else{self.units[vi].cargo.push(Crew{kind:u.kind.clone(),hp:u.hp});}
                        self.units[i].hp=if self.map.original{-1.}else{0.};
                        self.events.push(json!({"kind":"crewed","team":u.team,"type":v.kind}));break 'unit;
                    }
                }
                if let Some(point)=goal {
                    if self.units[i].order==u.order&&d.speed>0.&&(u.path.is_empty()||u.dest.is_none_or(|old|(old.0-point.0).hypot(old.1-point.1)>=16.)) {
                        self.units[i].path=self.route_unit(&u,point.0,point.1);self.units[i].dest=Some(point);
                    }
                }
            }
            // An aimed target takes precedence over opportunistic fire once it
            // is reachable by the weapon. Outside that range the existing
            // pursuit and nearby return fire continue unchanged.
            let building_target_in_range = building_hold;
            let explicit_target = if u.order == "attack" { u.target } else { None };
            // A native attack animation owns its acquired target until its
            // reacquisition transition. A newly reachable enemy must not reset
            // a partly completed windup on every callback.
            let locked=u.native_fire.as_ref().filter(|s|!s.reacquire).map(|s|s.target);
            let native_manual_point=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref).is_some_and(|m|m.flags69&32!=0);
            let native_candidate=if self.map.original&&!native_manual_point&&!building_target_in_range&&!wall_hold&&!bridge_hold&&u.kind!="apc"{
                locked.or(explicit_target).and_then(|id|self.units.iter().find(|v|v.id==id&&v.team>=0&&v.team!=u.team
                    &&original_combat::alive(true,v.hp)&&!workshops::hidden(v)&&self.native_target_in_range(&u,v,d.range)&&self.unit_target_los(&u,v))).cloned()
                    .or_else(||self.native_scan(i,0,d.range))
            }else{None};
            let enemy = if self.map.original{native_candidate.as_ref()}else{snapshot
                .iter()
                .filter(|v| {
                    !building_target_in_range
                        && !wall_hold
                        && !bridge_hold
                        && original_combat::alive(self.map.original,v.hp)
                        && !workshops::hidden(v)
                        && v.team >= 0
                        && v.team != u.team
                        && (v.x - u.x).hypot(v.y - u.y) <= d.range
                        && self.unit_target_los(&u,v)
                })
                .min_by(|a, b| {
                    (Some(b.id)==locked).cmp(&(Some(a.id)==locked))
                        .then_with(||(Some(b.id) == explicit_target).cmp(&(Some(a.id) == explicit_target)))
                        .then_with(|| ((a.x - u.x).hypot(a.y - u.y)).total_cmp(&(b.x - u.x).hypot(b.y - u.y)))
                })};
            if let Some(v) =
                enemy.filter(|_| !(d.cls == 0 && u.order == "move" && !u.path.is_empty()))
            {
                if !self.map.original||d.cls==0{self.units[i].aim = (v.y - u.y).atan2(v.x - u.x);}
                let release=if self.map.original {self.advance_native_fire(i,v)}else{self.units[i].cooldown<=0.};
                if d.damage > 0. && release {
                    if self.map.original&&matches!(self.units[i].native_class7c,1|2){let rate=self.units[i].native_hardware_slot.and_then(|s|self.native_hardware_pool.get(s)).map_or(0,|w|w.attributes.byte(32));let reload=100u8.wrapping_sub(rate)as i8;self.units[i].native_reload7b=reload;self.units[i].cooldown=f64::from(reload as u8)*original_motion::TICK_SECONDS;}else{self.units[i].cooldown = d.reload;}
                    let grenade = self.map.original&&self.units[i].native_fire.as_ref().is_some_and(|s|s.mode==original_fire::Mode::Grenade);
                    let sniper = u.kind == "sniper" && stats(&v.kind).cls > 0 && v.crew.is_some();
                    let amt = if !self.map.original && u.grenades > 0 && stats(&v.kind).cls > 0 {
                        self.units[i].grenades -= 1;
                        d.damage + 38.
                    } else {
                        d.damage
                    };
                    if self.map.original {
                        let mut attack=original_combat::Attack::from_unit(&u,v.id,if grenade{50.}else{amt},self.units[i].aim);
                        if grenade{attack.bullet=false;}
                        if attack.bullet {self.launch_native_bullet(i,v,attack);}else{self.launch_native_heavy(i,v,attack,grenade);}
                    } else {damage.push((v.id, amt, sniper));}
                    if !self.map.original{self.effects.push(Fx {
                        x: u.x,
                        y: u.y,
                        age: 0.,
                        life: 0.14,
                        kind: "shot".into(),
                        radius: 2.,
                        x2: v.x,
                        y2: v.y,
                    });}
                    self.events.push(json!({"kind":"shot","type":u.kind,"weapon":if grenade{"grenade"}else{"primary"},"x":u.x,"y":u.y,"team":u.team}));
                }
            } else if u.order == "attackBuilding" && building_hold {
                if let Some(b)=u.target.and_then(|id|self.buildings.iter().find(|b|b.id==id&&b.team!=u.team&&b.alive(self.map.original))).cloned() {
                    if self.map.original&&(walls::capable(&u)||walls::apc_point_capable(&u)) {
                        let point=(u.x.clamp(b.x+8.,b.x+b.w-8.),u.y.clamp(b.y+8.,b.y+b.h-8.));
                        self.units[i].aim=(point.1-u.y).atan2(point.0-u.x);
                        self.fire_native_ground(i,point,b.id,!walls::primary_capable_unit(&u)&&!walls::apc_point_capable(&u));
                    }else if !self.map.original&&self.units[i].cooldown<=0.&&d.damage>0. {
                        self.units[i].cooldown=d.reload;bdamage.push((b.id,d.damage,u.team));
                        self.events.push(json!({"kind":"shot","type":u.kind,"x":u.x,"y":u.y,"team":u.team}));
                    }
                }
            }
            if self.map.original&&((enemy.is_none()&&!wall_hold&&!building_hold&&!bridge_hold)||d.cls==0&&u.order=="move"&&!u.path.is_empty()) {
                let ended=self.units[i].native_fire.take().is_some();if ended&&d.cls==0{self.units[i].native_phase35=if self.units[i].path.is_empty(){0}else{2};}
            }

            let base_speed=d.speed*if !self.map.original&&d.cls==0&&u.order=="move"{1.2}else{1.};
            // Revalidate the next segment: bridges and map occupancy can change
            // after the route was planned. Never interpolate across forbidden water.
            if let Some(&(nx,ny))=self.units[i].path.front() {
                if !self.movement_segment_clear_unit(&self.units[i],(nx,ny)) {
                    let goal=self.units[i].dest.or_else(||self.units[i].path.back().copied());
                    self.units[i].path=goal.map(|(x,y)|if u.order=="attackWall"{u.target.and_then(|id|self.wall(id)).map(|wall|self.route_to_wall_attack(&u,wall,d.range)).unwrap_or_default()}else if u.order=="attack"&&u.target.is_some(){u.target.and_then(|id|self.units.iter().find(|v|v.id==id)).map(|v|self.route_to_unit_attack(&u,v,d.range)).unwrap_or_default()}else if u.order=="attackBuilding"{u.target.and_then(|id|self.buildings.iter().find(|b|b.id==id)).map(|b|self.route_to_building_attack(&u,b,d.range)).unwrap_or_default()}else{self.route_unit(&u,x,y)}).unwrap_or_default();
                }
            }
            let hold = pursuit_hold || wall_hold || building_hold || bridge_hold || enemy.is_some() && u.order == "guard"
                || self.map.original&&d.cls==0&&(u.native_fire.is_some()||self.units[i].native_fire.is_some());
            if !hold && base_speed > 0. && self.map.original {
                self.move_original(i,first_motion_tick,motion_ticks,enemy.is_none()&&d.cls==0);
                if self.native_marching_object_scan(i){break 'unit;}
            } else if !hold && base_speed > 0. {
                // Waypoints are geometry, not pauses: carry unused movement
                // time into the next leg, instead of losing part of every
                // update at every tile center. Terrain is re-evaluated there.
                let mut remaining=dt;
                for _ in 0..128 {
                    let Some((x,y))=self.units[i].path.front().copied()else{break;};
                    let(cx,cy)=(self.units[i].x,self.units[i].y);
                    if !self.movement_segment_clear_for(cx,cy,x,y,&u.kind){break;}
                    let dist=(x-cx).hypot(y-cy);
                    if dist<=1e-9{self.units[i].path.pop_front();continue;}
                    if remaining<=1e-10{break;}
                    let speed=base_speed*self.terrain_factor_unit(&self.units[i]);
                    if speed<=0.{break;}
                    let traveled=(speed*remaining).min(dist);
                    self.units[i].angle=(y-cy).atan2(x-cx);
                    self.units[i].x=cx+(x-cx)/dist*traveled;
                    self.units[i].y=cy+(y-cy)/dist*traveled;
                    self.units[i].moving=true;
                    if enemy.is_none(){self.units[i].aim=self.units[i].angle;}
                    remaining=(remaining-traveled/speed).max(0.);
                    if traveled>=dist-1e-9 {
                        self.units[i].x=x;self.units[i].y=y;self.units[i].path.pop_front();
                    }else{break;}
                }
            }


          }
          self.finish_native_unit_callback(i,native_display_slot);
          self.capture_native_unit_display(i,native_display_slot);
        }
        if self.map.original {
            self.maintain_native_group_rosters();
            self.update_original_tactics();
            let native_due=self.original_ai.as_mut().map(|c|c.advance(self.motion_tick));
            if native_due.unwrap_or(self.ai_clock<=0.){self.ai();self.ai_clock=[8.,4.5,2.5,1.3][self.difficulty];}
            self.update_native_vehicle_spacing();self.capture_native_flag_display();self.update_native_route_worker();self.update_native_deaths();self.update_native_world_objects();
            self.update_native_blasts();
            self.update_native_cosmetics20();
            self.update_native_ambient_animations();
            self.update_native_ambient();
            self.update_native_lava();
            self.advance_native_fort_fragments();
            self.update_native_world_effects();
            self.update_native_bullets();self.update_native_projectiles();if let Some(result)=self.native_trail_scene_result(){self.update_native_building_trails(result);}self.advance_native_feedback_clock();
        }
        for b in &self.buildings {
            if b.kind == "fort" && (if self.map.original{b.hp>=0.}else{b.alive(self.map.original)}) {
                if let Some(u) = self.units.iter().find(|u| {
                    original_combat::alive(self.map.original,u.hp)
                        && !workshops::hidden(u)
                        && u.team >= 0
                        && u.team != b.team
                        && stats(&u.kind).cls == 0
                        && (u.x - b.gate.unwrap_or((b.x+b.w/2.,b.y+b.h+5.)).0).abs() < 24.
                        && (u.y - b.gate.unwrap_or((b.x+b.w/2.,b.y+b.h+5.)).1).abs() < 24.
                }) {
                    self.winner = Some(u.team);
                }
            }
        }
        if let Some(w) = self.winner {
            self.events.push(json!({"kind":"victory","team":w}));
            return;
        }
        for (id, amount, sniper) in damage {
            if let Some(i) = self.units.iter().position(|u| u.id == id && u.hp > 0. && !workshops::hidden(u)) {
                if sniper {
                    if let Some(c) = &mut self.units[i].crew {
                        c.hp -= amount;
                        if c.hp <= 0. {
                            self.units[i].team = -1;
                            self.units[i].crew = None;
                            self.units[i].path.clear();
                        }
                    }
                } else {
                    self.units[i].hp -= amount;
                }
                if self.units[i].hp <= 0. {
                    let u = &self.units[i];
                    self.effects.push(Fx {
                        x: u.x,
                        y: u.y,
                        age: 0.,
                        life: 0.8,
                        kind: "explosion".into(),
                        radius: if stats(&u.kind).cls == 0 { 12. } else { 30. },
                        x2: 0.,
                        y2: 0.,
                    });
                    self.events.push(json!({"kind":"explosion","x":u.x,"y":u.y,"radius":if stats(&u.kind).cls==0 {12.} else {30.}}));
                }
            }
        }
        for (id, amount, attacker) in bdamage {
            if let Some(b)=self.buildings.iter_mut().find(|b|b.id==id&&b.alive(self.map.original)) {
                b.hp=(b.hp-amount).max(0.);
                if b.hp<=0. {self.events.push(json!({"kind":"buildingLost","team":b.team}));if b.kind=="fort"{self.winner=Some(attacker);}}
            }
        }
        // The public play guide describes immediate flag transfer, including an almost finished product.
        let mut captured_sectors=Vec::new();
        for s in &mut self.sectors {
            if self.map.original&&self.map.native_road_graph.is_some(){continue;}
            if s.home {
                continue;
            }
            if let Some(u) = self
                .units
                .iter()
                .find(|u| original_combat::alive(self.map.original,u.hp) && !workshops::hidden(u) && u.team >= 0 && (u.x - s.x).hypot(u.y - s.y) < 22.)
            {
                if s.owner != u.team {
                    let prev = s.owner;
                    s.owner = u.team;captured_sectors.push(s.id);
                    if let Some(strategy)=self.native_strategy.as_mut(){strategy.captured(u.team,s.id);}
                    for b in self.buildings.iter_mut().filter(|b| b.sector == s.id) {
                        b.team = u.team;
                    }
                    self.events.push(
                        json!({"kind":"capture","team":u.team,"previous":prev,"sector":s.id}),
                    );
                }
            }
        }

        for sector in captured_sectors{self.clear_native_hardware_restrictions(sector);if self.map.original{self.initialize_native_capture_defaults(sector);self.rebuild_native_scene_flags();}}
        if !self.map.original{self.ensure_production();}
        if !self.map.original{self.update_production(dt);}
        self.advance_native_flag_counters();
        for team in [0, 1] {
            if !self.units.iter().any(|u| original_combat::alive(self.map.original,u.hp) && u.team == team) {
                self.winner = Some(1 - team);
            }
        }
        let original=self.map.original;
        let retained_drivers=if original{self.units.iter().filter(|u|u.team>=0&&original_combat::alive(true,u.hp)&&u.crew.is_some()).filter_map(|u|u.transport_actor).collect::<std::collections::HashSet<_>>()}else{Default::default()};
        self.units.retain(|u| original_combat::alive(original,u.hp)||retained_drivers.contains(&u.id));
        if let Some(w) = self.winner {
            self.events.push(json!({"kind":"victory","team":w}));
        }
    }
}
thread_local! {static GAME:RefCell<Option<Game>>=const{RefCell::new(None)};static OUT:RefCell<Vec<u8>>=const{RefCell::new(Vec::new())};}
fn request_team(v: &Value) -> Result<i32, &'static str> {
    match v.get("team") {
        None | Some(Value::Null) => Ok(0),
        Some(team) => match team.as_i64() {
            Some(team @ 0..=1) => Ok(team as i32),
            _ => Err("Invalid player team"),
        },
    }
}

/// Static map and resource-independent descriptors are unchanged in almost all
/// ticks. Do not serialize their large arrays just to discard them in JavaScript.
fn game_snapshot(g: &Game, request: &Value) -> Value {
    let compact = request["compact"].as_bool().unwrap_or(false);
    let known_seed = request["knownMapSeed"].as_u64() == Some(g.map.seed as u64);
    let known_revision = request["knownMapRevision"].as_u64() == Some(g.map.revision as u64);
    let include_static = !compact || request["op"] == "init" || request["includeStatic"] == true || !known_seed;
    let include_map = include_static || !known_revision;
    let visible_units=g.units.iter().filter(|u|!g.map.original||original_combat::alive(true,u.hp)).collect::<Vec<_>>();
    let mut value = json!({
        "sectors": &g.sectors, "buildings": &g.buildings, "units":visible_units,
        "nativeDisplayQueue":g.native_display_queue(),"nativeTrailEffects":g.native_building_trail_draws(),"nativeEffects":g.native_world_effects(),"nativeDeathVisuals":g.native_death_visuals(),"nativeAmbientVisuals":g.native_ambient_visuals(),"nativeAmbientAnimations":g.native_ambient_animation_visuals(),"nativeCosmetic20Visuals":g.native_cosmetic20_visuals(),"nativeLavaVisuals":g.native_lava_visuals(),"nativeTrackVisuals":g.native_track_visuals(),"nativeObjectVisuals":g.native_object_visuals(),"bullets": &g.bullets,"projectiles":&g.projectiles,"blasts":&g.blasts,"fires":&g.fires, "effects": &g.effects, "crates": &g.crates, "events": &g.events, "walls": &g.walls,
        "time": g.time, "winner": g.winner, "difficulty": g.difficulty,
        "mission": g.mission, "multiplayer": g.multiplayer,
        "mapSeed": g.map.seed, "mapRevision": g.map.revision,
        "productionSeconds": g.buildings.iter().map(|b| g.remaining_seconds(b)).collect::<Vec<_>>(),
        "buildMenus": g.buildings.iter().map(|b| g.products(b)).collect::<Vec<_>>(),
    });
    if g.map.original{value["nativeRouteBoundary"]=g.native_route_boundary.map(|b|format!("{b:?}")).into();value["nativeUiClock"]=json!(g.native_feedback.ui_clock);}
    if g.map.original&&!g.multiplayer{
        let f=&g.native_feedback;let speaker=f.display_speaker.and_then(|slot|g.units.iter().rfind(|u|u.native_slot==Some(slot)));
        value["nativePortrait"]=json!({"mode":f.portrait_mode,"message":f.current_message,"phraseIndex":f.phrase_index,"phraseCounter":f.phrase_counter,"phraseFrame":f.phrase_frame,"unitId":speaker.map(|u|u.id),"nativeSlot":f.display_speaker,"rawSpeakerSlot":f.speaker});
    }
    if g.map.original&&g.multiplayer{value["nativePortraits"]=json!([g.native_portrait(0),g.native_portrait(1)]);value["nativePortrait"]=g.native_portrait(0);}
    if include_map { value["map"] = serde_json::to_value(&g.map).unwrap();value["objects"]=json!(&g.objects); }
    if include_static {
        value["catalog"] = json!(TYPES.iter().map(|k| { let mut d = stats(k);if g.map.original {d.speed=original_combat::movement_speed(k);} json!({"type":k,"hp":d.hp,"range":d.range,"damage":d.damage,"speed":d.speed,"seconds":d.seconds,"members":d.members,"stars":d.stars,"workload":original_stats::workload(k),"class":d.cls,"canDestroyWalls":walls::primary_capable(k),"wallGrenades":walls::grenade_capable(k),"requirements":{"vehicle":original_stats::required(k,"vehicle"),"robot":original_stats::required(k,"robot"),"fort":original_stats::required(k,"fort")}}) }).collect::<Vec<_>>());
    }
    if let Some(units)=value["units"].as_array_mut(){for u in units{if u["team"]==-1{if let Some(sprite)=u["id"].as_u64().and_then(|id|g.native_neutral_unit_sprite(id as usize)){u["nativeObjectSprite"]=json!(sprite);}}}}
    if let Some(buildings)=value["buildings"].as_array_mut(){for b in buildings{if g.map.original&&b["kind"]=="fort"{b["nativeFortDestroyed"]=json!(b["hp"].as_f64().is_some_and(|hp|hp<0.));}if b["hp"].as_f64().is_some_and(|hp|hp<0.){b["hp"]=json!(0);}}}
    value
}

pub fn request(v: Value) -> Value {
    GAME.with(|cell| {
        let mut state = cell.borrow_mut();
        if v["op"] == "init" {
            let multiplayer = v["multiplayer"].as_bool().unwrap_or(false);
            let difficulty = if multiplayer { 1 } else { v["difficulty"].as_u64().unwrap_or(1) as usize };
            let mut g = Game::new(v["mission"].as_u64().unwrap_or(0) as usize, difficulty, v["seed"].as_u64().unwrap_or(7163) as u32);
            g.multiplayer = multiplayer;
            g.native_seasonal_mode=u8::from(v["nativeCalendar"]["month"].as_i64()==Some(11)&&v["nativeCalendar"]["day"].as_i64()==Some(25));
            g.native_cosmetic_rng=state.as_ref().map_or(v["nativeCosmeticSeed"].as_u64().unwrap_or(1)as u32,|previous|previous.native_cosmetic_rng);
            if !v["originalMission"].is_null() {
                g.original_rng=original_combat::OriginalRng::for_original_mission(state.as_ref().filter(|p|p.map.original).map(|p|&p.original_rng),multiplayer);
                if let Some(previous)=state.as_mut(){g.native_routes.get_mut().road_scratch=std::mem::take(&mut previous.native_routes.get_mut().road_scratch);}
                if let Some(previous)=state.as_mut(){g.native_world=previous.native_world.take();g.native_smoke20=std::mem::take(&mut previous.native_smoke20);g.native_bullet_impacts20=std::mem::take(&mut previous.native_bullet_impacts20);g.native_fire_base_strength=previous.native_fire_base_strength;g.native_hardware_pool=std::mem::take(&mut previous.native_hardware_pool);g.native_ambient=previous.native_ambient.take();g.native_lava=std::mem::take(&mut previous.native_lava);g.native_lava_sound_tick=previous.native_lava_sound_tick;g.native_ambient_animations=std::mem::take(&mut previous.native_ambient_animations);g.native_order_arguments=previous.native_order_arguments;g.native_feedback=std::mem::take(&mut previous.native_feedback);g.native_path_contact=std::mem::take(&mut previous.native_path_contact);g.native_query_shape=previous.native_query_shape;g.native_audio=std::mem::take(&mut previous.native_audio);g.native_listener_audio=std::mem::take(&mut previous.native_listener_audio);g.native_production_limit=previous.native_production_limit;g.native_blast_sound_gate=previous.native_blast_sound_gate;g.native_blast_sound_cycle=previous.native_blast_sound_cycle;g.native_blast_terrain_class=previous.native_blast_terrain_class;g.native_protected_order_seen=previous.native_protected_order_seen;g.native_wait_clock=previous.native_wait_clock;g.native_flag_frame=previous.native_flag_frame;g.native_object_frame=previous.native_object_frame;g.native_frame16=previous.native_frame16;g.native_attack_ray_building=previous.native_attack_ray_building;g.native_selection=std::mem::take(&mut previous.native_selection);g.native_hardware_dummy=previous.native_hardware_dummy;g.native_hardware_dummy_attributes=previous.native_hardware_dummy_attributes;g.native_factory_placement=std::mem::take(&mut previous.native_factory_placement);g.native_group_rosters=std::mem::take(&mut previous.native_group_rosters);g.native_pressure=std::mem::take(&mut previous.native_pressure);}
                if let Err(e) = g.apply_original(&v["originalMission"]) { return json!({"error":e}); }
            }
            if multiplayer { g.ai_teams.clear(); }
            else if let Some(a) = v["aiTeams"].as_array() { g.ai_teams = a.iter().filter_map(|x| x.as_i64().filter(|x| (0..=1).contains(x)).map(|x| x as i32)).collect(); }
            if g.original_ai.is_some()&&!multiplayer {
                if g.native_strategy.is_some(){g.update_original_strategy(true);
                    //Initial source selectors run only for authoredFF products.
                    let empty=v["originalMission"]["buildings"].as_array().into_iter().flatten().filter(|b|b["product"].is_null()).filter_map(|b|b["id"].as_u64().map(|n|n as usize)).collect::<std::collections::BTreeSet<_>>();
                    for index in 0..g.buildings.len(){if empty.contains(&g.buildings[index].id)&&g.buildings[index].team==1&&g.buildings[index].completion_ticks==0{let old=g.buildings[index].product.take();if let Some(product)=g.native_cpu_factory_product(index,false,0){g.buildings[index].remaining_work=Some(original_stats::workload(&product)as f64);g.buildings[index].product=Some(product);}else{g.buildings[index].product=old;}}}
                }else{g.ai();}
            }
            *state = Some(g);
        }
        let Some(g) = state.as_mut() else { return json!({"error":"Initialize first"}); };
        let requested_team = if matches!(v["op"].as_str(), Some("command" | "produce" | "placeGun" | "previewGun")) {
            match request_team(&v) { Ok(team) => team, Err(error) => return json!({"error":error}) }
        } else { 0 };
        // Single-player gun/production endpoints historically belong to red.
        // A client-supplied team becomes meaningful only in an explicit PvP game.
        let team = if !g.multiplayer && matches!(v["op"].as_str(), Some("produce" | "placeGun" | "previewGun")) { 0 } else { requested_team };
        let id = v["id"].as_u64().and_then(|id| usize::try_from(id).ok()).unwrap_or(usize::MAX);
        let slot = v.get("slot").map(|n| n.as_u64().and_then(|i| usize::try_from(i).ok()).unwrap_or(usize::MAX)).unwrap_or(0);
        let x = v["x"].as_f64().unwrap_or(f64::NAN);
        let y = v["y"].as_f64().unwrap_or(f64::NAN);
        if g.map.original&&v["op"]!="init"&&v["op"]!="uiTick"&&v.get("nativeSelected").is_some(){let previous=g.native_selected_slot;g.native_selected_slot=if g.multiplayer{None}else{v["nativeSelected"].as_u64().and_then(|id|usize::try_from(id).ok()).and_then(|id|g.units.iter().find(|u|u.id==id&&u.team==0&&original_combat::alive(true,u.hp))).and_then(|u|u.native_slot)};if g.native_selected_slot.is_some()&&g.native_selected_slot!=previous{g.native_feedback_selection_changed();}}
        if g.map.original {
            if !g.multiplayer{if let Some(enabled)=v["nativeAudio"]["soundEnabled"].as_bool(){g.native_feedback.sound_enabled=u8::from(enabled);}}else{g.set_native_listener_settings(&v["nativeListenerAudio"]);}

            let read_pair=|name:&str|->Option<[i16;2]>{let p=v["nativeView"][name].as_array()?;if p.len()!=2{return None;}Some([i16::try_from(p[0].as_i64()?).ok()?,i16::try_from(p[1].as_i64()?).ok()?])};
            if let (Some(camera),Some(size))=(read_pair("camera"),read_pair("size")){if size[0]>0&&size[1]>0{g.native_view=Some((camera,size));g.set_native_world_effect_view(camera,[0,0,size[0],size[1]]);}}
        }
        if v["op"] == "previewGun" {
            if let (Some(bases),Some((camera,_)))=(g.native_fort_bases(),g.native_view){g.record_native_gun_placement_view(bases[1],camera);}
            return serde_json::to_value(g.gun_placement_preview(id, slot, x, y, team)).unwrap();
        }
        if v["op"] != "snapshot" { g.events.clear(); }
        let mut placement = None;
        match v["op"].as_str().unwrap_or("") {
            "tick" => g.step(v["dt"].as_f64().unwrap_or(0.04)),
            "audioSettings" => {},
            "uiTick" => g.step_native_ui_only(v["dt"].as_f64().unwrap_or(0.)),
            "advance" => for _ in 0..v["steps"].as_u64().unwrap_or(1).min(30000) { g.step(0.1); },
            "command" => g.command(&v),
            "produce" => { g.produce_for_team(id, v["product"].as_str(), team); },
            "placeGun" => placement = Some(g.place_gun_for_team(id, slot, x, y, team)),
            _ => {},
        }
        let mut value = game_snapshot(g, &v);
        if let Some(result) = placement { value["placementResult"] = serde_json::to_value(result).unwrap(); }
        value
    })
}

#[no_mangle]
pub extern "C" fn alloc(n: usize) -> *mut u8 {
    let b = vec![0u8; n].into_boxed_slice();
    Box::into_raw(b) as *mut u8
}
#[no_mangle]
pub unsafe extern "C" fn free(p: *mut u8, n: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(p, n)));
}
#[no_mangle]
pub unsafe extern "C" fn run(p: *const u8, n: usize) -> usize {
    let v = serde_json::from_slice(std::slice::from_raw_parts(p, n));
    let value = match v {
        Ok(v) => request(v),
        Err(e) => json!({"error":e.to_string()}),
    };
    OUT.with(|o| {
        *o.borrow_mut() = serde_json::to_vec(&value).unwrap();
        o.borrow().len()
    })
}
#[no_mangle]
pub extern "C" fn output() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod fort_mount_tests;
#[cfg(test)]
mod production_tests;
#[cfg(test)]
mod gun_placement_tests;
#[cfg(test)]
mod attack_target_tests;
#[cfg(test)]
mod pursuit_tests;
#[cfg(test)]
mod terrain_tests;
#[cfg(test)]
mod route_quality_tests;
#[cfg(test)]
mod multiplayer_tests;
#[cfg(test)]
mod wall_tests;
#[cfg(test)]
mod workshop_tests;

fn production_sector()->bool{true}

fn advance_random(rng:&mut u32)->u32 {
    *rng^=*rng<<13;*rng^=*rng>>17;*rng^=*rng<<5;*rng
}

#[cfg(test)]
mod audit_order_tests;
#[cfg(test)]
mod tank_lifecycle_tests;
#[cfg(test)]
mod march_initiative_tests;
#[cfg(test)]
mod speed_bridge_tests;

/// Original objects and occupied tanks use different native origins. Boarding
/// reads004967e0 by object type (00450db0/00456c90); Heavy also adds00496934
/// in00450cd0. Unloading reads004970b0 by occupied family, then subtracts
/// another four native X pixels at00458b8b before passing X to00458a40.
/// World coordinates here use two pixels per native pixel. Existing occupied
/// mission spawns already have occupied origins.
fn original_tank_origin_offset(kind:&str,boarding:bool)->(f64,f64) {
    match (kind,boarding) {
        ("medium",true)=>(16.,12.),("medium",false)=>(-16.,-12.),
        ("heavy",true)=>(12.,8.),("heavy",false)=>(-12.,-8.),
        ("light",true)=>(22.,14.),("light",false)=>(-22.,-14.),
        _=>(0.,0.),
    }
}
