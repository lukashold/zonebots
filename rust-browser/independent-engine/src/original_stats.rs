// Verified GOG data-table values. Damage resolution and movement timing remain separately researched.
use super::Stats;
pub fn data(k:&str)->Option<Stats>{Some(match k{
"cannon"=>Stats{hp:100.,speed:0.,range:240.,damage:75.,reload:2.3529411765,seconds:210.1633986928,members:1,stars:2,cls:2},
"medium"=>Stats{hp:200.,speed:35.,range:240.,damage:80.,reload:1.7647058824,seconds:406.8496732026,members:1,stars:2,cls:1},
"grunt"=>Stats{hp:30.,speed:40.,range:240.,damage:5.,reload:1.1764705882,seconds:127.8300653595,members:3,stars:1,cls:0},
"tough"=>Stats{hp:100.,speed:32.,range:240.,damage:40.,reload:1.1764705882,seconds:219.7908496732,members:2,stars:3,cls:0},
"sniper"=>Stats{hp:50.,speed:44.,range:288.,damage:10.,reload:1.1764705882,seconds:270.1111111111,members:3,stars:3,cls:0},
"heavy"=>Stats{hp:300.,speed:28.,range:288.,damage:120.,reload:2.3529411765,seconds:496.4052287582,members:1,stars:4,cls:1},
"psycho"=>Stats{hp:50.,speed:40.,range:240.,damage:2.,reload:0.1176470588,seconds:178.3856209150,members:3,stars:2,cls:0},
"light"=>Stats{hp:100.,speed:46.,range:240.,damage:50.,reload:1.1764705882,seconds:259.0326797386,members:1,stars:1,cls:1},
"missile"=>Stats{hp:100.,speed:0.,range:320.,damage:100.,reload:4.1176470588,seconds:415.3921568627,members:1,stars:5,cls:2},
"launcher"=>Stats{hp:300.,speed:32.,range:320.,damage:120.,reload:4.7058823529,seconds:575.3660130719,members:1,stars:5,cls:1},
"gatling"=>Stats{hp:50.,speed:0.,range:240.,damage:6.,reload:0.1764705882,seconds:176.6993464052,members:1,stars:1,cls:2},
"jeep"=>Stats{hp:50.,speed:72.,range:240.,damage:4.,reload:0.1176470588,seconds:150.4575163399,members:1,stars:1,cls:1},
"truck"=>Stats{hp:75.,speed:50.,range:0.,damage:0.,reload:5.8823529412,seconds:151.3006535948,members:1,stars:1,cls:1},
"apc"=>Stats{hp:200.,speed:55.,range:240.,damage:0.,reload:1.1764705882,seconds:219.0718954248,members:1,stars:3,cls:1},
"howitzer"=>Stats{hp:75.,speed:0.,range:400.,damage:100.,reload:5.2941176471,seconds:314.1633986928,members:1,stars:3,cls:2},
"firetruck"=>Stats{hp:200.,speed:50.,range:240.,damage:0.,reload:0.1176470588,seconds:181.8758169935,members:1,stars:4,cls:1},
"crane"=>Stats{hp:300.,speed:36.,range:240.,damage:0.,reload:2.9411764706,seconds:242.0588235294,members:1,stars:5,cls:1},
"pyro"=>Stats{hp:80.,speed:36.,range:240.,damage:20.,reload:0.1176470588,seconds:293.2222222222,members:4,stars:4,cls:0},
"laser"=>Stats{hp:60.,speed:40.,range:272.,damage:15.,reload:1.1764705882,seconds:326.9215686275,members:4,stars:5,cls:0},
_=>return None,})}
pub const PRODUCTS:&[(&str,usize,usize,usize,usize,i32,bool)]=&[
("cannon",0,2,3,2,1746,true),
("medium",1,2,0,3,3380,true),
("grunt",2,0,1,1,1062,true),
("tough",3,0,3,2,1826,true),
("sniper",4,0,3,3,2244,true),
("heavy",5,4,0,5,4124,true),
("psycho",6,0,2,2,1482,true),
("light",7,1,0,2,2152,true),
("missile",8,5,0,5,3451,true),
("launcher",9,5,0,5,4780,true),
("gatling",10,1,1,1,1468,true),
("jeep",11,1,0,1,1250,true),
("truck",12,0,0,0,1257,false),
("apc",13,3,0,3,1820,true),
("howitzer",14,3,5,4,2610,true),
("firetruck",15,4,0,4,1511,false),
("crane",16,5,0,5,2011,true),
("pyro",17,0,4,4,2436,true),
("laser",18,0,5,5,2716,true),
];
pub const MENU_ORDER:&[usize]=&[10,0,14,8,2,6,3,4,17,18,11,13,7,1,5,9,16];
pub fn workload(k:&str)->i32{PRODUCTS.iter().find(|r|r.0==k).map(|r|r.5*221/12).unwrap_or(1)}
pub fn required(k:&str,kind:&str)->usize{PRODUCTS.iter().find(|r|r.0==k).map(|r|match kind{"vehicle"=>r.2,"robot"=>r.3,"fort"=>r.4,_=>0}).unwrap_or(0)}
pub fn rate(hp:f64,max_hp:f64,owned:usize,total:usize)->i32{if max_hp<=0.||total==0{return 0;}let health=(hp*100./max_hp)as i32;let territory=1+((owned*100/total)>>2);health*territory as i32/100}
