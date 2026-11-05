//! Pure 447c20/4476f0/447970 caller projection. No Game activation or storage.
//!
//! Raw slot and field readers are supplied by their existing owners. In
//! particular B6 is indexed by physical UNIT slot, never by passenger position.
//! 447790 and direct manual emission remain named ordered leaf operations;
//! replaying their results is not a proof of a new living emitter adapter.
use crate::{native_weapon_data, original_combat::OriginalRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    Actor(u8),
    Target(u8),
    Member(u8),
    Group(u8),
    Missing(&'static str),
    MuzzleFacing(u8),
    HardwareToken(u32),
}
pub type Result<T> = std::result::Result<T, Boundary>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub slot: u8,
    pub position: [i16; 2],
    pub family30: u8,
    pub class7c: u8,
    pub group5f: u8,
    pub facing39: u8,
    pub saved_class8e: u8,
    pub rate78: u8,
    pub reload7b: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Front,
    Right,
    Back,
    Left,
}
impl Side {
    pub fn from_port(port: u8) -> Self {
        match port & 3 {
            0 => Self::Front,
            1 => Self::Right,
            2 => Self::Back,
            _ => Self::Left,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shot {
    pub port: u8,
    pub origin: [i16; 2],
    pub target_slot: u8,
    pub physical_member: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectShot {
    pub port: u8,
    pub actor_position: [i16; 2],
    pub facing39: u8,
    pub saved_class8e: u8,
    pub physical_member: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Range { point: [i16; 2], outside: bool },
    Gate(bool),
    Attempt { port: u8, side: Side },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Return {
    pub attempts: u8,
    pub invoked_targets: u16,
}

/// Exact UNIT8 domain, not a new W pool. Invalid non-NIL pointers do not
/// become the dummy. The adapter supplies the existing composite W owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareRef {
    Slot(u8),
    Dummy,
}
pub fn hardware_ref(token: u32) -> Result<HardwareRef> {
    if token == 0 || token == u32::MAX {
        return Ok(HardwareRef::Dummy);
    }
    let Some(offset) = token.checked_sub(0x59e6c2) else {
        return Err(Boundary::HardwareToken(token));
    };
    if offset % 44 != 0 || offset / 44 >= 240 {
        return Err(Boundary::HardwareToken(token));
    }
    Ok(HardwareRef::Slot((offset / 44) as u8))
}
/// These ordered operations are leaves, not replacement implementations of
/// audio, scatter, collision warning, finite allocation or actual emission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    Wrapper27,
    Direct43WithCycle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emission {
    Heavy {
        origin: [i16; 2],
        target: [i16; 2],
        speed: u8,
        sample: u8,
    },
    Flame {
        origin: [i16; 2],
        target: [i16; 2],
        speed: u8,
    },
    Bullet {
        origin: [i16; 2],
        target: [i16; 2],
        speed: u8,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View {
    pub camera: [i16; 2],
    pub width: i16,
    pub height: i16,
}
impl View {
    pub fn contains(self, point: [i16; 2]) -> bool {
        let x = i32::from(point[0].wrapping_sub(self.camera[0]));
        let y = i32::from(point[1].wrapping_sub(self.camera[1]));
        x > -49 && x <= i32::from(self.width) + 32 && y > -49 && y <= i32::from(self.height) + 32
    }
}
pub trait LeafWorld {
    fn actor(&self) -> Actor;
    fn view(&mut self) -> Result<View>;
    fn master_enabled(&mut self) -> Result<bool>;
    fn sound_once(&mut self) -> Result<u8>;
    fn write_sound_once(&mut self, value: u8);
    fn sound(&mut self, operation: Sound) -> Result<()>;
    fn saved_facing(&mut self) -> Result<u8>;
    fn write_saved_facing(&mut self, value: u8);
    fn write_facing(&mut self, value: u8);
    fn field46(&mut self) -> Result<u16>;
    fn write_field46(&mut self, value: u16);
    fn scatter448c20(&mut self, target_slot: u8, rng: &mut OriginalRng) -> Result<()>;
    fn target42(&mut self) -> Result<i16>;
    fn target44(&mut self) -> Result<i16>;
    fn speed79(&mut self) -> Result<u8>;
    fn payload65(&mut self) -> Result<u8>;
    fn hardware_token8(&mut self) -> Result<u32>;
    fn read_w(&mut self, owner: HardwareRef, offset: u8) -> Result<u8>;
    fn write_w(&mut self, owner: HardwareRef, offset: u8, value: u8);
    /// Must execute existing warning/audio/allocation in source order. A full
    /// pool is an ordinary return, not Err and not a second allocation attempt.
    fn emit(&mut self, operation: Emission, rng: &mut OriginalRng) -> Result<()>;
    fn write_laser622be6(&mut self, value: u8);
    fn clear_bullet_output64d7b0(&mut self);
    fn bullet_output64d7b0(&mut self) -> Result<Option<u32>>;
    fn increment_bullet_range11(&mut self, token: u32) -> Result<()>;
}

/// 447790 control around named ordered leaves. No leaf is run again after an
/// Err. Shared saved-facing is read afresh at restore, not a shadow local.
pub fn target_leaf(w: &mut impl LeafWorld, shot: Shot, rng: &mut OriginalRng) -> Result<()> {
    w.write_saved_facing(w.actor().facing39);
    if w.view()?.contains(shot.origin) {
        let class = w.actor().saved_class8e;
        if class != 3 && class != 15 {
            if class == 16 {
                if w.master_enabled()? {
                    w.sound(Sound::Direct43WithCycle)?;
                }
            } else {
                if w.sound_once()? == 0 {
                    w.sound(Sound::Wrapper27)?;
                }
                w.write_sound_once(1);
            }
        }
    }
    let saved46 = w.field46()?;
    w.scatter448c20(shot.target_slot, rng)?;
    let x = w.target42()?.wrapping_add(8);
    let token = w.hardware_token8()?;
    w.write_field46(saved46);
    let target = [x, w.target44()?.wrapping_add(8)];
    let speed = w.speed79()?;
    let hw = hardware_ref(token)?;
    let payload = w.payload65()?;
    w.write_w(hw, 6, payload);
    let range = w.read_w(hw, 39)?;
    w.write_w(hw, 5, range);
    match w.actor().saved_class8e {
        3 => w.emit(
            Emission::Heavy {
                origin: shot.origin,
                target,
                speed,
                sample: 29,
            },
            rng,
        )?,
        15 => {
            let facing = w.saved_facing()?;
            if facing >= 8 {
                return Err(Boundary::MuzzleFacing(facing));
            }
            let offset = native_weapon_data::APC_FLAME[usize::from(facing)];
            let origin = [
                shot.origin[0].wrapping_add(offset[0] as i16),
                shot.origin[1].wrapping_add(offset[1] as i16),
            ];
            w.emit(
                Emission::Flame {
                    origin,
                    target,
                    speed,
                },
                rng,
            )?;
        }
        class => {
            let mut origin = shot.origin;
            if class == 16 {
                w.write_laser622be6(1);
                origin[0] = origin[0].wrapping_sub(7);
                origin[1] = origin[1].wrapping_sub(7);
            }
            w.clear_bullet_output64d7b0();
            w.emit(
                Emission::Bullet {
                    origin,
                    target,
                    speed,
                },
                rng,
            )?;
            w.write_laser622be6(0);
            if let Some(token) = w.bullet_output64d7b0()? {
                w.increment_bullet_range11(token)?;
            }
        }
    }
    let facing = w.saved_facing()?;
    w.write_facing(facing);
    Ok(())
}

/// The implementation must use the one existing trail.error byte. None of
/// these methods permits a fresh header or a fabricated inactive slot.
pub trait GateWorld {
    fn multiplayer(&self) -> bool;
    fn strategy(&mut self, group: u8) -> Result<(u8, u8)>;
    fn sector_bounds(&mut self, index: usize) -> Result<[i16; 4]>;
    fn sector_flags24(&mut self, index: usize) -> Result<u8>;
    fn current_group3d(&mut self) -> Result<u8>;
    fn trail_error(&self) -> u8;
    fn write_trail_error(&mut self, value: u8);
}

/// 17400: two ordered 64-record passes. Unknown flags of an unmatched sector
/// are never read. Writes preceding an unavailable record remain committed.
pub fn locate_sector(w: &mut impl GateWorld, position: [i16; 2]) -> Result<Option<usize>> {
    let cell = [position[0] >> 4, position[1] >> 4];
    let mut error = w.trail_error();
    for extra in [0i16, 1] {
        let x = cell[0].wrapping_add(extra);
        let y = cell[1].wrapping_add(extra);
        for i in 0..64 {
            let [left, top, right, bottom] = w.sector_bounds(i)?;
            if left == -1 {
                continue;
            }
            if left > 1000 {
                error = 2;
            }
            w.write_trail_error(error);
            if left <= x && x < right && top <= y && y < bottom {
                return Ok(Some(i));
            }
        }
    }
    Ok(None)
}

/// 416160 short-circuits multiplayer before any strategy/sector/group read.
pub fn acquisition_blocked(w: &mut impl GateWorld, a: Actor) -> Result<bool> {
    if w.multiplayer() {
        return Ok(false);
    }
    let (flags8, mode5) = w.strategy(a.group5f)?;
    if mode5 == 0 || flags8 & 8 == 0 {
        return Ok(false);
    }
    if let Some(index) = locate_sector(w, a.position)? {
        if w.sector_flags24(index)? & 2 != 0 {
            return Ok(w.current_group3d()? != 0);
        }
    }
    Ok(false)
}

/// Explicit ordered ports, with no persistent owner in this module.
pub trait World: GateWorld + ActorTeam {
    fn actor(&self) -> Actor;
    fn write_reload7b(&mut self, value: u8);
    fn side_target(&self, side: Side) -> u8;
    fn write_side_target(&mut self, side: Side, value: u8);
    /// Actual 58500 scanner; updates the same raw71/77 owner before returning.
    fn scan58500(&mut self, relative: i8) -> Result<u16>;
    fn target_position(&mut self, slot: u8) -> Result<[i16; 2]>;
    fn target_flags31(&mut self, slot: u8) -> Result<u8>;
    /// W5 from canonical UNIT8 resolution; only NIL/FFFF selects shared dummy.
    fn weapon_range5(&mut self) -> Result<u8>;
    fn write_range_point(&mut self, point: [i16; 2]);
    /// One live byte from the group pointer captured before roster iteration.
    /// A leaf may change that storage; no roster snapshot is taken here.
    fn roster_member21(&mut self, group: u8, index: usize) -> Result<u8>;
    fn read_b6(&mut self, slot: u8) -> Result<u8>;
    fn write_b6(&mut self, slot: u8, value: u8);
    fn manual_mode(&mut self) -> Result<u8>;
    /// Real 447790 ordered leaf, including its W/dummy/scatter/RNG/finite-pool
    /// ports. The caller increments its local success WORD after any return,
    /// even when the leaf allocated nothing. Err retains preceding writes.
    fn target_leaf447790(&mut self, shot: Shot, rng: &mut OriginalRng) -> Result<()>;
    /// 447970 manual3/15 branch, explicitly distinct from 447790. It does not
    /// increment the success WORD; its emitter remains an ordered leaf.
    fn direct_manual_leaf(&mut self, shot: DirectShot, rng: &mut OriginalRng) -> Result<()>;
    fn observe(&mut self, _event: Event) {}
}

/// 45afb0 after its two global WORD writes. All arithmetic below is signed
/// DWORD with wrapping IMUL/ADD, followed by the original signed comparison.
pub fn outside_range(a: Actor, point: [i16; 2], range: u8) -> bool {
    let dx = i32::from(a.position[0])
        .wrapping_sub(i32::from(point[0]))
        .wrapping_add(12);
    let dy = i32::from(a.position[1])
        .wrapping_sub(i32::from(point[1]))
        .wrapping_add(8);
    let distance = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
    let hardware = matches!(a.class7c, 1 | 2);
    if hardware && a.family30 == 32 && range == 0 {
        return distance > 64;
    }
    let r = i32::from(range) + i32::from(hardware && a.family30 != 32);
    let pixels = r * 8;
    pixels * pixels < distance
}

/// 4476f0 stores a cached scanner winner before reading its live raw header.
pub fn acquire(w: &mut impl World, side: Side, relative: i8) -> Result<()> {
    w.write_side_target(side, 255);
    let packed = w.scan58500(relative)?;
    let slot = (packed >> 8) as u8;
    if slot != 255 {
        w.write_side_target(side, slot);
        if slot >= 240 {
            return Err(Boundary::Target(slot));
        }
        let position = w.target_position(slot)?;
        let point = [position[0].wrapping_add(12), position[1].wrapping_add(8)];
        w.write_range_point(point);
        let a = w.actor();
        let outside = outside_range(a, point, w.weapon_range5()?);
        w.observe(Event::Range { point, outside });
        if !outside {
            let flags31 = w.target_flags31(slot)?;
            if flags31 & 4 == 0 {
                w.write_side_target(side, 255);
                return Ok(());
            }
            // actor.flags31 low team is an explicit scanner/header input,
            // not family/class. The scanner port supplies it independently.
            if w.actor_team31()? & 3 != flags31 & 3 {
                let blocked = acquisition_blocked(w, a)?;
                w.observe(Event::Gate(blocked));
                if !blocked {
                    return Ok(());
                }
            }
        }
    }
    w.write_side_target(side, 255);
    Ok(())
}

// Kept as a separate reader to avoid confusing class/team or reconstructing
// flags31 from a renderer identity. Implementors must supply the actual byte.
pub trait ActorTeam {
    fn actor_team31(&mut self) -> Result<u8>;
}

fn muzzle(a: Actor, port: u8) -> Result<[i16; 2]> {
    if a.facing39 >= 8 {
        return Err(Boundary::MuzzleFacing(a.facing39));
    }
    let table = match Side::from_port(port) {
        Side::Front => native_weapon_data::APC_FRONT,
        Side::Right => native_weapon_data::APC_RIGHT,
        Side::Back => native_weapon_data::APC_BACK,
        Side::Left => native_weapon_data::APC_LEFT,
    };
    let offset = table[usize::from(a.facing39) * 3 + usize::from(port / 4)];
    Ok([
        a.position[0].wrapping_add(offset[0] as i16).wrapping_sub(6),
        a.position[1].wrapping_add(offset[1] as i16).wrapping_sub(6),
    ])
}

/// Pure caller only. There is no fallback after a partial call, and no Game,
/// passenger-position phase mirror, world RNG clone or second finite pool.
pub fn run(w: &mut impl World, rng: &mut OriginalRng) -> Result<Return> {
    let initial = w.actor();
    if initial.slot >= 240 {
        return Err(Boundary::Actor(initial.slot));
    }
    if initial.reload7b != 0 {
        return Ok(Return::default());
    }
    for (side, relative) in [
        (Side::Right, 2),
        (Side::Left, -2),
        (Side::Front, 0),
        (Side::Back, 4),
    ] {
        acquire(w, side, relative)?;
    }
    if [Side::Front, Side::Right, Side::Back, Side::Left]
        .iter()
        .all(|&s| w.side_target(s) == 255)
    {
        return Ok(Return::default());
    }
    let a = w.actor();
    w.write_reload7b(150u8.wrapping_sub(a.rate78 >> 1).wrapping_sub(a.rate78));
    if a.group5f >= 240 {
        return Err(Boundary::Group(a.group5f));
    }
    let mut result = Return::default();
    for index in 0..21 {
        let member = w.roster_member21(a.group5f, index)?;
        if member == 255 {
            continue;
        }
        if member >= 240 {
            return Err(Boundary::Member(member));
        }
        let phase = w.read_b6(member)?.wrapping_add(1) & 15;
        w.write_b6(member, phase);
        if w.actor().saved_class8e == 15 && phase >= 10 {
            continue;
        }
        loop {
            let side = Side::from_port(result.attempts);
            w.observe(Event::Attempt {
                port: result.attempts,
                side,
            });
            let manual = w.manual_mode()? != 0;
            let target_slot = w.side_target(side);
            let mut invoked = false;
            if manual || target_slot != 255 {
                let a = w.actor();
                let shot = Shot {
                    port: result.attempts,
                    origin: muzzle(a, result.attempts)?,
                    target_slot,
                    physical_member: member,
                };
                if manual && matches!(a.saved_class8e, 3 | 15) {
                    w.direct_manual_leaf(
                        DirectShot {
                            port: shot.port,
                            actor_position: a.position,
                            facing39: a.facing39,
                            saved_class8e: a.saved_class8e,
                            physical_member: member,
                        },
                        rng,
                    )?;
                } else {
                    w.target_leaf447790(shot, rng)?;
                    result.invoked_targets = result.invoked_targets.wrapping_add(1);
                    invoked = true;
                }
            }
            result.attempts += 1;
            if result.attempts == 12 {
                return Ok(result);
            }
            if invoked {
                break;
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "original_apc_automatic_tests.rs"]
mod tests;
