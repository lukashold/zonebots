//! Particle portions of the two fort bursts. They share the existing
//! effect slots, raw constructor globals and native RNG with bridge effects.

use super::{bridge::{self, Globals, NATIVE_POOL}, spray::Pool};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Burst { First, Second }

const FIRST_CONSTRUCTOR: u32 = 0x4032e0;
const SECOND_CONSTRUCTOR: u32 = 0x4020f0;

/// Particle request after the same-actor suppressed removal returns.
/// The source sets the constructor globals even if no pool slot is usable.
pub(crate) fn release_poof(world:&mut impl bridge::World,globals:&mut Globals,point:[i16;2],flags64:u8)->usize {
    globals.context.origin=[point[0].wrapping_sub(152),point[1].wrapping_sub(84)];
    globals.context.owner=flags64&3;
    globals.context.fade=0;
    globals.context.flags=12;
    globals.context.sprite=1903;
    globals.maximum=7;
    globals.raw.constructor=Some(bridge::HIT_CONSTRUCTOR);
    bridge::emit(world,globals,1,bridge::Constructor::Hit)
}

/// 420890 precedes the jeep's insertion test. 458930 overrides effect
/// suppression only for the spray requested by a rejected footprint.
pub(crate) fn retirement_contact(world:&mut impl bridge::World,globals:&mut Globals,point:[i16;2],flags64:u8,jeep:bool)->usize {
    globals.context.owner=flags64&3;
    globals.context.flags=12;
    let suppression=globals.suppression;
    let (kind,count)=if jeep{
        globals.context.origin=[point[0].wrapping_sub(152),point[1].wrapping_sub(84)];
        globals.context.fade=1;globals.context.sprite=58;globals.maximum=7;
        globals.raw.constructor=Some(0x403200);
        (bridge::Constructor::JeepWreck,1)
    }else{
        let aligned=point.map(|v|v.wrapping_add(7)&!15);
        globals.context.origin=[aligned[0].wrapping_sub(154),aligned[1].wrapping_sub(80)];
        globals.context.fade=0;globals.context.sprite=557;globals.maximum=5;
        globals.raw.constructor=Some(0x4017b0);
        globals.suppression=0;
        (bridge::Constructor::BlockedWreck,64)
    };
    let made=bridge::emit(world,globals,count,kind);
    globals.suppression=suppression;
    made
}

fn emit(world: &mut impl Pool, globals: &mut Globals, burst: Burst) -> usize {
    globals.raw.constructor = Some(match burst { Burst::First => FIRST_CONSTRUCTOR, Burst::Second => SECOND_CONSTRUCTOR });
    if globals.suppression != 0 { return 0; }
    let Some(slot) = world.first_free() else { return 0; };
    let mut raw = world.load(slot);
    bridge::common(&mut raw, globals);
    globals.raw.last_record = Some(NATIVE_POOL + slot as u32 * 44);
    raw[23] = 0;
    for field in [10, 11] {
        raw[field] = (world.random() & 31).wrapping_sub(16) as u8;
    }
    raw[12] = (-4i8 - (world.random() & 3) as i8) as u8;
    let spin = (world.random() & 63) as i16 - 32;
    raw[30..32].copy_from_slice(&(if spin == 0 { 6 } else { spin }).to_le_bytes());
    raw[13..15].fill(4);
    raw[18..22].copy_from_slice(&0x30190u32.to_le_bytes());
    if burst == Burst::First {
        raw[12] = raw[12].wrapping_add(1);
        raw[18..22].copy_from_slice(&28u32.to_le_bytes());
        raw[23..25].copy_from_slice(&[0, 7]);
        raw[16..18].fill(4);
    }
    world.store(slot, raw);
    1
}

pub(crate) fn spawn(world: &mut impl Pool, globals: &mut Globals, burst: Burst, base: [i16; 2], fort_owner: u8) -> usize {
    let [x, y] = base;
    globals.context.fade = 0;
    globals.context.flags = 12;
    globals.context.owner = 0;
    let mut made = 0;
    match burst {
        Burst::First => {
            globals.context.sprite = 473;
            globals.maximum = 7;
            for _ in 0..6 {
                let a = (world.random() & 63) as i16;
                let b = (world.random() & 31) as i16;
                let c = (world.random() & 63) as i16;
                globals.context.origin = [x.wrapping_sub(128).wrapping_add(b).wrapping_add(a), y.wrapping_sub(76).wrapping_add(c)];
                made += emit(world, globals, burst);
            }
        }
        Burst::Second => {
            globals.maximum = 11;
            for _ in 0..12 {
                let a = (world.random() & 3) as usize;
                let b = (world.random() & 1) as usize;
                globals.context.sprite = [1692, 1704, 1716, 1728, 1740][a + b];
                let dy = (world.random() & 127) as i16;
                let dx = (world.random() & 127) as i16;
                globals.context.origin = [x.wrapping_sub(160).wrapping_add(dx), y.wrapping_sub(92).wrapping_add(dy)];
                made += emit(world, globals, burst);
                globals.raw.payload36 = globals.raw.payload36.wrapping_add(10);
            }
            globals.maximum = 0;
            globals.context.sprite = if fort_owner == 1 { 1234 } else { 1238 };
            globals.context.origin = [x.wrapping_sub(83), y.wrapping_sub(59)];
            made += emit(world, globals, burst);
        }
    }
    made
}
