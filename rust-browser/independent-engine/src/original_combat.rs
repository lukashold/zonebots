//! Verified translations of the local GOG Z.exe damage kernels.
//! Function/address evidence is recorded in the private local Ghidra research.
//! This module does not replace trajectory collision: a successful aim roll
//! changes target coordinates; the travelling projectile must still collide.
//! The engine currently resolves contacts instantly. Acquisition scattering,
//! shell blast geometry, hatch animation and full original RNG ordering remain
//! separate work; no acquisition roll is converted into a guaranteed miss.

use super::Unit;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatState {
    pub aim_skill: u8,
    pub movement_control: u8,
    pub fire_rate: u8,
    /// Original hardware+0x19. The engine leaves the hatch closed until its
    /// secondary weapon animation is implemented; external saves can retain it.
    pub secondary_state: i8,
    pub immune: bool,
}
#[derive(Clone, Copy)]
struct Profile {
    class: u8,
    family: u8,
    aim: u8,
    movement: u8,
    fire: u8,
    crew_hp: u8,
    bullet: bool,
}
fn profile(kind: &str) -> Profile {
    let (class, family, aim, movement, fire, crew_hp, bullet) = match kind {
        "grunt" => (0, 4, 60, 40, 80, 30, true),
        "tough" => (3, 4, 40, 30, 80, 100, false),
        "sniper" => (0, 4, 120, 50, 80, 50, true),
        "psycho" => (0, 4, 50, 40, 98, 50, true),
        "pyro" => (15, 4, 80, 40, 98, 80, false),
        "laser" => (16, 4, 40, 50, 80, 60, true),
        "cannon" => (2, 5, 60, 0, 60, 30, false),
        "medium" => (1, 6, 80, 40, 70, 30, false),
        "heavy" => (1, 25, 80, 30, 60, 30, false),
        "light" => (1, 30, 80, 50, 80, 30, false),
        "launcher" => (1, 24, 80, 20, 20, 30, false),
        "missile" => (4, 21, 60, 0, 30, 30, false),
        "gatling" => (5, 22, 50, 0, 97, 30, true),
        "howitzer" => (6, 23, 60, 0, 10, 30, false),
        "jeep" => (1, 34, 40, 60, 98, 30, true),
        "apc" => (1, 32, 60, 50, 80, 30, false),
        "crane" => (1, 36, 60, 50, 50, 40, false),
        "truck" => (1, 38, 60, 60, 0, 30, false),
        "firetruck" => (1, 38, 60, 50, 98, 40, false),
        _ => (0, 4, 60, 40, 80, 30, true),
    };
    Profile {
        class,
        family,
        aim,
        movement,
        fire,
        crew_hp,
        bullet,
    }
}
impl CombatState {
    pub fn new(kind: &str) -> Self {
        let p = profile(kind);
        Self {
            aim_skill: p.aim,
            movement_control: p.movement,
            fire_rate: p.fire,
            secondary_state: -1,
            immune: false,
        }
    }
    pub fn speed_scale(&self, kind: &str) -> f64 {
        let base = profile(kind).movement;
        if base == 0 {
            1.
        } else {
            f64::from(self.movement_control) / f64::from(base)
        }
    }
    pub fn reload_seconds(&self) -> f64 {
        f64::from(100 - self.fire_rate.min(100)) / 17.
    }
    pub fn crew_boarded(&mut self, kind: &str) {
        self.aim_skill = profile(kind).aim;
    }
}
pub fn initial_crew_hp(kind: &str) -> f64 {
    f64::from(profile(kind).crew_hp)
}
pub fn runtime_class(kind: &str) -> u8 {
    profile(kind).class
}
pub fn alive(original: bool, hp: f64) -> bool {
    if original {
        hp >= 0.
    } else {
        hp > 0.
    }
}
pub fn facing(angle: f64) -> u8 {
    // Original0 points north,2east,4south,6west. Engine angle0 points east.
    ((angle / std::f64::consts::FRAC_PI_4).round() as i32 + 2).rem_euclid(8) as u8
}
#[derive(Clone, Copy)]
pub struct Attack {
    pub target: usize,
    pub damage: u8,
    pub aim: u8,
    pub facing: u8,
    pub bullet: bool,
}
impl Attack {
    pub fn from_unit(unit: &Unit, target: usize, damage: f64, angle: f64) -> Self {
        Self {
            target,
            damage: damage.clamp(0., 255.) as u8,
            aim: unit
                .original_combat
                .as_ref()
                .map(|s| s.aim_skill)
                .unwrap_or(profile(&unit.kind).aim),
            facing: facing(angle),
            bullet: profile(&unit.kind).bullet,
        }
    }
}
/// Adapter from the engine's serialized unit representation to the verified
/// kernels below. Returns true only for a crew kill that leaves usable hull.
pub fn apply_attack(
    unit: &mut Unit,
    attack: Attack,
    difficulty: usize,
    next_random: impl FnMut() -> u32,
) -> bool {
    let p = profile(&unit.kind);
    let mechanical = super::stats(&unit.kind).cls > 0;
    let state = unit
        .original_combat
        .get_or_insert_with(|| CombatState::new(&unit.kind));
    let mut target = Target {
        runtime_class: p.class,
        animation_family: p.family,
        facing: facing(unit.angle),
        secondary_state: state.secondary_state,
        human_team_zero: unit.team == 0,
        immune: state.immune,
        crew_hp: if mechanical {
            unit.crew.as_ref().map(|c| c.hp as i8).unwrap_or(0)
        } else {
            unit.hp as i8
        },
        hull_hp: mechanical.then_some(unit.hp as i16),
    };
    let old_hull = target.hull_hp;
    let difficulty = match difficulty {
        0 => Difficulty::Easy,
        1 => Difficulty::Normal,
        _ => Difficulty::Hard,
    };
    let result = if attack.bullet {
        resolve_bullet_contact(
            &mut target,
            attack.aim,
            attack.facing,
            attack.damage,
            difficulty,
            false,
            next_random,
        )
    } else {
        apply_body_damage(&mut target, attack.damage, difficulty, false)
    };
    if result.route == DamageRoute::Body && !state.immune {
        if let Some(mut hull) = old_hull {
            for _ in 0..result.calls {
                if let Some(factor) =
                    hull_degrade_factor(hull, unit.max_hp as i16, result.damage_per_call)
                {
                    state.movement_control =
                        (u16::from(state.movement_control) * u16::from(factor) / 100) as u8;
                    state.aim_skill = (u16::from(state.aim_skill) * u16::from(factor) / 100) as u8;
                    if p.family != 22 && p.family != 34 {
                        state.fire_rate =
                            (u16::from(state.fire_rate) * u16::from(factor) / 100) as u8;
                    }
                }
                hull = hull.wrapping_sub(i16::from(result.damage_per_call));
            }
        }
    }
    if mechanical {
        unit.hp = f64::from(target.hull_hp.unwrap());
        if let Some(crew) = &mut unit.crew {
            crew.hp = f64::from(target.crew_hp);
        }
        if result.route == DamageRoute::Crew && result.crew_killed {
            unit.team = -1;
            unit.crew = None;
            unit.path.clear();
            unit.dest = None;
            unit.target = None;
            unit.order = "guard".into();
            unit.moving = false;
            return true;
        }
    } else {
        unit.hp = f64::from(target.crew_hp);
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

/// Unit+0x7c is the runtime class, NOT equipment ID or group ID.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub runtime_class: u8,
    pub animation_family: u8,  // unit+0x30
    pub facing: u8,            // unit+0x39, eight directions
    pub secondary_state: i8,   // hardware+0x19; 3 exposes crew in most tanks
    pub human_team_zero: bool, // unit+0x31 low2bits == 0
    pub immune: bool,          // unit+0x9c &0x14 !=0
    pub crew_hp: i8,           // unit+0x3a, -1 is encoded dead
    pub hull_hp: Option<i16>,  // hardware+8; zero STILL survives this kernel
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageRoute {
    Crew,
    Body,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageOutcome {
    pub route: DamageRoute,
    pub damage_per_call: u8,
    pub calls: u8,
    pub crew_killed: bool,
    pub hull_destroyed: bool,
}

/// Original 0x460910 multiply-with-carry RNG. Words are the original state at
/// globals0x622cdc,+4,+8,+12,+16. Initialization/seeding is intentionally caller
/// supplied. In the game's multiplayer mode this function returns zero.
pub struct OriginalRng {
    pub words: [u32; 5],
    pub multiplayer: bool,
}
impl OriginalRng {
    pub fn next_u32(&mut self) -> u32 {
        if self.multiplayer {
            return 0;
        }
        let [a, b, c, d, carry] = self.words;
        let sum = u64::from(a) * 0x13fb
            + u64::from(b) * 0x6f0
            + u64::from(c) * 0x5d4
            + u64::from(d) * 0x7dd4ffc7
            + u64::from(carry);
        self.words = [sum as u32, a, b, c, (sum >> 32) as u32];
        (a & 0xffff0000) | ((sum as u32) & 0xffff)
    }
}

/// 0x456950: returns true when acquisition roll succeeds. Normally the game
/// uses aim+floor(aim/4), without clamping to127. Flag0x69 bit0x80 uses base aim.
/// Comparison is inclusive: aim60 gives threshold75, hence76/128 successful
/// rolls. A miss can still collide with a target after coordinate scattering.
pub fn aim_roll_hits(aim: i8, flag_69_bit80: bool, random_word: u32) -> bool {
    let aim = i16::from(aim);
    let threshold = if flag_69_bit80 { aim } else { aim + (aim >> 2) };
    (random_word & 127) as i16 <= threshold
}

/// 0x462a70 geometric contact bounds, in original tile16 world coordinates.
/// The bullet position is compared against unit position after subtracting8.
pub fn bullet_overlaps(target: &Target, bullet_xy: [i16; 2], unit_xy: [i16; 2]) -> bool {
    let half_extent = if target.runtime_class == 1 { 16 } else { 8 };
    (0..2).all(|axis| {
        (i32::from(unit_xy[axis]) - i32::from(bullet_xy[axis]) + 8).abs() <= half_extent
    })
}

/// 0x462b68..0x462c17: first stage of bullet contact attempts to hit crew.
/// Return None when crew is ineligible, so the caller need not consume RNG.
/// Class2 fixed guns halve chance. Vehicle families32(APC),36(Crane),38(Truck)
/// never take this crew route. Jeep34 is always eligible; most tanks require
/// hardware+0x19==3. Families27/24 additionally require opposite facing directions and
/// halve aim. Exact meaning/name of family27 remains unresolved.
pub fn crew_roll_threshold(target: &Target, attacker_aim: u8, attacker_facing: u8) -> Option<u8> {
    let mut shift = 0;
    if target.runtime_class == 2 {
        shift = 1;
    } else {
        if [32, 36, 38].contains(&target.animation_family) || target.runtime_class != 1 {
            return None;
        }
        if ![34, 27].contains(&target.animation_family) && target.secondary_state != 3 {
            return None;
        }
    }
    let mut aim = attacker_aim;
    if [27, 24].contains(&target.animation_family) {
        aim >>= 1;
        if attacker_facing != target.facing.wrapping_sub(4) & 7 {
            return Some(0);
        }
    }
    Some(aim >> shift)
}

/// 0x462c3f..0x462c75: bullet armor. Normal robots/classes0,3,15,16 use full
/// damage. Other classes subtract5; a negative signed byte becomes0 or1 with
/// equal RNG bit0x200. Damage5 yields0, without consuming an RNG value.
pub fn bullet_body_damage(runtime_class: u8, damage: u8, random_word: impl FnOnce() -> u32) -> u8 {
    if [0, 3, 15, 16].contains(&runtime_class) {
        return damage;
    }
    let reduced = damage.wrapping_sub(5);
    if (reduced as i8) >= 0 {
        reduced
    } else {
        u8::from(random_word() & 0x200 != 0)
    }
}

/// Difficulty settings for 0x41e7b0 direct crew damage.
pub fn crew_damage_plan(
    damage: u8,
    human: bool,
    difficulty: Difficulty,
    multiplayer: bool,
) -> (u8, u8) {
    if multiplayer {
        return (damage, 1);
    }
    match (difficulty, human) {
        (Difficulty::Easy, true) => ((damage >> 1).max(1), 1),
        (Difficulty::Easy, false) => (damage, 2),
        (Difficulty::Hard, false) => ((damage - (damage >> 2)).max(1), 1),
        _ => (damage, 1),
    }
}

/// Difficulty settings for 0x41f260 generic hull/body damage. The assembler is
/// essential: the initial decompiler omitted the easy-human damage>>1 argument.
pub fn body_damage_plan(
    damage: u8,
    human: bool,
    difficulty: Difficulty,
    multiplayer: bool,
) -> (u8, u8) {
    if multiplayer {
        return (damage, 1);
    }
    match (difficulty, human) {
        (Difficulty::Easy, true) => ((damage >> 1).max(1), 1),
        (Difficulty::Easy, false) | (Difficulty::Hard, true) => (damage, 2),
        _ => (damage, 1),
    }
}

/// 0x41e7b0: direct crew route bypasses hull and the bullet armor deduction.
/// Signed-byte wrapping reproduces original storage, including its edge cases.
pub fn apply_crew_damage(
    target: &mut Target,
    damage: u8,
    difficulty: Difficulty,
    multiplayer: bool,
) -> DamageOutcome {
    let (damage, calls) = crew_damage_plan(damage, target.human_team_zero, difficulty, multiplayer);
    if !target.immune {
        for _ in 0..calls {
            target.crew_hp = target.crew_hp.wrapping_sub(damage as i8);
        }
        if target.crew_hp <= 0 {
            target.crew_hp = -1;
        }
    }
    DamageOutcome {
        route: DamageRoute::Crew,
        damage_per_call: damage,
        calls,
        crew_killed: !target.immune && target.crew_hp < 0,
        hull_destroyed: false,
    }
}

/// 0x41e860: factor selected when a hit crosses an original hull threshold.
/// This factor scales current movement, aim and selected fire-rate attributes.
/// A hit crossing several thresholds applies ONE factor selected by final HP.
pub fn hull_degrade_factor(hull_hp: i16, max_hp: i16, damage: u8) -> Option<u8> {
    if damage == 0 {
        return None;
    }
    let after = hull_hp.wrapping_sub(i16::from(damage));
    let half = max_hp >> 1;
    let quarter = max_hp >> 2;
    let (threshold, factor) = if after <= quarter {
        (quarter, 80)
    } else if after <= half {
        (half, 85)
    } else if after <= half + quarter {
        (half + quarter, 90)
    } else {
        return None;
    };
    (hull_hp > threshold).then_some(factor)
}

/// 0x41f150 kernel; caller has already selected the damage amount. Special
/// truck fire-event handling (global0x4a6e73 and family38) belongs to the event
/// system and is not modeled here. Immunity uses the actual unit flag above.
fn apply_body_damage_one(target: &mut Target, damage: u8) {
    if target.immune {
        return;
    }
    if let Some(ref mut hull) = target.hull_hp {
        *hull = hull.wrapping_sub(i16::from(damage));
        if *hull >= 0 {
            return;
        }
        target.crew_hp = 0;
    }
    target.crew_hp = target.crew_hp.wrapping_sub(damage as i8);
    if target.crew_hp < 0 {
        target.crew_hp = -1;
    }
}

/// Generic direct impact/explosion damage after geometry has selected a unit.
/// Bullet callers must first apply bullet_body_damage, except on crew route.
pub fn apply_body_damage(
    target: &mut Target,
    damage: u8,
    difficulty: Difficulty,
    multiplayer: bool,
) -> DamageOutcome {
    let (damage, calls) = body_damage_plan(damage, target.human_team_zero, difficulty, multiplayer);
    for _ in 0..calls {
        apply_body_damage_one(target, damage);
    }
    DamageOutcome {
        route: DamageRoute::Body,
        damage_per_call: damage,
        calls,
        crew_killed: !target.immune && target.crew_hp < 0,
        hull_destroyed: !target.immune && target.hull_hp.is_some_and(|hp| hp < 0),
    }
}

/// Exact bullet contact resolution AFTER collision with an active enemy unit.
/// Damage originates from bullet+0xa, copied from unit+0x65/equipment+32.
/// aim roll during acquisition and projectile scattering happen separately.
pub fn resolve_bullet_contact(
    target: &mut Target,
    attacker_aim: u8,
    attacker_facing: u8,
    damage: u8,
    difficulty: Difficulty,
    multiplayer: bool,
    mut next_random: impl FnMut() -> u32,
) -> DamageOutcome {
    if let Some(threshold) = crew_roll_threshold(target, attacker_aim, attacker_facing) {
        if next_random() & 127 < u32::from(threshold) {
            return apply_crew_damage(target, damage, difficulty, multiplayer);
        }
    }
    let damage = bullet_body_damage(target.runtime_class, damage, || next_random());
    apply_body_damage(target, damage, difficulty, multiplayer)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn robot() -> Target {
        Target {
            runtime_class: 0,
            animation_family: 4,
            facing: 0,
            secondary_state: -1,
            human_team_zero: false,
            immune: false,
            crew_hp: 30,
            hull_hp: None,
        }
    }
    #[test]
    fn acquisition_is_inclusive_and_normal_boosts() {
        assert!(aim_roll_hits(60, false, 75));
        assert!(!aim_roll_hits(60, false, 76));
        assert!(aim_roll_hits(60, true, 60));
        assert!(!aim_roll_hits(60, true, 61));
        assert!(aim_roll_hits(120, false, 127));
    }
    #[test]
    fn bullet_armor_boundary_and_rng() {
        assert_eq!(bullet_body_damage(1, 5, || panic!("no RNG needed")), 0);
        assert_eq!(bullet_body_damage(1, 10, || panic!("no RNG needed")), 5);
        assert_eq!(bullet_body_damage(1, 2, || 0), 0);
        assert_eq!(bullet_body_damage(1, 2, || 0x200), 1);
        assert_eq!(bullet_body_damage(0, 5, || panic!("no RNG needed")), 5);
    }
    #[test]
    fn crew_and_body_difficulty_differ() {
        assert_eq!(crew_damage_plan(10, false, Difficulty::Hard, false), (8, 1));
        assert_eq!(
            body_damage_plan(10, false, Difficulty::Hard, false),
            (10, 1)
        );
        assert_eq!(body_damage_plan(10, true, Difficulty::Hard, false), (10, 2));
        assert_eq!(body_damage_plan(5, true, Difficulty::Easy, false), (2, 1));
    }
    #[test]
    fn exact_zero_survives_generic_body_but_not_crew() {
        let mut a = robot();
        a.crew_hp = 10;
        assert!(!apply_body_damage(&mut a, 10, Difficulty::Normal, false).crew_killed);
        assert_eq!(a.crew_hp, 0);
        let mut b = robot();
        b.crew_hp = 10;
        assert!(apply_crew_damage(&mut b, 10, Difficulty::Normal, false).crew_killed);
        assert_eq!(b.crew_hp, -1);
    }
    #[test]
    fn sniper_crew_hit_preserves_hull() {
        let mut a = robot();
        a.runtime_class = 1;
        a.animation_family = 34;
        a.hull_hp = Some(50);
        let outcome = resolve_bullet_contact(&mut a, 120, 0, 10, Difficulty::Normal, false, || 119);
        assert_eq!(outcome.route, DamageRoute::Crew);
        assert_eq!(a.crew_hp, 20);
        assert_eq!(a.hull_hp, Some(50));
        let outcome = resolve_bullet_contact(&mut a, 120, 0, 10, Difficulty::Normal, false, || 120);
        assert_eq!(outcome.route, DamageRoute::Body);
        assert_eq!(a.hull_hp, Some(45));
    }
    #[test]
    fn closed_hatch_blocks_crew_route() {
        let mut a = robot();
        a.runtime_class = 1;
        a.animation_family = 6;
        a.hull_hp = Some(200);
        assert_eq!(crew_roll_threshold(&a, 120, 0), None);
        a.secondary_state = 3;
        assert_eq!(crew_roll_threshold(&a, 120, 0), Some(120));
        a.animation_family = 32;
        assert_eq!(crew_roll_threshold(&a, 120, 0), None);
    }
    #[test]
    fn hull_exact_zero_and_degradation_boundaries() {
        let mut a = robot();
        a.hull_hp = Some(10);
        assert!(!apply_body_damage(&mut a, 10, Difficulty::Normal, false).hull_destroyed);
        assert_eq!(a.hull_hp, Some(0));
        assert!(apply_body_damage(&mut a, 1, Difficulty::Normal, false).hull_destroyed);
        assert_eq!(hull_degrade_factor(200, 200, 50), Some(90));
        assert_eq!(hull_degrade_factor(150, 200, 1), None);
        assert_eq!(hull_degrade_factor(200, 200, 151), Some(80));
    }
    fn engine_game() -> super::super::Game {
        let mut g = super::super::Game::new(0, 1, 7291);
        g.map.original = true;
        g.map.tiles.fill(0);
        g.units.clear();
        g.buildings.clear();
        g.sectors.clear();
        g.crates.clear();
        g.effects.clear();
        g.ai_teams.clear();
        g
    }
    #[test]
    fn original_engine_applies_armor_without_changing_procedural_damage() {
        let mut g = engine_game();
        g.spawn("grunt", 0, 640., 640., 1, None);
        g.spawn("medium", 1, 740., 640., 2, None);
        g.units[1].cooldown = 100.;
        g.step(0.05);
        assert_eq!(g.units[1].hp, 200.);
        assert_eq!(g.units[1].crew.as_ref().unwrap().hp, 30.);
        g.map.original = false;
        g.units[0].cooldown = 0.;
        g.step(0.05);
        assert_eq!(g.units[1].hp, 195.);
    }
    #[test]
    fn original_engine_keeps_zero_hp_robot_active_and_commandable() {
        let mut g = engine_game();
        g.spawn("grunt", 0, 640., 640., 1, None);
        g.spawn("grunt", 1, 740., 640., 2, None);
        let id = g.units[1].id;
        g.units[1].hp = 5.;
        g.units[1].cooldown = 100.;
        g.step(0.05);
        assert_eq!(g.units.iter().find(|u| u.id == id).unwrap().hp, 0.);
        assert_eq!(g.winner, None);
        g.command(&serde_json::json!({"team":1,"ids":[id],"kind":"move","x":780,"y":680}));
        assert_eq!(g.units.iter().find(|u| u.id == id).unwrap().order, "move");
    }
    #[test]
    fn crew_kill_adapter_leaves_neutral_usable_hull() {
        let mut g = engine_game();
        g.spawn("jeep", 1, 640., 640., 1, None);
        let unit = &mut g.units[0];
        unit.crew.as_mut().unwrap().hp = 10.;
        unit.path.push_back((700., 640.));
        unit.order = "attack".into();
        let attack = Attack {
            target: unit.id,
            damage: 10,
            aim: 120,
            facing: 2,
            bullet: true,
        };
        assert!(apply_attack(unit, attack, 1, || 0));
        assert_eq!(unit.hp, 50.);
        assert_eq!(unit.team, -1);
        assert!(unit.crew.is_none());
        assert!(unit.path.is_empty());
        assert_eq!(unit.order, "guard");
    }
    #[test]
    fn original_hull_difficulty_and_degradation_use_each_damage_call() {
        let mut g = engine_game();
        g.spawn("medium", 0, 640., 640., 1, None);
        g.spawn("medium", 1, 740., 640., 2, None);
        let attack = Attack {
            target: 0,
            damage: 80,
            aim: 80,
            facing: 2,
            bullet: false,
        };
        apply_attack(&mut g.units[0], attack, 2, || 0);
        apply_attack(&mut g.units[1], attack, 2, || 0);
        assert_eq!(g.units[0].hp, 40.);
        assert_eq!(g.units[1].hp, 120.);
        let state = g.units[0].original_combat.as_ref().unwrap();
        assert_eq!(state.aim_skill, 57);
        assert_eq!(state.movement_control, 28);
        assert_eq!(state.fire_rate, 50);
    }
    #[test]
    fn malformed_original_crates_are_reported() {
        let mut g = engine_game();
        let payload = serde_json::json!({"map":g.map,"sectors":[],"buildings":[],"spawns":[],
            "crates":[{"x":10,"y":10,"taken":"invalid"}]});
        assert!(g
            .apply_original(&payload)
            .unwrap_err()
            .contains("Invalid original crates"));
    }
}
