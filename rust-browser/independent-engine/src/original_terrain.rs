//! Native terrain path masks, indexed by map attribute &0x7f (not visual tile).
//! 0041b950 selects unit+0x18; 0045f6a0 copies these 67-byte defaults to team
//! tables. 00433b30 forbids9/38/39/42 for infantry in native Volcan world3,
//! which is world1 in this Web engine. Terrain40 remains blocked everywhere.
//! The original also has dynamic occupancy/gate masks, a Tough-leader path
//! variant, and an APC hidden-class3 override; those hidden states are separate
//! from this four-profile adapter. Collision masks at unit+0x1c are different:
//! e.g. ordinary ground0 is temporarily unblocked by0045a5c0.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile { Infantry, Tracked, Wheeled, Crane }

pub fn profile(kind:&str)->Profile {
    match kind {
        "grunt"|"tough"|"psycho"|"sniper"|"pyro"|"laser"=>Profile::Infantry,
        "jeep"|"apc"=>Profile::Wheeled,
        "crane"=>Profile::Crane,
        // Family38 truck/firetruck uses the ordinary vehicle mask in0041b950.
        _=>Profile::Tracked,
    }
}

// Original path default0x00499888;0 passes,255 blocks.
const INFANTRY:[u8;67]=[
    0,255,255,255,255,255,255,0,0,0,0,255,255,255,255,255,
    255,255,255,255,0,0,0,0,255,255,0,0,255,255,0,255,
    255,255,255,0,0,0,0,0,255,0,0,255,0,0,255,255,
    255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,
    0,0,0,
];

// Original path default0x0049a840;0 passes,255 blocks.
const TRACKED:[u8;67]=[
    0,0,0,0,0,0,0,0,0,255,0,255,255,255,255,255,
    255,0,0,255,0,0,0,0,255,255,0,255,0,255,0,255,
    255,0,0,0,0,255,255,255,255,0,255,255,0,0,0,255,
    255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,
    0,0,0,
];

// Original path default0x0049a8a0;0 passes,255 blocks.
const WHEELED:[u8;67]=[
    0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,
    255,255,255,255,0,0,0,0,255,255,0,255,255,255,0,255,
    255,0,0,0,0,255,255,255,255,0,255,255,0,0,255,255,
    255,255,255,0,255,0,0,0,255,255,255,0,0,0,0,0,
    0,0,0,
];

// Original path default0x0049a930;0 passes,255 blocks.
const CRANE:[u8;67]=[
    0,255,255,255,255,255,255,0,0,255,0,255,255,255,255,255,
    255,255,255,255,0,0,0,0,255,255,0,255,255,255,0,255,
    255,0,0,0,0,255,255,255,255,0,255,255,0,0,255,0,
    0,0,0,0,255,0,0,0,0,0,0,0,0,0,0,0,
    0,0,0,
];

pub fn passable(code:u8,world:usize,kind:Profile)->bool {
    let code=code&127;
    if kind==Profile::Infantry && world==1 && matches!(code,9|38|39|42) { return false; }
    let table=match kind {Profile::Infantry=>&INFANTRY,Profile::Tracked=>&TRACKED,
        Profile::Wheeled=>&WHEELED,Profile::Crane=>&CRANE};
    table.get(code as usize)==Some(&0)
}


//00499b50 contains signed speed modifiers AND action codes100..116. Actions
//resolve before speed conversion; their return values0..3 add no full /10 step.
//00433b30 always overwrites code40 with-10 at world initialization.
pub const TERRAIN_EFFECT:[i8;67]=[
    0,0,0,0,0,0,0,20,-20,-10,0,0,0,0,0,0,
    0,20,0,0,100,101,-20,20,0,0,102,103,0,0,0,90,
    0,104,105,106,107,108,0,-20,109,110,-30,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,0,0,-20,111,112,0,0,
    0,0,0,
];

//Original planner prices0x00499b98; membership/passability is separate.
pub const INFANTRY_ROUTE_COST:[u8;67]=[
    5,12,12,12,12,12,12,2,7,8,5,12,12,12,12,12,
    12,12,12,12,5,5,7,2,5,5,12,12,12,12,5,5,
    12,12,12,5,5,5,5,7,12,5,10,12,12,5,12,12,
    12,12,12,6,12,5,5,5,12,12,12,7,5,5,5,5,
    5,5,5,
];

//Original planner prices0x00499ec8; membership/passability is separate.
pub const VEHICLE_ROUTE_COST:[u8;67]=[
    5,12,12,12,12,12,12,2,7,72,5,12,12,12,12,12,
    12,12,12,12,5,5,7,2,5,5,12,12,12,72,5,5,
    12,12,12,5,5,5,72,72,72,72,72,12,12,5,12,12,
    12,12,12,6,12,5,5,5,12,12,12,7,5,5,5,5,
    5,5,5,
];

//Pattern means from00499fd4: sequences of native pixels/update, terminated-1.
const MEAN_STEP:[f64;13]=[1./5.,1./4.,1./3.,2./5.,1./2.,2./3.,4./5.,1.,4./3.,3./2.,2.,7./3.,5./2.];

/// Relative terrain speed in the ordinary native movement state. Absolute
/// scheduler, pattern phase, squad boost+3 and unit+0x8c are deliberately separate.
pub fn speed_factor_for(code:u8,movement_control:u8)->f64 {
    let code=code&127;
    let modifier=if code==40 {-10} else {
        TERRAIN_EFFECT.get(code as usize).copied().filter(|v|*v<100).unwrap_or(0)
    };
    let raw=i32::from(movement_control)/10;
    let base=raw.clamp(0,12)as usize;
    let adjusted=(raw+i32::from(modifier)/10).clamp(0,12)as usize;
    MEAN_STEP[adjusted]/MEAN_STEP[base]
}

/// Damage changes the same quantized control index as terrain, rather than
/// multiplying speed by a continuous control percentage. Keep the immobile
/// profile's special base0 behavior separate from the slowest moving pattern.
pub fn control_speed_factor(current:u8,base:u8)->f64 {
    if base==0 {return 1.;}
    MEAN_STEP[(usize::from(current)/10).min(12)]/MEAN_STEP[(usize::from(base)/10).min(12)]
}

/// Original route-comparison prices from0043bbe0/0043e190 ->0045cf90.
/// The new A* adapter uses these terrain prices; native consecutive-price>=10
/// halving and its original route-candidate planner remain separate.
pub fn route_cost(code:u8,kind:Profile)->u8 {
    let table=if kind==Profile::Infantry {&INFANTRY_ROUTE_COST}else{&VEHICLE_ROUTE_COST};
    table.get((code&127)as usize).copied().unwrap_or(12).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn near(a:f64,b:f64){assert!((a-b).abs()<1e-10,"{a} != {b}");}
    #[test]
    fn native_terrain_speed_uses_control_and_quantized_pattern_ratios() {
        near(speed_factor_for(0,40),1.);
        near(speed_factor_for(7,40),1.6);
        near(speed_factor_for(23,40),1.6);
        near(speed_factor_for(51,40),1.);
        near(speed_factor_for(9,40),0.8);
        near(speed_factor_for(38,40),1.);
        near(speed_factor_for(39,40),2./3.);
        near(speed_factor_for(42,40),0.5);
        near(speed_factor_for(7,30),5./3.);
        near(speed_factor_for(7,50),1.5);
        near(speed_factor_for(7,60),5./3.);
        near(speed_factor_for(42,0),1.);
        near(speed_factor_for(7,120),1.);
        near(speed_factor_for(9,130),1.);
        near(speed_factor_for(7|128,40),1.6);
    }
    #[test]
    fn damaged_control_and_terrain_compose_through_the_same_native_pattern() {
        near(control_speed_factor(45,50),0.75);
        near(control_speed_factor(50,50),1.);
        near(control_speed_factor(0,50),0.3);
        near(control_speed_factor(20,0),1.);
        near(control_speed_factor(255,120),1.);
        //Base50 -> damaged45 -> road index6: (1/2)/(2/3) * (4/5)/(1/2).
        near(control_speed_factor(45,50)*speed_factor_for(7,45),1.2);
        near(control_speed_factor(45,50)*speed_factor_for(0,45),0.75);
        near(control_speed_factor(32,40)*speed_factor_for(39,32),(1./4.)/(1./2.));
    }
    #[test]
    fn original_route_prices_prefer_roads_and_bridges_without_making_51_a_road() {
        for p in [Profile::Infantry,Profile::Tracked,Profile::Wheeled,Profile::Crane] {
            assert_eq!(route_cost(0,p),5);
            assert_eq!(route_cost(7,p),2);
            assert_eq!(route_cost(23,p),2);
            assert_eq!(route_cost(51,p),6);
        }
        assert_eq!(route_cost(9,Profile::Infantry),8);
        assert_eq!(route_cost(38,Profile::Infantry),5);
        assert_eq!(route_cost(39,Profile::Infantry),7);
        assert_eq!(route_cost(42,Profile::Infantry),10);
        assert_eq!(route_cost(9,Profile::Tracked),72);
        assert_eq!(route_cost(127,Profile::Infantry),12);
    }
    #[test]
    fn native_partial_water_distinguishes_foot_vehicles_and_volcan() {
        for world in 0..5 {
            for code in [9,38,39,42] {
                assert_eq!(passable(code,world,Profile::Infantry),world!=1);
                for hardware in [Profile::Tracked,Profile::Wheeled,Profile::Crane] {
                    assert!(!passable(code,world,hardware));
                }
            }
            for p in [Profile::Infantry,Profile::Tracked,Profile::Wheeled,Profile::Crane] {
                assert!(!passable(40,world,p),"Blocked wet40 must not become all-water swimming");
                assert!(passable(7,world,p),"Road");
                assert!(passable(23,world,p),"Intact bridge");
            }
        }
    }
    #[test]
    fn vehicle_families_preserve_native_mask_differences() {
        assert_eq!(profile("jeep"),Profile::Wheeled);
        assert_eq!(profile("apc"),Profile::Wheeled);
        assert_eq!(profile("medium"),Profile::Tracked);
        assert_eq!(profile("truck"),Profile::Tracked);
        assert_eq!(profile("crane"),Profile::Crane);
        assert!(passable(1,0,profile("medium")));
        assert!(!passable(1,0,profile("jeep")));
        for code in [47,48,49,50,56,57,58] {
            assert!(passable(code,0,Profile::Crane));
            assert!(!passable(code,0,Profile::Tracked));
            assert!(!passable(code,0,Profile::Wheeled));
        }
    }
    #[test]
    fn terrain_flags_are_masked_and_unknown_codes_fail_closed() {
        for p in [Profile::Infantry,Profile::Tracked,Profile::Wheeled,Profile::Crane] {
            for world in 0..5 {
                for code in 0..=66 {assert_eq!(passable(code,world,p),passable(code|128,world,p));}
                for code in [19,31,32,43,52,67,127] {assert!(!passable(code,world,p));}
            }
        }
    }
}
