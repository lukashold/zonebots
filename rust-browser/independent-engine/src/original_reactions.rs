//! Incoming-projectile cover reactions, recovered from the local executable.
//! This is a short infantry pose, not a lateral displacement. The caller must
//! supply native states and group/acquisition fields; renderer/path adapters
//! must not silently substitute arbitrary browser orders for those fields.

/// State43's 00496e78 graphics row: sprite=2075+frame, direction stride0.
pub const COVER_SPRITE_BASE: u16 = 2075;

///0046400b..0046401a. The original prefix uses X for both terms; replacing
///the first withY would change the candidate set.00462930 sign-extends it.
pub fn projectile_bucket(native_x: u16) -> i8 {
    (((native_x >> 4) as u8 & 0xf8).wrapping_add((native_x >> 7) as u8)) as i8
}

pub const PROJECTILE_BUCKET_OFFSETS: [i16; 9] = [0, -1, 1, -17, -16, -15, 15, 16, 17];

///0044cd60 adds in16 bits and filters the unsigned index before reading a
///bucket. Per-bucket rosters and the total288-candidate cutoff remain external.
pub fn candidate_bucket(origin: i8, offset: i16) -> Option<usize> {
    let index = i16::from(origin).wrapping_add(offset) as u16;
    (index < 256).then_some(usize::from(index))
}

/// 00462930, called before each heavy projectile advances. Candidates already
/// came from the native nine spatial buckets. It applies no faction filter.
pub fn projectile_nearby(class: u8, state: u8, projectile: [i16; 2], unit: [i16; 2]) -> bool {
    if matches!(class, 1 | 2) || !matches!(state, 0 | 1) {
        return false;
    }
    // Both subtractions wrap in AX before sign extension. IMUL/ADD are signed
    // 32-bit operations, including their unusual far-coordinate overflow.
    let dx = i32::from(projectile[0].wrapping_sub(unit[0]));
    let dy = i32::from(projectile[1].wrapping_sub(unit[1]));
    dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)) < 64
}

/// Raw fields consumed by 00458590. A leader has member=-1, an independent
/// robot member=0; both prevent cover. A follower uses its native group record.
#[derive(Clone, Copy, Debug, Default)]
pub struct GroupGate {
    pub member: i8,
    pub group: u8,
    pub flags09: u8,
    pub timer3d: u8,
    pub command3a: u8,
    pub control10: u8,
    pub count12: u8,
    pub acquisition_override: bool,
    pub range04: i8,
}

/// 00458590. `probe` is 00458500's low word. AH=FF means no candidate. The
/// caller must perform that search only after the early group gates pass.
pub fn group_prevents_cover(g: GroupGate, probe: u16) -> bool {
    if !group_needs_probe(g) {
        return true;
    }
    probe & 0xff00 != 0xff00 && i32::from((probe & 127) * 2) <= i32::from(g.range04)
}

pub fn group_needs_probe(g: GroupGate) -> bool {
    !matches!(g.member, -1 | 0)
        && g.group != 255
        && g.flags09 & 1 == 0
        && g.timer3d == 0
        && (g.acquisition_override || g.command3a == 0)
        && (g.control10 == 0 || g.count12 == 0)
}

/// 00459250 samples terrain at the unit's native anchor+(5,11). It excludes
/// these four water/shore codes, independently of the generic path table.
pub fn terrain_allows_cover(terrain: u8) -> bool {
    !matches!(terrain & 127, 9 | 38 | 39 | 42)
}

/// Fields written by 00459250; byte offsets refer to the native live unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Cover {
    pub state35: u8,
    pub frame32: i16,
    pub timer82: i16,
    pub animation89: u8,
    pub counter7e: i16,
    pub remaining80: u8,
}

pub fn begin_cover(facing39: u8, terrain: u8, group_prevents: bool) -> Option<Cover> {
    if !terrain_allows_cover(terrain) || group_prevents {
        return None;
    }
    let direction = i16::from(facing39 & 14);
    Some(Cover {
        state35: 43,
        frame32: direction + (direction >> 1),
        timer82: -1,
        animation89: 4,
        counter7e: 3,
        remaining80: 4,
    })
}

/// 00459110. Returns true when the caller must invoke the normal 0044cac0
/// reset/command-resumption path. No position changes occur in this callback.
pub fn tick_cover(cover: &mut Cover, facing39: u8, alternate_phase: u8, group_prevents: bool) -> bool {
    if alternate_phase != 0 {
        return false;
    }
    if !group_prevents {
        let direction = i16::from(facing39 & 14);
        let terminal = direction + (direction >> 1) + 2;
        if cover.frame32 != terminal {
            cover.frame32 = cover.frame32.wrapping_add(1);
            return false;
        }
        if cover.remaining80 != 0 {
            cover.remaining80 = cover.remaining80.wrapping_sub(1);
            return false;
        }
    }
    true
}

/// The infantry prefix of 0044cac0 consumes exactly two native RNG values.
/// Its later negative-command resumption remains the caller's responsibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdleReset {
    pub flags8a: u8,
    pub state35: u8,
    pub frame32: i16,
    pub countdown3d: u8,
    pub phase3c: u8,
    pub facing40: u8,
    pub phase41: u8,
    pub idle7d: u8,
}

pub fn cover_idle_reset(facing39: u8, flags8a: u8, random: [u32; 2]) -> IdleReset {
    IdleReset {
        flags8a: flags8a & !8,
        state35: 1,
        frame32: 1,
        countdown3d: 8,
        phase3c: (random[0] & 3) as u8,
        facing40: facing39,
        phase41: 0,
        idle7d: ((random[1] & 65535) % 53 + 12) as u8,
    }
}

///00462fd0 is separate from the short cover pose. It may issue a fresh movement
///line while a hostile shell is flying. The nine-bucket search is external.
#[derive(Clone, Copy, Debug)]
pub struct DodgeUnit {
    pub position: [i16; 2],
    pub flags31: u8,
    pub flags64: u8,
    pub flags69: u8,
    pub class7c: u8,
    pub state35: u8,
    pub aim02: i8,
}

pub fn dodge_candidate(unit: DodgeUnit, shooter_team: u8, projectile: [i16; 2], target: [i16; 2]) -> bool {
    if unit.flags31 & 3 == shooter_team & 3 || unit.class7c == 2
        || unit.flags64 & 64 != 0 || unit.flags31 & 64 != 0
        || matches!(unit.state35, 2 | 7 | 18) {
        return false;
    }
    for axis in 0..2 {
        let d = unit.position[axis].wrapping_sub(target[axis]).wrapping_add(8);
        let absolute = if d < 0 { d.wrapping_neg() } else { d };
        if absolute > 32 { return false; }
    }
    let mut radius = i16::from(unit.aim02 >> 2);
    if unit.flags69 & 128 == 0 {
        radius = radius.wrapping_add(i16::from(unit.aim02));
        if radius < 0 { radius = 127; }
        radius >>= 2;
    }
    let squared = radius.wrapping_mul(radius);
    let dx = i32::from(unit.position[0]) - i32::from(projectile[0]);
    let dy = i32::from(unit.position[1]) - i32::from(projectile[1]);
    squared > 0 && dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)) <= i32::from(squared)
}

///Both native RNG calls affect the result: X escapes by40 pixels, Y by48.
///The caller also sets unit.flags64|=0x40 and counter8c=2, invokes004525a0,
///then sets hardware.flags17|=1 for class1 with a valid hardware pointer.
pub fn dodge_destination(unit: [i16; 2], target: [i16; 2], random: [u32; 2]) -> [i16; 2] {
    std::array::from_fn(|axis| {
        let d = ((random[axis] & 15) as i16).wrapping_sub(target[axis]).wrapping_add(unit[axis]);
        unit[axis].wrapping_sub(d).wrapping_add(if d < 0 { -[40, 48][axis] } else { [40, 48][axis] })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(h: &mut u32, values: &[i32]) {
        for &n in values {
            *h = (*h ^ n as u32).wrapping_mul(16777619);
        }
    }

    #[test]
    fn proximity_matches_450_executed_native_cases() {
        let mut h = 2166136261;
        let mut scenario = 0;
        for class in [0, 1, 2, 3, 15, 16, 17, 18, 255] {
            for state in [0, 1, 2, 43, 255] {
                for [dx, dy] in [[0, 0], [7, 0], [8, 0], [5, 6], [-7, 0], [0, -8], [32767, 0], [-32768, 0], [32767, 32767], [-32768, -32768]] {
                    let point = [(500 + dx) as i16, (500 + dy) as i16];
                    let hit = projectile_nearby(class, state, point, [500, 500]);
                    hash(&mut h, &[scenario, i32::from(hit)]);
                    scenario += 1;
                }
            }
        }
        assert_eq!(h, 4174309496);
    }

    #[test]
    fn bucket_prefix_matches_15_executed_native_cases() {
        let mut h = 2166136261;
        for x in [0, 1, 15, 127, 128, 255, 256, 511, 512, 1023, 1024, 2047, 32767, 32768, 65535] {
            hash(&mut h, &[i32::from(x), i32::from(projectile_bucket(x))]);
        }
        assert_eq!(h, 2839521127);
        assert_eq!(candidate_bucket(-1, 1), Some(0));
        assert_eq!(candidate_bucket(-1, 0), None);
    }

    #[test]
    fn cover_entry_matches_4096_executed_native_cases() {
        let mut h = 2166136261;
        let mut scenario = 0;
        for code in 0..128 {
            for facing in 0..16 {
                for blocked in [false, true] {
                    let started = begin_cover(facing, code, blocked);
                    let result = i32::from(started.is_none());
                    let c = started.unwrap_or(Cover { state35: 1, frame32: -123, timer82: -456, animation89: 77, counter7e: -1234, remaining80: 63 });
                    hash(&mut h, &[scenario, result, i32::from(terrain_allows_cover(code)), i32::from(c.state35), i32::from(c.frame32), i32::from(c.timer82), i32::from(c.animation89), i32::from(c.counter7e), i32::from(c.remaining80)]);
                    scenario += 1;
                }
            }
        }
        assert_eq!(h, 4264668485);
    }

    #[test]
    fn group_gates_match_4096_executed_native_cases() {
        let mut h = 2166136261;
        let probes = [0xff00, 0, 1, 15, 127, 255, 0xff01, 0x0107];
        for scenario in 0..4096 {
            let g = GroupGate { member: [-1, 0, 1, 127][scenario as usize & 3], group: if scenario & 4 != 0 { 255 } else { 0 }, range04: (scenario * 43) as i8,
                flags09: (scenario >> 3 & 1) as u8, timer3d: (scenario >> 4 & 1) as u8, command3a: (scenario >> 5 & 1) as u8,
                acquisition_override: scenario >> 6 & 1 != 0, control10: (scenario >> 7 & 1) as u8, count12: (scenario >> 8 & 1) as u8 };
            hash(&mut h, &[scenario, i32::from(group_prevents_cover(g, probes[scenario as usize >> 9 & 7])), i32::from(group_needs_probe(g))]);
        }
        assert_eq!(h, 1782493944);
    }

    #[test]
    fn pose_counters_match_384_executed_native_cases() {
        let mut h = 2166136261;
        let mut scenario = 0;
        for facing in 0..16 {
            for phase in 0..2 {
                for blocked in [false, true] {
                    for variant in 0..6 {
                        let base = i16::from(facing & 14) * 3 / 2;
                        let mut c = Cover { state35: 43, frame32: [0, base, base + 1, base + 2, base + 3, 32767][variant], timer82: -1, animation89: 4, counter7e: 3, remaining80: (variant * 51) as u8 };
                        let done = tick_cover(&mut c, facing, phase, blocked);
                        hash(&mut h, &[scenario, i32::from(c.frame32), i32::from(c.remaining80), i32::from(done), i32::from(phase == 0)]);
                        scenario += 1;
                    }
                }
            }
        }
        assert_eq!(h, 567337925);
    }

    #[test]
    fn idle_prefix_matches_128_executed_native_cases_and_two_rng_calls() {
        let mut h = 2166136261;
        for scenario in 0..128 {
            let r = cover_idle_reset((scenario * 13) as u8, (scenario * 37) as u8, [(scenario * 2027 ^ 0x7731a5c9) as u32, (scenario * 7919 ^ 0x12348765) as u32]);
            hash(&mut h, &[scenario, i32::from(r.flags8a), i32::from(r.state35), i32::from(r.frame32), i32::from(r.countdown3d), i32::from(r.phase3c), i32::from(r.facing40), i32::from(r.phase41), i32::from(r.idle7d), 2]);
        }
        assert_eq!(h, 801765979);
    }
    #[test]
    fn dodge_gates_and_both_rng_coordinates_match_768_executed_native_cases() {
        let mut h = 2166136261;
        for s in 0..768 {
            let (class,state,flags31,flags64,flags69,skill,point,target) = if s < 512 {
                ([0,1,2,3,15,16,17,18][s as usize&7], [0,1,2,7,18,43,60,255][s as usize>>3&7], (4|((s>>6)&1)*64) as u8, (((s>>7)&1)*64) as u8, (((s>>8)&1)*128) as u8, 120i8, [500,500], [508,508])
            } else {
                let q = s-512;
                (0,1,4,0,((q&1)*128) as u8,(q*43) as i8,[(500+q*13%83-41) as i16,(500+q*17%83-41) as i16],[(508+(q%5-2)*32) as i16,(508+(q/5%5-2)*32) as i16])
            };
            let u = DodgeUnit { position:[500,500],flags31,flags64,flags69,class7c:class,state35:state,aim02:skill };
            let hit = dodge_candidate(u,1,point,target);
            let dest = if hit { dodge_destination(u.position,target,[(s*2027^0x7731a5c9) as u32,(s*7919^0x12348765) as u32]) } else { [-999,-999] };
            hash(&mut h,&[s,i32::from(hit),i32::from(dest[0]),i32::from(dest[1]),i32::from(if hit { flags64|64 } else { flags64 }),if hit { 2 } else { 0 },i32::from(hit&&class==1),if hit { 2 } else { 0 }]);
        }
        assert_eq!(h,80887623);
    }
}
