pub enum FocusDirection { Left, Right, Up, Down, }
use crate::{nodes::Nodes, storage::{
    nodes::{Nidx, Sidx, Lidx, ROOT}},
    widgets::Widgets,
};

mod leaf{
    use super::Rect;
    use crate::storage::{nodes::Sidx, widgets::Widx};
    #[derive(Clone, Copy, PartialEq)]
    //subset of split constraints since leaves cant have children therefore shouldnt be able to have
    //fit sizing
    pub enum Bound{
        Absolute(u16),
        Relative(f32),
    }
    #[derive(Clone, Copy, PartialEq)]
    pub struct Bounds{
        max_w:Bound,
        min_w:Bound,
        max_h:Bound,
        min_h:Bound,
    }
    #[derive(PartialEq)]
    pub struct Leaf{
        pub parent: Sidx,
        pub w: Widx,
        pub r: Rect,
        pub c: Bounds,
    }
    impl TryFrom<super::Bounds> for Bounds{
        type Error = &'static str;
        fn try_from(c:super::Bounds)->Result<Self, Self::Error>{
            Ok(Self{
                max_w:c.max_w.try_into()?,
                min_w:c.min_w.try_into()?,
                max_h:c.max_h.try_into()?,
                min_h:c.min_h.try_into()?,
            })
        }
    }
    impl TryFrom<super::Bound> for Bound{
        type Error = &'static str;
        fn try_from(c:super::Bound)->Result<Self, Self::Error>{
            match c{
                super::Bound::Absolute(a) => Ok(Self::Absolute(a)),
                super::Bound::Relative(r) => Ok(Self::Relative(r)),
                _ => Err("leaf cannot have children therefore cant have fit")
            }
        }
    }
    impl From<Bounds> for super::Bounds{
        fn from(c:Bounds)->Self{
            Self {
                max_w: c.max_w.into(),
                min_w: c.min_w.into(),
                max_h: c.max_h.into(),
                min_h: c.min_h.into()
            }
        }
    }
    impl From<Bound> for super::Bound{
        fn from(c:Bound)->Self{
            match c{
                Bound::Relative(r)=>Self::Relative(r),
                Bound::Absolute(a)=>Self::Absolute(a),
            }
        }
    }
}
pub use leaf::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(PartialEq)]
pub struct Split {
    pub parent: Option<Sidx>,
    pub focus: u8,
    pub children: Vec<Nidx>, //children cannot be left empty hard invariant
    pub direction: Direction,
    pub r: Rect,
    pub b: Bounds,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bound{
    Absolute(u16),
    Relative(f32),//relative to parent
    Fit,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Bounds{
    pub max_w: Bound,
    pub min_w: Bound,
    pub max_h: Bound,
    pub min_h: Bound,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}
impl Rect {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };
    pub fn top(self) -> u16 {
        self.y
    }
    pub fn bottom(self) -> u16 {
        self.y + self.h - 1
    }
    pub fn left(self) -> u16 {
        self.x
    }
    pub fn right(self) -> u16 {
        self.x + self.w - 1
    }
}

impl Nodes{
        fn recalc_size(&mut self, i:Sidx)->Sidx{
            let (d, p_r, b) = {let Split{direction:d, r, b, ..} = self[i]; (d, r, b)};
            let min_h = |i: Nidx, n:&Nodes|->u16{
                match i.bounds(n).min_h{
                    Bound::Fit => i.r(n).h,
                    Bound::Absolute(a) => a,
                    Bound::Relative(a) => (a * p_r.h as f32) as u16,
                }
            };
            let max_h = |i: Nidx, n:&Nodes|->u16{
                match i.bounds(n).max_h{
                    Bound::Fit => i.r(n).h,
                    Bound::Absolute(a) => a,
                    Bound::Relative(a) => (a * p_r.h as f32) as u16,
                }
            };
            let min_w = |i: Nidx, n:&Nodes|->u16{
                match i.bounds(n).min_w{
                    Bound::Fit => i.r(n).w,
                    Bound::Absolute(a) => a,
                    Bound::Relative(a) => (a * p_r.w as f32) as u16,
                }
            };
            let max_w = |i: Nidx, n:&Nodes|->u16{
                match i.bounds(n).max_w{
                    Bound::Fit => i.r(n).w,
                    Bound::Absolute(a) => a,
                    Bound::Relative(a) => (a * p_r.w as f32) as u16,
                }
            };
            let mut left = match d{
                Direction::Vertical   => p_r.h, 
                Direction::Horizontal => p_r.w,
            };
            let mut resize: Vec<_> = self[i].children.iter().map(|x| Some(*x)).collect();
            let (mut w, mut h) = (0, 0);
            for child in resize.iter(){
                let child = child.unwrap();
                let (min_h, max_h, min_w, max_w) = (min_h(child, self), max_h(child, self), min_w(child, self), max_w(child, self));
                let c_r = child.r_mut(self);
                        match d{
                            Direction::Horizontal => {
                                left -= min_w;
                                w += min_w;
                                h = h.max(min_h);
                                c_r.w = min_w;
                                c_r.h = max_h.min(p_r.h);
                            }
                            Direction::Vertical  => {
                                left -= min_h;
                                h += min_h;
                                w = w.max(min_w);
                                c_r.h = min_h;
                                c_r.w = max_w.min(p_r.w);
                            }
                        }
            }
            if let Bound::Fit = self[i].b.min_w{
                self[i].r.w = w;
            }
            if let Bound::Fit = self[i].b.min_h{
                self[i].r.h = h;
            }
            if let Bound::Fit = self[i].b.max_w{
                self[i].r.w = w;
            }
            if let Bound::Fit = self[i].b.max_h{
                self[i].r.h = h;
            }
            if ![b.min_w, b.max_w, b.min_h, b.max_h].iter().any(|x| matches!(x, Bound::Fit)){
                while left > 0 && !resize.is_empty(){
                    let len = resize.iter().filter(|x| x.is_some()).count() as u16;
                    if len == 0{break};
                    let per = left / len;
                    let rem = left % len;
                    let mut used = 0;
                    for (i, child) in resize.iter_mut().enumerate(){
                        let Some(c) = child else {continue};
                        let (size, max) = match d{
                            Direction::Vertical  => ( c.r(self).h, max_h(*c, self)),
                            Direction::Horizontal=> ( c.r(self).w, max_w(*c, self)),
                        };
                        if max < size{panic!()}
                        let add = (per + u16::from((i as u16) < rem)).min(max.saturating_sub(size));
                        match d{
                            Direction::Horizontal => c.r_mut(self).w += add,
                            Direction::Vertical   => c.r_mut(self).h += add,
                        }
                        used += add;
                        if size + add >= max{
                            *child = None;
                        }
                    }
                    left -= used;
                    if used == 0 {break;}
                    let c = std::mem::take(&mut self[i].children);
                    for c in c.iter(){
                        if let Nidx::S(c) = c{
                            self.recalc_size(*c);
                        }
                    }
                    self[i].children = c;
                }
                i
            }else{
                if let Some(p) = self[i].parent{
                    self.recalc_size(p)
                }else{
                    i
                }
            }
        }
        fn recalc_pos(&mut self, i:Sidx){
            let (c, d, mut r) = {let Split{children:c, direction:d, r, ..} = &mut self[i]; (std::mem::take(c), *d, *r)};
            for c in c.iter(){
                c.r_mut(self).x = r.x;
                c.r_mut(self).y = r.y;
                match d{
                    Direction::Horizontal => r.x += c.r(self).w,
                    Direction::Vertical   => r.y += c.r(self).h,
                }
            }
            for c in c.iter(){
                if let Nidx::S(c) = c{
                    self.recalc_pos(*c);
                }
            }
            self[i].children = c;
        }
        pub fn recalc(&mut self, i: Sidx) {
            let i = self.recalc_size(i);
            self.recalc_pos(i);
        }
        pub fn close<C, E>(&mut self, w:&mut Widgets<C, E>, f:&mut Lidx){
            let p = self[*f].parent;
            if self[p].parent.is_none() && self[p].children.len() <= 1 {
                return;
            };
            self[p].children.retain(|c| *c != Nidx::L(*f));
            self[p].focus = if self[p].children.len() == 0 { 0 }else{
                self[p].focus.min((self[p].children.len() - 1) as u8)
            };
            w.remove(self[*f].w);
            let s = self.prune(*f);
            self.l.remove(*f);
            self.recalc(s);

            let mut s = s.into();
            loop{
                s = match s {
                    Nidx::S(s) => self[s].children[self[s].focus as usize],
                    _ => break,
                };
            }
            let Nidx::L(s) = s else {unreachable!()};
            *f = s;
        }
        fn prune(&mut self, i: Lidx) -> Sidx{
            fn prune(n: &mut Nodes, i: Nidx) -> Sidx{
                let p = match i {
                    Nidx::L(i) => n[i].parent,
                    Nidx::S(i) => match n[i].parent {
                        Some(s) => s,
                        _ => i,
                    },
                };
                let Some(pp) = n[p].parent else { return p };
                match n[p].children.len() {
                    1 => {
                        let x = n[p].children[0];
                        let i = n[pp].children.iter()
                            .position(|c| *c == Nidx::S(p))
                            .unwrap();
                        n[pp].children[i] = x;
                        match x {
                            Nidx::S(i) => n[i].parent = Some(pp),
                            Nidx::L(i) => n[i].parent = pp,
                        }
                        n.s.remove(p);
                        prune(n, Nidx::S(pp))
                    }
                    0 => {
                        n[pp].children.retain(|c| *c != Nidx::S(p));
                        n[pp].focus = if n[pp].children.len() == 0 { 0 } else {
                            u8::min(n[pp].focus, (n[pp].children.len() - 1) as u8)
                        };
                        n.s.remove(p);
                        prune(n, Nidx::S(pp))
                    }
                    _ => {
                        n.recalc(p);
                        return p;
                    }
                }
            }
            prune(self, i.into())
        }
        pub fn resize(&mut self, w: u16, h: u16) {
            self[ROOT].r = Rect { w, h, x: 0, y: 0 };
            self.recalc(ROOT);
        }
        pub fn focus(&mut self, f: &mut Lidx, d: FocusDirection){
            let r = self[*f].r;
            let mut curr = self[*f].parent;

            let target = 'search: loop {
                let (parent, len) = {
                    let s = &self[curr];
                    (s.parent, s.children.len())
                };

                let forward = matches!(d, FocusDirection::Right | FocusDirection::Down);
                for j in 0..len {
                    let i = if forward { j } else { len - 1 - j };
                    let c = self[curr].children[i];
                    if matches!(c, Nidx::L(x) if x == *f) {
                        continue;
                    }
                    let cr = match c {
                        Nidx::L(x) => self[x].r,
                        Nidx::S(x) => self[x].r,
                    };
                    let found = match d {
                        FocusDirection::Left => cr.x + cr.w <= r.x,
                        FocusDirection::Right => cr.x >= r.x + r.w,
                        FocusDirection::Up => cr.y + cr.h <= r.y,
                        FocusDirection::Down => cr.y >= r.y + r.h,
                    };
                    if found {
                        self[curr].focus = i as u8;
                        break 'search c;
                    }
                }
                match parent {
                    Some(p) => curr = p,
                    None => return,
                }
            };
            let mut target = target;
            loop {
                match target {
                    Nidx::L(l) => {
                        *f = l;
                        return;
                    }
                    Nidx::S(s) => {
                        let children = &self[s].children;
                        let f = self[s].focus.min((children.len() as u8) - 1);
                        target = children[f as usize];
                    }
                }
            }
        }
}
