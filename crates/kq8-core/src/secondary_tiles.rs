//! Swamp and Barren controller rules independent of animation/rendering.
//! Source: behaviour/secondary-tile-puzzles.md, rules 1–15 and 26–35.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind { Swamp, Barren }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell { pub col: i32, pub row: i32 }
impl Cell { pub const fn new(col: i32, row: i32) -> Self { Self { col, row } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where { Cell(Cell), South, North, West, East }

impl Kind {
    pub fn dimensions(self) -> (i32,i32) { match self { Self::Swamp => (6,7), Self::Barren => (7,5) } }
    pub fn actual(self, c: Cell) -> bool { let (w,h)=self.dimensions(); (0..w).contains(&c.col) && (0..h).contains(&c.row) }
    pub fn name(self, c: Cell) -> String { match self { Self::Swamp => format!("tile{}{}",c.col,c.row), Self::Barren => format!("tile{}{}",c.row,c.col) } }
    pub fn where_is(self, p: [f32;3]) -> Where {
        let (x0,x1,y0,y1,first_col)=match self { Self::Swamp=>(45000.,65000.,91600.,105600.,-2),Self::Barren=>(85640.,99640.,150943.,160943.,0) };
        if p[1]<=y0 { Where::South } else if p[1]>=y1 { Where::North }
        else if p[0]<=x0 { Where::West } else if p[0]>=x1 { Where::East }
        else { Where::Cell(Cell::new(((p[0]-x0)/2000.) as i32+first_col,((p[1]-y0)/2000.) as i32)) }
    }
}

/// Only named, loaded positions are copied. Unpopulated off-route approach
/// slots are absent rather than synthesized into extra floor geometry.
#[derive(Debug, Clone)]
pub struct Grid { pub kind: Kind, places: BTreeMap<Cell,[f32;3]> }
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Choice { pub cell: Option<Cell>, pub point: [f32;3], pub diagonal: bool, pub exit: bool, pub ordinary: bool }
impl Choice {
    pub fn motion(self, back: bool) -> &'static str {
        if self.ordinary { return "jump"; }
        match (back,self.diagonal) { (false,false)=>"tilejump1",(false,true)=>"tilejump2",(true,false)=>"tilebackjump",(true,true)=>"tilebackjump2" }
    }
    pub fn turn_point(self, from: [f32;3], back: bool) -> [f32;3] {
        if back { [2.*from[0]-self.point[0],2.*from[1]-self.point[1],from[2]] } else { self.point }
    }
}

impl Grid {
    pub fn read(kind: Kind, place_of: impl Fn(&str)->Option<[f32;3]>) -> (Self,Vec<String>) {
        let mut places=BTreeMap::new(); let mut missing=Vec::new(); let (w,h)=kind.dimensions();
        for col in 0..w { for row in 0..h { let c=Cell::new(col,row); let name=kind.name(c);
            if let Some(p)=place_of(&name) { places.insert(c,p); } else { missing.push(name); }
        }}
        let approaches: Vec<(Cell,Cell,[f32;2])>=match kind {
            Kind::Swamp=>vec![(Cell::new(-2,3),Cell::new(0,3),[46000.,98600.]),(Cell::new(-1,3),Cell::new(0,3),[48000.,98600.]),(Cell::new(6,3),Cell::new(5,3),[62000.,98600.]),(Cell::new(7,3),Cell::new(5,3),[64000.,98600.])],
            Kind::Barren=>vec![(Cell::new(3,-1),Cell::new(3,0),[92640.,149943.]),(Cell::new(3,5),Cell::new(3,4),[92640.,161943.])],
        };
        for (cell,near,xy) in approaches { if let Some(p)=places.get(&near).copied() { places.insert(cell,[xy[0],xy[1],p[2]]); } }
        (Self {kind,places},missing)
    }
    pub fn place(&self,cell:Cell)->Option<[f32;3]> { self.places.get(&cell).copied() }
    pub fn found(&self,cell:Cell)->bool { self.kind.actual(cell) && self.places.contains_key(&cell) }
    fn candidate(&self, from: Cell, to: Cell, exit: bool)->Option<Choice> {
        Some(Choice {cell:Some(to),point:self.place(to)?,diagonal:from.col!=to.col && from.row!=to.row,exit,ordinary:false})
    }
    fn neighbors(&self,from:Cell,solved:bool,highlight:bool)->Vec<Choice> {
        let mut choices=Vec::new();
        for col in from.col-1..=from.col+1 { for row in from.row-1..=from.row+1 {
            let to=Cell::new(col,row); if to==from {continue;}
            let actual=self.kind.actual(to);
            let allowed=match self.kind {
                Kind::Swamp=>actual || (row==3 && (-2..=7).contains(&col)),
                Kind::Barren if highlight=>actual,
                Kind::Barren=>actual && if solved {col==3} else {!(row==4 && (2..=4).contains(&col))},
            };
            if !allowed {continue;}
            if self.kind==Kind::Swamp && !self.kind.actual(from) && from.col!=to.col && from.row!=to.row && !actual {continue;}
            if let Some(c)=self.candidate(from,to,false) {choices.push(c);}
        }}
        if self.kind==Kind::Barren && !highlight {
            if from.row==0 && (2..=4).contains(&from.col) {if let Some(c)=self.candidate(from,Cell::new(3,-1),true) {choices.push(c);}}
            if from.row==4 && solved {if let Some(c)=self.candidate(from,Cell::new(3,5),true) {choices.push(c);}}
        }
        // x first, then y, including the permitted approach candidate.
        choices.sort_by_key(|c|c.cell);
        choices
    }
    pub fn choose(&self,p:[f32;3],heading:f32,back:bool,solved:bool,highlight:bool)->Option<Choice> {
        let choices=match self.kind.where_is(p) {
            Where::Cell(from)=>self.neighbors(from,solved,highlight),
            _ if highlight=>return None,
            Where::West|Where::East if self.kind==Kind::Swamp=>return Some(Choice {cell:None,point:p,diagonal:false,exit:false,ordinary:true}),
            Where::South if self.kind==Kind::Barren=>{
                let mut choices=Vec::new(); for col in 2..=4 {if let Some(c)=self.candidate(Cell::new(3,-1),Cell::new(col,0),false) {choices.push(c);}}
                choices.push(Choice {cell:None,point:[92640.,144000.,1000.],diagonal:false,exit:true,ordinary:false});choices
            }
            Where::North if self.kind==Kind::Barren=>{
                let mut choices=self.candidate(Cell::new(3,5),Cell::new(3,4),false).into_iter().collect::<Vec<_>>();
                choices.push(Choice {cell:None,point:[92640.,162000.,1000.],diagonal:false,exit:true,ordinary:false});choices
            }
            _=>return None,
        };
        let aim=heading+if back {std::f32::consts::PI} else {0.}; let facing=[aim.sin(),-aim.cos()];
        let mut best=None;
        for c in choices {let dx=c.point[0]-p[0];let dy=c.point[1]-p[1]; let len=dx.hypot(dy); if len==0. {continue;}
            let dot=(dx*facing[0]+dy*facing[1])/len;
            if best.is_none_or(|(_,score)|dot>score) {best=Some((c,dot));}
        }
        best.map(|(c,_)|c)
    }
}

pub const PRIMES: [(Cell,u8);7]=[(Cell::new(2,1),3),(Cell::new(1,4),5),(Cell::new(6,3),7),(Cell::new(5,0),11),(Cell::new(0,2),13),(Cell::new(5,4),17),(Cell::new(4,2),19)];
pub const LAVA_BITMAPS: [&str;8]=["lvtile01.bmp","lvtile03.bmp","lvtile05.bmp","lvtile07.bmp","lvtile11.bmp","lvtile13.bmp","lvtile17.bmp","lvtile19.bmp"];
#[derive(Debug,Clone,Default,PartialEq,Serialize,Deserialize)]
pub struct Sequence {pub previous:Option<u8>,pub valid:bool,pub revealed:u8,pub lower_final:bool}
#[derive(Debug,Clone,Default,PartialEq)]
pub struct Landing {pub cues:Vec<(String,i32,Option<String>)>,pub sound:Option<String>,pub complete:bool}
impl Sequence {
    pub fn land(&mut self,cell:Option<Cell>,solved:bool)->Landing {
        let mut out=Landing::default();
        if self.lower_final && cell!=Some(Cell::new(4,2)) {out.cues.push(("tile24".into(),2401,None));self.lower_final=false;}
        if solved {return out;}
        let Some((index,(_,prime)))=PRIMES.iter().enumerate().find(|(_, (c,_))|Some(*c)==cell) else {return out;};
        self.valid=index==0 || (self.valid && self.previous==Some(PRIMES[index-1].1)); self.previous=Some(*prime);
        if self.revealed & (1<<index)==0 {self.revealed|=1<<index;out.cues.push((Kind::Barren.name(cell.unwrap()),2400,Some(format!("lvtile{prime:02}.bmp"))));}
        if self.valid {out.sound=Some(format!("T{prime}.aud"));}
        if self.valid && index==6 {
            out.complete=true;self.lower_final=true;
            for row in 0..5 {let name=Kind::Barren.name(Cell::new(3,row));if row==4 {out.cues.push((name.clone(),2403,None));}out.cues.push((name,2400,Some("lvtile01.bmp".into())));}
            for col in 0..7 {for row in 0..5 {let c=Cell::new(col,row);if col!=3 && c!=Cell::new(4,2) {out.cues.push((Kind::Barren.name(c),2401,None));}}}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grid(kind:Kind)->Grid { Grid::read(kind,|name| {let bytes=name.as_bytes();let a=(bytes[4]-b'0') as f32;let b=(bytes[5]-b'0') as f32;
        Some(match kind {Kind::Swamp=>[50000.+2000.*a,92600.+2000.*b,a+b],Kind::Barren=>[86640.+2000.*b,151943.+2000.*a,-700.]})}).0 }
    #[test]
    fn secondary_tiles_grouped_geometry_and_choices() {
        for (kind,x0,x1,y0,y1) in [(Kind::Swamp,45000.,65000.,91600.,105600.),(Kind::Barren,85640.,99640.,150943.,160943.)] {
            assert_eq!(kind.where_is([x0,y0,0.]),Where::South);assert_eq!(kind.where_is([x1,y1,0.]),Where::North);
            assert_eq!(kind.where_is([x0,y0+1.,0.]),Where::West);assert_eq!(kind.where_is([x1,y1-1.,0.]),Where::East);
            let g=grid(kind);let c=Cell::new(2,2);let p=g.place(c).unwrap();assert_eq!(kind.where_is([p[0]+1000.,p[1],99999.]),Where::Cell(Cell::new(3,2)));
            for (back,diag,m) in [(false,false,"tilejump1"),(false,true,"tilejump2"),(true,false,"tilebackjump"),(true,true,"tilebackjump2")] {assert_eq!(Choice {cell:None,point:p,diagonal:diag,exit:false,ordinary:false}.motion(back),m);}
        }
        let s=grid(Kind::Swamp);assert_eq!(s.kind.name(Cell::new(2,3)),"tile23");assert!(s.place(Cell::new(-1,2)).is_none());
        assert_eq!(s.choose([50000.,96600.,0.],-std::f32::consts::FRAC_PI_2,false,false,false).unwrap().cell,Some(Cell::new(-1,3)));
        assert_eq!(s.choose([48000.,98600.,0.],-std::f32::consts::FRAC_PI_2,false,false,false).unwrap().cell,Some(Cell::new(-2,3)));
        assert!(s.choose([44000.,98600.,0.],0.,false,false,false).unwrap().ordinary);
        assert!(s.choose([50000.,91000.,0.],0.,false,false,false).is_none());
        let b=grid(Kind::Barren);assert_eq!(b.kind.name(Cell::new(2,3)),"tile32");
        let p=b.place(Cell::new(3,3)).unwrap();assert_ne!(b.choose(p,std::f32::consts::PI,false,false,false).unwrap().cell,Some(Cell::new(3,4)));
        assert_eq!(b.choose(p,std::f32::consts::PI,false,false,true).unwrap().cell,Some(Cell::new(3,4)));
        assert_eq!(b.choose(p,std::f32::consts::PI,false,true,false).unwrap().cell,Some(Cell::new(3,4)));
        let p=[92640.,149943.,0.];let target=b.place(Cell::new(2,0)).unwrap();let heading=(target[0]-p[0]).atan2(-(target[1]-p[1]));
        assert_eq!(b.choose(p,heading,false,true,false).unwrap().cell,Some(Cell::new(2,0)),"outside-south ignores solved column filter");
        assert!(b.choose([85000.,154000.,0.],0.,false,false,false).is_none());
    }
    #[test]
    fn secondary_tiles_grouped_prime_history_and_bridge() {
        let mut sequence=Sequence::default();let wrong=sequence.land(Some(PRIMES[2].0),false);assert!(!sequence.valid && wrong.sound.is_none());assert_eq!(wrong.cues.len(),1);
        for (i,(c,p)) in PRIMES.into_iter().enumerate() {let land=sequence.land(Some(c),false);assert_eq!(land.sound,Some(format!("T{p}.aud")));assert_eq!(land.complete,i==6);
            if i==2 {assert!(land.cues.is_empty(),"repeat prime does not reveal twice");}
            if i==6 {assert_eq!(land.cues.iter().filter(|(_,cue,_)|*cue==2401).count(),29);assert!(!land.cues.iter().any(|(n,c,_)|n=="tile24" && *c==2401));}
            else {let before=sequence.clone();sequence.land(Some(Cell::new(0,0)),false);assert_eq!(sequence,before);}
        }
        assert!(sequence.land(Some(Cell::new(4,2)),true).cues.is_empty());
        assert_eq!(sequence.land(Some(Cell::new(3,2)),true).cues,[("tile24".into(),2401,None)]);
        assert!(sequence.land(None,true).cues.is_empty());
    }
}
