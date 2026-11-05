use super::Move;

pub(crate) type Record = [u8;44];
pub(crate) const STAGING_SIZE: usize = 16;
pub(crate) const RING_SIZE: usize = 640;

pub(crate) struct Queue {
    pub staging: [Record;STAGING_SIZE],
    pub count: u8,
    pub rings: [Box<[Record;RING_SIZE]>;2],
    pub cursor: [i16;2],
}

impl Default for Queue {
    fn default()->Self {
        let mut blank=[0;44];blank[..4].fill(255);
        Self{staging:[[0;44];STAGING_SIZE],count:0,
            rings:std::array::from_fn(|_|vec![blank;RING_SIZE].into_boxed_slice().try_into().unwrap()),cursor:[0;2]}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Input {
    pub clock: i32,
    pub selection_flag: u8,
    pub auxiliary: u16,
    pub selection_context: [u16;12],
    pub ring_selector: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    StageFull,
    RingFull { stage: u8 },
    Copied { stage: u8, ring: usize, slot: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CursorBoundary { pub ring: usize, pub cursor: i16 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dispatch {
    pub forced: u8,
    pub selection_flag: u8,
    pub opcode: i8,
    pub slot: i16,
    pub auxiliary: i32,
    pub target: [i16;2],
    pub argument: u16,
    pub selection_context: [u16;12],
}

impl Dispatch {
    fn decode(row: &Record) -> Self {
        let word=|offset|u16::from_le_bytes([row[offset],row[offset+1]]);
        Self { forced:row[4],selection_flag:row[5],opcode:row[6] as i8,
            slot:word(8) as i16,auxiliary:i32::from(word(10) as i16),target:[word(12) as i16,word(14) as i16],
            argument:word(16),selection_context:std::array::from_fn(|i|word(18+2*i)) }
    }
}

pub(crate) trait DrainWorld {
    type Boundary;
    fn clock(&self) -> i32;
    fn set_local_router(&mut self, bypass:u8);
    fn local_order(&mut self, order:Dispatch) -> Result<(),Self::Boundary>;
    fn release_dispatch_context(&mut self);
}

impl Queue {
    /// The known opcode 1 / argument 3 form. Copy into the first vacant slot
    /// from the current cursor. A saturated ring still consumes staging.
    pub(crate) fn stage_forced_move(&mut self, command: Move, input: Input) -> Result<Route,CursorBoundary> {
        let Some((stage,row))=append(&mut self.staging,&mut self.count,command,input)else{return Ok(Route::StageFull);};
        let ring = usize::from(input.ring_selector==0);
        Ok(self.copy_to_ring(row,ring)?.map_or(Route::RingFull{stage},|slot|Route::Copied{stage,ring,slot}))
    }

    /// 45eaf0 receives the other peer's raw record into the opposite ring.
    fn receive(&mut self,row:Record,local_selector:u16)->Result<Option<usize>,CursorBoundary>{
        self.copy_to_ring(row,usize::from(local_selector!=0))
    }

    fn copy_to_ring(&mut self,row:Record,ring:usize)->Result<Option<usize>,CursorBoundary>{
        let cursor = self.cursor[ring];
        let start = usize::try_from(cursor).ok().filter(|&pos|pos<RING_SIZE)
            .ok_or(CursorBoundary {ring,cursor})?;
        for distance in 0..RING_SIZE {
            let slot = (start+distance)%RING_SIZE;
            let target = &mut self.rings[ring][slot];
            if target[..4]==[255;4] {
                *target = row;
                self.cursor[ring]=((slot+1)%RING_SIZE) as i16;
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }

    /// The Main consumer visits the first then the second ring in physical
    /// order. Earlier or later clock values are not due. Each matching row
    /// is retired before local dispatch, leaving the rest of its bytes warm.
    pub(crate) fn drain_due<W:DrainWorld>(&mut self, world:&mut W, multiplayer:bool) -> Result<(),W::Boundary> {
        if !multiplayer { return Ok(()); }
        world.set_local_router(1);
        for ring in &mut self.rings {
            for row in ring.iter_mut() {
                if i32::from_le_bytes(row[..4].try_into().unwrap()) != world.clock() { continue; }
                let order=Dispatch::decode(row);
                row[..4].fill(255);
                world.local_order(order)?;
                world.release_dispatch_context();
            }
        }
        world.set_local_router(0);
        Ok(())
    }
}

fn append(staging:&mut [Record;STAGING_SIZE],count:&mut u8,command:Move,input:Input)->Option<(u8,Record)>{
    let stage=*count;
    let row=staging.get_mut(usize::from(stage))?;
    *count+=1;
    row[..4].copy_from_slice(&input.clock.wrapping_add(20).to_le_bytes());
    row[4..7].copy_from_slice(&[1,input.selection_flag,1]);
    let [x,y]=command.target;
    let fixed=[u16::from(command.slot),input.auxiliary,x as u16,y as u16,3];
    for (out,value) in row[8..42].chunks_exact_mut(2).zip(fixed.into_iter().chain(input.selection_context)){
        out.copy_from_slice(&value.to_le_bytes());
    }
    Some((stage,*row))
}

/// The Web host runs one world. Each original peer generated the automatic
/// Fort order, so one local row and one received row are still required.
pub(crate) struct PeerOrders {
    pub(crate) host:Queue,
    guest:[Record;STAGING_SIZE],
    guest_count:u8,
}

impl Default for PeerOrders {
    fn default()->Self{Self{host:Queue::default(),guest:[[0;44];STAGING_SIZE],guest_count:0}}
}

impl PeerOrders {
    pub(crate) fn stage(&mut self,command:Move,host:Input,guest:Input)->Result<(),CursorBoundary>{
        self.host.stage_forced_move(command,host)?;
        append(&mut self.guest,&mut self.guest_count,command,guest);
        Ok(())
    }

    /// The original UI clears the staging count after sending. The raw
    /// staging cells themselves stay warm for the next callback.
    pub(crate) fn finish_callback(&mut self,host_selector:u16)->Result<(),CursorBoundary>{
        for i in 0..usize::from(self.guest_count){self.host.receive(self.guest[i],host_selector)?;}
        self.guest_count=0;
        self.host.count=0;
        Ok(())
    }
}

#[cfg(test)]
#[path = "original_fort_order_queue_tests.rs"]
mod tests;
