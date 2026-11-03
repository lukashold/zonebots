//! Native world-contact selection and damage arithmetic. Coordinates are raw
//! signed16 game pixels, before the browser's two-pixel scale. Pool order and
//! metadata are inputs; no approximate visual building bounds are substituted.

#[derive(Clone, Copy, Debug)]
pub struct Object {
    pub position: [i16; 2],
    pub flags13: u8,
    pub metadata_flags18: u8,
    pub metadata_extent20: i8,
}

///0041c600. The metadata extent is used for only the next lookup. The native
///callback clears that global even if the lookup fails.
pub fn object_contains(object: Object, point: [i16; 2], all_objects: bool, metadata_extent: bool) -> bool {
    if object.position[0] == -1 || (!all_objects && object.flags13 & 0x50 == 0) {
        return false;
    }
    let extent = if metadata_extent { i16::from(object.metadata_extent20) }
        else { i16::from(object.metadata_flags18.wrapping_add(1).wrapping_mul(8) & 24) };
    (0..2).all(|axis| object.position[axis] <= point[axis]
        && i32::from(point[axis]) - i32::from(extent) <= i32::from(object.position[axis]))
}

pub fn first_object(objects: &[Object], point: [i16; 2], all_objects: bool, metadata_extent: bool) -> Option<usize> {
    objects.iter().take(200).position(|&object| object_contains(object, point, all_objects, metadata_extent))
}

///0041eb90 searches these four points, stopping after the first matching pool
///slot. A metadata-extent override applies only to the first of these searches.
pub fn blast_object(objects: &[Object], origin: [i16; 2], all_objects: bool, metadata_extent: bool) -> Option<(usize, usize)> {
    for (sample, delta) in [[4, 16], [20, 16], [20, 32], [4, 32]].into_iter().enumerate() {
        let point = [origin[0].wrapping_add(delta[0]), origin[1].wrapping_add(delta[1])];
        if let Some(slot) = first_object(objects, point, all_objects, metadata_extent && sample == 0) {
            return Some((slot, sample));
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub position: [i16; 2],
    pub kind: i8,
    pub hp: i16,
}

//00498ca8: signed width/height in16-pixel tiles and signed result adjustments.
pub const BUILDING_SHAPES: [[i8; 4]; 9] = [
    [5, 4, 31, 28], [4, 5, 32, 60], [10, 10, 1, 1],
    [4, 5, 24, 8], [4, 5, 40, 48], [6, 6, 1, 1],
    [6, 6, 1, 1], [6, 6, 1, 1], [4, 3, 23, 9],
];

///0041c290, or0041c4f0 when `ruins` is true. Live lookups accept every nonzero
///HP, including negative values. Type2 is separately gated by allow_type2.
pub fn building_contact(building: Building, point: [i16; 2], allow_type2: bool, ruins: bool) -> Option<[i16; 2]> {
    let shape = *BUILDING_SHAPES.get(usize::try_from(building.kind).ok()?)?;
    if (building.hp == 0) != ruins || (!ruins && building.kind == 2 && !allow_type2) {
        return None;
    }
    let y = building.position[1].wrapping_sub(if building.kind == 2 { 16 } else { 0 });
    let width = i16::from(shape[0]) * 16;
    let height = i16::from(shape[1]) * 16 - if building.kind == 0 { 32 } else { 0 };
    if point[0] < building.position[0] || point[1] < y
        || i32::from(point[0]) > i32::from(building.position[0]) + i32::from(width)
        || i32::from(point[1]) > i32::from(y) + i32::from(height) {
        return None;
    }
    Some([point[0].wrapping_add(i16::from(shape[2])), point[1].wrapping_add(i16::from(shape[3]))])
}

pub fn first_building(buildings: &[Building], point: [i16; 2], allow_type2: bool, ruins: bool) -> Option<(usize, [i16; 2])> {
    buildings.iter().take(35).enumerate().find_map(|(slot, &building)| building_contact(building, point, allow_type2, ruins).map(|point| (slot, point)))
}

///0040e920: ordinary explosions use(+16,+16); fire's6e73 mode uses the raw
///origin. There is no ownership/faction filter for normal buildings.
pub fn building_blast_point(origin: [i16; 2], fire_mode: bool) -> [i16; 2] {
    let offset = if fire_mode { 0 } else { 16 };
    [origin[0].wrapping_add(offset), origin[1].wrapping_add(offset)]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damage16 { pub hp: i16, pub changed: bool, pub destroyed: bool }

pub fn building_damage(hp: i16, strength: i16) -> Damage16 {
    if hp == 0 { return Damage16 { hp, changed: false, destroyed: false }; }
    let after = hp.wrapping_sub(strength).max(0);
    Damage16 { hp: after, changed: true, destroyed: after == 0 }
}

///0041eb90 stores the wrapped low HP byte before testing the signed16
///difference. The force-destroy branch does not write that HP byte.
pub fn object_damage(hp: u8, strength: i16, force_destroy: bool) -> (u8, bool) {
    if force_destroy { (hp, true) }
    else { (hp.wrapping_sub(strength as u8), i16::from(hp).wrapping_sub(strength) <= 0) }
}

///0041e990/0041ea90. The high bit survives replacement of code28 by22.
pub fn clear_object_terrain(code: u8) -> u8 {
    if code & 127 == 28 { (code & 0x96) | 22 } else { code }
}

pub fn restored_wreck_terrain(code: u8) -> u8 { if code & 127 == 28 { 22 } else { code & 127 } }

pub fn restores_wreck_tiles(metadata_type: u16) -> bool {
    matches!(metadata_type, 6 | 24 | 25 | 27 | 30 | 32 | 34 | 36 | 38)
}

pub fn object_secondary_explosion(sprite: u16) -> bool { matches!(sprite, 350 | 351) }

#[derive(Clone, Copy, Debug)]
pub struct Bridge { pub position: [i16; 2], pub size: [i16; 2], pub hp: i16, pub maximum: i16, pub flags15: u8 }

///004093e0 returns the first matching one of12 bridge records. Destroyed
///flag2, rather than HP, gates lookup. Rectangle endpoints are inclusive.
pub fn bridge_contains(bridge: Bridge, point: [i16; 2]) -> bool {
    bridge.position[0] != -1 && bridge.flags15 & 2 == 0 && (0..2).all(|axis|
        point[axis] >= bridge.position[axis]
        && i32::from(point[axis]) <= i32::from(bridge.position[axis]) + i32::from(bridge.size[axis]))
}

///00409c50. Returns(newHP,newFlags,destructionCallback,halfDamageCallback).
pub fn bridge_damage(bridge: Bridge, strength: i16) -> (i16, u8, bool, bool) {
    if bridge.hp < 1 { return (bridge.hp, bridge.flags15, false, false); }
    let hp = bridge.hp.wrapping_sub(strength);
    if hp < 1 { (-1, bridge.flags15 | 2, true, false) }
    else if hp <= bridge.maximum >> 1 && bridge.flags15 & 32 == 0 { (hp, bridge.flags15 | 32, false, true) }
    else { (hp, bridge.flags15, false, false) }
}

///00409c50 passes this inclusive rectangle to459d10 and43f650. A horizontal
///bridge has orientation0; other orientation values use the vertical branch.
pub fn bridge_collapse_rectangle(bridge: Bridge, orientation: i16) -> [i16;4] {
    let [x,y]=bridge.position;let [w,h]=bridge.size;
    if orientation==0 {[x.wrapping_add(8),y.wrapping_sub(8),x.wrapping_add(w).wrapping_sub(8),y.wrapping_add(h).wrapping_add(8)]}
    else {[x.wrapping_sub(8),y,x.wrapping_add(w).wrapping_add(8),y.wrapping_add(h).wrapping_sub(8)]}
}
pub fn rectangle_contains(rect:[i16;4],point:[i16;2])->bool {
    point[0]>=rect[0]&&point[0]<=rect[2]&&point[1]>=rect[1]&&point[1]<=rect[3]
}
///459d10's bridge mode excludes fixed gun class2, irrespective of ownership.
pub fn collapse_unit(flags31:u8,class7c:u8,point:[i16;2],rect:[i16;4],bridge_mode:bool)->bool {
    flags31&4!=0&&(!bridge_mode||class7c!=2)&&rectangle_contains(rect,point)
}
///43f650 removes only these neutral hardware/wreck descriptor indices.
pub fn collapse_object(kind:u16,point:[i16;2],rect:[i16;4])->bool {
    point[0]>=0&&matches!(kind,8|25|26|29|30|31|32|33|34|35|36)&&rectangle_contains(rect,point)
}
///4094e0 replaces the two deck lanes with endpoint code50 and interior47.
///Returns the byte saved at bridge+14. Terrain high bits are preserved by the
///native masks. Inputs describe an authored bridge of at least two tiles.
pub fn collapse_bridge_terrain(bridge:Bridge,orientation:i16,mut read:impl FnMut(i16)->u8)->(Vec<(i16,u8)>,u8){
    let [x,y]=bridge.position;let base=((y as u16&0xfff0).wrapping_mul(8)).wrapping_add((x>>4)as u16)as i16;
    let horizontal=orientation==0;let length=if horizontal{bridge.size[0]>>4}else{bridge.size[1]>>4};
    if length<2{return(Vec::new(),0);}
    let step=if horizontal{1}else{128};let lanes=if horizontal{[128i16,256]}else{[1i16,2]};let mut out=Vec::new();
    for i in 0..length{for lane in lanes{let cell=base.wrapping_add(lane).wrapping_add(i.wrapping_mul(step));let old=read(cell);out.push((cell,if i==0||i==length-1{(old&0xb2)|50}else{(old&0xaf)|47}));}}
    (out,if horizontal{length as u8}else{length.wrapping_sub(1)as u8})
}

///409690 clears roads and bridge codes in the matched authored road rectangle,
///including its north/west one-tile fringe. The caller retires that road node.
pub fn collapsed_road_terrain(bounds:[i16;4],mut read:impl FnMut(i16)->u8)->Vec<(i16,u8)>{
    let mut out=Vec::new();let mut y=bounds[1].wrapping_sub(16);let min_x=bounds[0].wrapping_sub(16);
    while y<=bounds[3]{let mut x=min_x;while x<=bounds[2]{let cell=((y as u16&0xfff0).wrapping_mul(8)).wrapping_add((x>>4)as u16)as i16;let old=read(cell);if matches!(old&127,7|51|23){out.push((cell,old&128));}if x>i16::MAX-16{break;}x+=16;}if y>i16::MAX-16{break;}y+=16;}
    out
}

///0042fdb0 stores each destructible-wall column HP as a signed byte.
pub fn wall_damage(hp: i8, strength: u8) -> (i8, bool) {
    let after = hp.wrapping_sub(strength as i8);
    (after, after <= 0)
}

///0042fdb0 normalizes any front code18 to its wall column's head before
///searching the5000-entry HP table. The returned byte is global4a897b.
pub fn wall_anchor(cell:i16,mut terrain:impl FnMut(i16)->u8)->(i16,u8){
    let mut cell=cell;let mut height=1u8;
    if terrain(cell)&127==18||terrain(cell.wrapping_add(128))&127==18{
        cell=cell.wrapping_add(128);let mut visits=0;
        while terrain(cell)&127==18{cell=cell.wrapping_add(128);visits+=1;if visits>=512{return(cell,height);}}
        cell=cell.wrapping_sub(128);
        while terrain(cell)&127==18{cell=cell.wrapping_sub(128);height=height.wrapping_add(1);visits+=1;if visits>=1024{break;}}
    }
    (cell,height)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainContact { Bridge(i16), Wall(i16) }

///00420200 searches one mutually exclusive environment contact. Fire disables
///wall recognition via6f43, while bridges retain their place in the scan order.
pub fn environment_contact(origin: [i16; 2], suppress_walls: bool, mut terrain: impl FnMut(i16) -> u8) -> Option<TerrainContact> {
    let x = origin[0].wrapping_add(12) >> 4;
    let y = origin[1].wrapping_add(32) as u16;
    let base = (y & 0xfff0).wrapping_mul(8).wrapping_add(x as u16) as i16;
    for offset in [0i16, -1, 1, -128, 128] {
        let cell = base.wrapping_add(offset);
        let code = terrain(cell) & 127;
        if code == 23 { return Some(TerrainContact::Bridge(cell)); }
        if !suppress_walls && (matches!(code, 1..=6 | 18)) { return Some(TerrainContact::Wall(cell)); }
    }
    None
}

///0043f080 halves repeated flame damage inside the native BUILD or bridge
///rectangle, with a minimum of1. The bridge test runs only if BUILD misses.
pub fn fire_strength(base: u8, inside_structure: bool) -> u8 {
    if inside_structure { (u16::from(base) * 50 / 100).max(1) as u8 } else { base }
}

///43ecf0 followed by the43ef70 metadata flags. Flags from later runtime
///interactions (reservation bits, corpse transitions) must be retained by users.
pub fn initial_object_flags(kind: u16, metadata_flags: u8) -> u8 {
    let mut flags = if matches!(kind, 5 | 8 | 21 | 22 | 23 | 24 | 26 | 29 | 31 | 33 | 35 | 37) { 0x70 }
        else if matches!(kind, 11 | 12 | 17) { 0 } else { 0x80 };
    if metadata_flags & (4 | 16) != 0 { flags |= 64; }
    flags
}
///43ecf0 writes OBJECT byte19 (hex13), independently of flags byte13(hex0d).
///41cba0 separately replaces a flag object's team scan mask after allocation.
pub fn initial_object_scan_flags(kind:u16)->u8{if matches!(kind,5|8|9|10|21|22|23|24|26|29|31|33|35|37){15}else if matches!(kind,11|12|17){0}else{1}}
pub fn initial_object_bucket(point:[i16;2])->u8{(((point[1]>>3)&0xf0).wrapping_add(point[0]>>7))as u8}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FortDamage { pub hp: [i16; 2], pub destroyed: [bool; 2], pub first_nearby: bool }

fn inside(point: [i16; 2], lower: [i16; 2], upper: [i16; 2]) -> bool {
    (0..2).all(|axis| point[axis] >= lower[axis] && point[axis] <= upper[axis])
}

///0041fe30. First/second refer to globals622d54/622d58, never inferred team
///labels. With a source projectile and6e72=false, team1 may damage only the
///second fort; other source teams may damage only the first. HP0 still lives.
pub fn fort_damage(origin: [i16; 2], source_team: Option<u8>, allow_friendly: bool, bases: [[i16; 2]; 2], hp: [i16; 2], strength: i16) -> FortDamage {
    let point = [origin[0].wrapping_add(12), origin[1].wrapping_add(24)];
    let broad = [origin[0].wrapping_add(28), origin[1].wrapping_add(48)];
    let first = bases[0]; let second = bases[1];
    let first_hit = inside(point, [first[0], first[1].wrapping_sub(16)], [first[0].wrapping_add(160), first[1].wrapping_add(144)]);
    let second_hit = inside(point, second, [second[0].wrapping_add(160), second[1].wrapping_add(160)]);
    let mut result = FortDamage { hp, destroyed: [false; 2], first_nearby: source_team.is_some() && inside(broad, [first[0].wrapping_sub(16), first[1].wrapping_sub(16)], [first[0].wrapping_add(176), first[1].wrapping_add(176)]) };
    let hit_first = first_hit && (source_team.is_none() || allow_friendly || source_team.unwrap() & 3 != 1);
    if hit_first {
        result.hp[0] = hp[0].wrapping_sub(strength);
        result.destroyed[0] = result.hp[0] < 0;
        // The first-fort destruction callback returns before testing the second.
        if result.destroyed[0] || source_team.is_none() { return result; }
    }
    let hit_second = second_hit && (source_team.is_none() || allow_friendly || source_team.unwrap() & 3 == 1);
    if hit_second {
        result.hp[1] = hp[1].wrapping_sub(strength);
        result.destroyed[1] = result.hp[1] < 0;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(h: &mut u32, values: &[i32]) { for &v in values { *h = (*h ^ v as u32).wrapping_mul(16777619); } }
    const STRENGTHS: [i16; 8] = [-32768, -1, 0, 1, 10, 127, 255, 32767];

    #[test] fn object_search_matches_4096_executed_native_cases() {
        let mut h = 2166136261;
        for s in 0..4096 {
            let objects: Vec<_> = (0..3).map(|i| Object { position: [if s >> (i+6) & 1 != 0 { -1 } else { (500+i*6) as i16 }, (500+i*6) as i16], flags13: (s*37+i*83) as u8, metadata_flags18: (s*13+i*17) as u8, metadata_extent20: (s*43+i*7) as i8 }).collect();
            let slot = first_object(&objects, [(500+s%49-24) as i16, (500+s/49%49-24) as i16], s&1!=0, s>>1&1!=0).map(|n| n as i32).unwrap_or(-1);
            hash(&mut h, &[s, slot, 0]);
        }
        assert_eq!(h, 2187318226);
    }
    #[test] fn building_search_matches_4096_executed_native_cases() {
        let mut h = 2166136261;
        for s in 0..4096 {
            let buildings: Vec<_> = (0..3).map(|i| Building { position: [(500+i*24) as i16; 2], kind: ((s+i)%10-1) as i8, hp: [-1,0,1,500][(s >> (i+2) & 3) as usize] }).collect();
            let point = [(500+s%193-16) as i16, (500+s/193%193-16) as i16];
            let (slot, point) = first_building(&buildings, point, s&1!=0, s&2!=0).map(|(n,p)| (n as i32,p)).unwrap_or((-1,point));
            hash(&mut h, &[s, slot, i32::from(point[0]), i32::from(point[1])]);
        }
        assert_eq!(h, 3165437374);
    }
    #[test] fn object_damage_matches_4096_executed_native_cases() {
        let mut h = 2166136261; let mut s = 0;
        for hp in 0..256 { for strength in STRENGTHS { for force in [false,true] {
            let (hp, dead) = object_damage(hp as u8,strength,force); hash(&mut h, &[s,i32::from(hp),i32::from(dead)]); s+=1;
        }}}
        assert_eq!(h, 2495191201);
    }
    #[test] fn building_damage_and_contact_point_match_96_executed_native_cases() {
        let mut h = 2166136261; let mut s = 0;
        for hp in [-32768,-1,0,1,500,32767] { for strength in STRENGTHS { for fire in [false,true] {
            let r = building_damage(hp,strength); let point = building_blast_point([500,600],fire);
            hash(&mut h,&[s,i32::from(r.hp),i32::from(r.destroyed),i32::from(point[0]),i32::from(point[1])]); s+=1;
        }}}
        assert_eq!(h, 526384903);
    }
    #[test] fn fort_hitboxes_ownership_and_zero_hp_match_2048_executed_native_cases() {
        let mut h = 2166136261; let mut s = 0;
        let points = [[488,460],[488,620],[648,460],[648,620],[487,460],[649,620],[688,676],[848,836],[687,676],[849,836],[500,500],[700,700],[588,560],[788,736],[32767,32767],[-32768,-32768]];
        for source in [None,Some(0),Some(1),Some(2)] { for allow in [false,true] { for origin in points { for hp in [0,1,500,32767] { for strength in [0,1,127,32767] {
            let r = fort_damage(origin,source,allow,[[500,500],[700,700]],[hp,hp],strength);
            hash(&mut h,&[s,i32::from(r.hp[0]),i32::from(r.hp[1]),i32::from(r.destroyed[0]),i32::from(r.destroyed[1]),i32::from(r.first_nearby)]); s+=1;
        }}}}}
        assert_eq!(h, 3686365813);
    }
    #[test] fn bridge_damage_matches_192_executed_native_cases() {
        let mut h = 2166136261; let mut s = 0;
        for hp in [-32768,-1,0,1,100,32767] { for strength in STRENGTHS { for flags in [0,2,32,255] {
            let bridge = Bridge { position:[0,0],size:[0,0],hp,maximum:200,flags15:flags };
            let (hp,flags,dead,_) = bridge_damage(bridge,strength);hash(&mut h,&[s,i32::from(hp),i32::from(flags),i32::from(dead)]);s+=1;
        }}}
        assert_eq!(h, 3467777085);
    }
    fn terrain_hash(plane:&[u8])->u32{plane.iter().fold(2166136261,|h,&code|(h^u32::from(code)).wrapping_mul(16777619))}
    fn plane(s:i32)->Vec<u8>{(0..16384).map(|i|(i*13+s*37)as u8).collect()}
    #[test] fn bridge_terrain_matches_48_executed_native_cases(){
        let mut h=2166136261;
        for s in 0..48{let mut p=plane(s);let orientation=s&1;let length=2+s/2%12;let b=Bridge{position:[500,500],size:if orientation==0{[(length*16)as i16,48]}else{[48,(length*16)as i16]},hp:-1,maximum:600,flags15:2};let(changes,value)=collapse_bridge_terrain(b,orientation as i16,|cell|p[cell as usize]);for(cell,code)in changes{p[cell as usize]=code;}hash(&mut h,&[s,i32::from(value),terrain_hash(&p)as i32]);}
        assert_eq!(h,2712023461);
    }
    #[test] fn collapse_unit_filter_matches_256_executed_native_cases(){
        let mut h=2166136261;
        for s in 0..256{let mut mask=0;for i in 0..4{let point=[(500+(s*13+i*7)%65-32)as i16,(500+(s*17+i*11)%65-32)as i16];if collapse_unit(((s>>(i+1)&1)*4)as u8,((s+i)%4)as u8,point,[484,484,516,516],s&1!=0){mask|=1<<i;}}hash(&mut h,&[s,mask]);}
        assert_eq!(h,1315042306);
    }
    #[test] fn collapse_object_filter_and_terrain_match_256_executed_native_cases(){
        let mut h=2166136261;
        for s in 0..256{let mut p=plane(s);let mut mask=0;for i in 0..4{let point=[(500+(s*13+i*7)%65-32)as i16,(500+(s*17+i*11)%65-32)as i16];if collapse_object(((s+i)%39)as u16,point,[484,484,516,516]){mask|=1<<i;let base=(point[1]&!15)*8+(point[0]>>4);for(j,delta)in[0,1,128,129].into_iter().enumerate(){let cell=(base+delta)as usize;p[cell]=(p[cell]&128)|restored_wreck_terrain((s*19+i*37+j as i32*11)as u8);}}}hash(&mut h,&[s,mask,terrain_hash(&p)as i32]);}
        assert_eq!(h,2832172973);
    }
    #[test] fn collapsed_road_terrain_matches_32_executed_native_cases(){
        let mut h=2166136261;
        for s in 0..32{let mut p=plane(s);let bounds=[480,480,(528+s%4*16)as i16,(528+s/4%4*16)as i16];let changes=collapsed_road_terrain(bounds,|cell|p[cell as usize]);for(cell,code)in changes{p[cell as usize]=code;}hash(&mut h,&[s,0,terrain_hash(&p)as i32]);}
        assert_eq!(h,4129932469);
    }
    #[test] fn wall_signed_byte_damage_matches_1536_executed_native_cases(){
        let mut h=2166136261;let mut s=0;
        for hp in -128..128{for strength in[0,1,50,127,128,255]{let(hp,dead)=wall_damage(hp as i8,strength);hash(&mut h,&[s,i32::from(hp as u8),i32::from(dead)]);s+=1;}}
        assert_eq!(h,2528095643);
    }
    #[test] fn tall_wall_anchor_matches_64_executed_native_cases(){
        let mut h=2166136261;
        for s in 0..64{let head=20*128+20;let mut plane=vec![0;16384];let height=s/8%5;plane[head]=3;for j in 1..=height{plane[head+j*128]=18;}let(cell,height)=wall_anchor((head as i16)+((s&7)as i16-1)*128,|cell|plane[cell as usize]);hash(&mut h,&[s as i32,if cell==head as i16{9}else{10},i32::from(height)]);}
        assert_eq!(h,109752606);
    }
    #[test]fn object_constructor_scan_masks_and_buckets_match_234_executed_native_cases(){let mut h=2166136261;for scenario in 0..234{let kind=(scenario%78)as u16;let point=[(500+scenario*13)as i16,(500+scenario*17)as i16];hash(&mut h,&[scenario,i32::from(initial_object_scan_flags(kind)),i32::from(initial_object_flags(kind,0)),i32::from(initial_object_bucket(point))]);}assert_eq!(h,945562179);}
}

// Numerical template facts at00498f38, not graphical resources.
pub fn object_height(kind:u16)->Option<i8>{const HEIGHTS:[i8;78]=[16,16,16,16,16,16,16,16,26,16,16,16,16,0,0,0,0,16,16,16,10,24,20,20,26,20,24,26,0,26,26,32,32,24,24,26,32,32,32,32,32,32,20,16,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,32,16,32,32,32,32,32,32,32,32,32,8,0,0,0,0,0,0];HEIGHTS.get(usize::from(kind)).copied()}
pub fn object_metadata(kind:u16)->Option<(u16,u8,i8)>{
 const DATA:[[i32;3];78]=[
  [1254,1,16],
  [1254,1,16],
  [1254,1,16],
  [1254,1,16],
  [1254,0,16],
  [352,50,24],
  [512,2,32],
  [1254,1,16],
  [5165,50,32],
  [350,49,16],
  [351,49,16],
  [2937,1,16],
  [2938,1,16],
  [2634,2,0],
  [2642,2,0],
  [2650,2,0],
  [2658,2,0],
  [2939,1,16],
  [2936,1,16],
  [2935,1,16],
  [2285,2,10],
  [1641,50,32],
  [1657,50,32],
  [5765,50,32],
  [5169,50,32],
  [5317,2,32],
  [5168,50,32],
  [5695,2,32],
  [480,1,0],
  [5172,50,32],
  [5747,2,32],
  [5173,50,32],
  [5688,2,32],
  [5214,50,32],
  [5687,2,32],
  [5834,50,32],
  [5696,2,32],
  [5215,50,32],
  [5692,2,32],
  [5970,14,32],
  [5971,14,32],
  [5972,14,32],
  [5973,14,32],
  [5974,13,16],
  [5958,14,32],
  [5959,14,32],
  [5960,14,32],
  [5961,14,32],
  [5980,14,32],
  [5981,14,32],
  [5982,14,32],
  [5983,14,32],
  [5988,14,32],
  [5989,14,32],
  [5992,14,32],
  [5993,14,32],
  [5994,14,32],
  [5995,14,32],
  [1254,50,32],
  [1254,18,32],
  [5966,14,32],
  [5968,14,16],
  [6000,14,32],
  [5689,18,32],
  [5690,18,32],
  [5691,18,32],
  [5694,18,32],
  [1254,2,32],
  [1254,2,32],
  [1254,2,32],
  [1254,2,32],
  [5836,1,8],
  [557,1,0],
  [469,1,0],
  [5840,1,0],
  [5844,1,0],
  [5848,1,0],
  [5854,1,0]
 ];
 DATA.get(usize::from(kind)).map(|r|(r[0]as u16,r[1]as u8,r[2]as i8))
}
