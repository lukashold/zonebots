use super::{Dispatch,DrainWorld,Input,Move,Queue,Record,Route};
use serde_json::Value;

fn decode(value:&Value)->Vec<u8> {
    value.as_str().unwrap().as_bytes().chunks_exact(2)
        .map(|bytes| u8::from_str_radix(std::str::from_utf8(bytes).unwrap(),16).unwrap()).collect()
}

fn rows(value:&Value)->Vec<Record> {
    decode(value).chunks_exact(44).map(|s|s.try_into().unwrap()).collect()
}

#[test]
#[ignore = "requires private original multiplayer staging trace"]
fn original_fort_moves_match_both_raw_rings_and_warm_staging_rows() {
    let path=std::env::var("NATIVE_FORT_SWEEP_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    let raw:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(raw.len(),192);
    let mut staged=0;
    let mut copied=0;
    let mut rejected=0;
    let mut ring_full=0;
    for (id,pair) in raw.chunks_exact(2).enumerate() {
        let (before,after)=(&pair[0],&pair[1]);
        let b=|n:usize| before["fields"][n].as_u64().unwrap();
        let a=|n:usize| after["fields"][n].as_u64().unwrap();
        let mut queue=Queue { staging:rows(&before["staging"]).try_into().unwrap(),
            count:b(3) as u8,cursor:[b(4) as i16,b(5) as i16],rings:["ringA","ringB"].map(|key|rows(&before[key]).into_boxed_slice().try_into().unwrap()) };
        let events=after["visits"].as_array().unwrap().iter().filter(|v|v[0]==0x45ebd0);
        for event in events {
            assert_ne!(b(0),0);assert_eq!(b(1),0);
            let slot=((event[1].as_u64().unwrap()-0x5e30c8)/192) as u8;
            let result=queue.stage_forced_move(Move{slot,target:[a(15) as i16,a(16) as i16]},Input{
                clock:0x10203040,sender:69,auxiliary:7,
                selection_context:std::array::from_fn(|i|1000+id as u16+i as u16*101),ring_selector:b(2) as u16}).unwrap();
            match result {
                Route::StageFull=>rejected+=1,
                Route::RingFull{..}=>{staged+=1;ring_full+=1;},
                Route::Copied{..}=>{staged+=1;copied+=1;},
            }
        }
        assert_eq!(queue.staging.as_slice(),rows(&after["staging"]).as_slice(),"staging {id}");
        assert_eq!(queue.rings[0].as_slice(),rows(&after["ringA"]).as_slice(),"ring A {id}");
        assert_eq!(queue.rings[1].as_slice(),rows(&after["ringB"]).as_slice(),"ring B {id}");
        assert_eq!(queue.cursor,[a(4) as i16,a(5) as i16]);
        assert_eq!(u64::from(queue.count),a(3));
    }
    assert_eq!((staged,copied,rejected,ring_full),(72,30,54,42));
}

const SLOTS: [usize;6] = [0,1,8,319,638,639];

struct Consumer {
    clock:i32,
    bypass:u8,
    sender:u8,
    forced:u8,
    local_mode:u8,
    clock_change:bool,
    recorded:Vec<Value>,
    stamps:[u32;12],
}

impl DrainWorld for Consumer {
    type Boundary = std::convert::Infallible;
    fn clock(&self)->i32 { self.clock }
    fn set_local_router(&mut self,bypass:u8) { self.bypass=bypass; }
    fn local_order(&mut self,order:Dispatch)->Result<(),Self::Boundary> {
        self.sender=order.sender;
        self.forced=order.forced;
        self.local_mode=0;
        let (ring,slot)=if order.sender>=30 {(1,order.sender-30)}else{(0,order.sender-10)};
        self.stamps[ring*6+usize::from(slot)]=u32::MAX;
        self.recorded.push(serde_json::json!([order.slot,order.sender,order.forced,self.bypass,order.opcode,
            0x12340000u32|u32::from(order.target[0] as u16),0x56780000u32|u32::from(order.target[1] as u16),
            order.argument,order.auxiliary as u32,self.local_mode,self.stamps,order.selection_context]));
        if self.clock_change&&self.recorded.len()==1 { self.clock+=1; }
        Ok(())
    }
    fn release_dispatch_context(&mut self) { self.sender=0;self.forced=0; }
}

#[test]
#[ignore = "requires private original multiplayer ring consumption trace"]
fn original_fort_order_consumer_matches_due_clock_physical_order_and_reentrant_clock_change() {
    let path=std::env::var("NATIVE_FORT_RING_CONSUME_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    assert_eq!(text.lines().count(),96);
    let mut checked=0;
    for (id,line) in text.lines().enumerate() {
        let row:Value=serde_json::from_str(line).unwrap();
        let n=|key:&str| row[key].as_u64().unwrap();
        assert_eq!(id as u64,n("id"));
        let stamps=std::array::from_fn(|i|row["before"][i].as_u64().unwrap() as u32);
        let mut queue=Queue{staging:[[0;44];16],count:0,cursor:[31,46],
            rings:std::array::from_fn(|_| {let mut empty=[0;44];empty[..4].fill(255);vec![empty;640].into_boxed_slice().try_into().unwrap()})};
        for (ring,records) in queue.rings.iter_mut().enumerate() {
            for (j,&slot) in SLOTS.iter().enumerate() {
                let r=&mut records[slot];
                *r=std::array::from_fn(|p|(id*17+ring*79+j*27+p*13) as u8);
                r[..4].copy_from_slice(&stamps[ring*6+j].to_le_bytes());
                r[4..7].copy_from_slice(&[1,10+ring as u8*20+j as u8,1]);
                let fields=[((j*37+ring*19+id)%240) as u16,((id*617+j*7) as u16).wrapping_sub(31000),
                    (200+id*11+ring*42+j) as u16,(900+id*13+ring*57+j) as u16,3];
                for (out,value) in r[8..18].chunks_exact_mut(2).zip(fields) {out.copy_from_slice(&value.to_le_bytes());}
            }
        }
        let before=queue.rings.clone();
        let mut consumer=Consumer{clock:n("clock") as i32,bypass:77,sender:59,forced:36,local_mode:81,
            clock_change:id>=48,recorded:vec![],stamps};
        queue.drain_due(&mut consumer,n("mp")!=0).unwrap();
        assert_eq!(consumer.recorded.as_slice(),row["visits"].as_array().unwrap(),"row {id}");
        assert_eq!([u64::from(consumer.bypass),u64::from(consumer.sender),u64::from(consumer.forced),u64::from(consumer.local_mode),consumer.clock as u64],
            ["bypass","sender","forced","localMode","finalClock"].map(n));
        let actual:Vec<u64>=queue.rings.iter().flat_map(|ring| SLOTS.into_iter().map(move |slot|u64::from(u32::from_le_bytes(ring[slot][..4].try_into().unwrap())))).collect();
        assert_eq!(actual,(0..12).map(|i|row["after"][i].as_u64().unwrap()).collect::<Vec<_>>(),"final stamps {id}");
        for (r,b) in queue.rings.iter().zip(before) {
            assert!(r.iter().zip(b.iter()).all(|(after,previous)|after[4..]==previous[4..]),"warm bytes {id}");
        }
        assert_eq!(queue.cursor,[31,46]);
        checked+=consumer.recorded.len();
    }
    assert_eq!(checked,78);
}
