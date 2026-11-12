pub enum FocusDirection { Left, Right, Up, Down, }
use std::process::Child;

use crate::{Background, Cell, ScreenBuffer, Widget, nodes::Nodes, storage::nodes::{Lidx, Nidx, ROOT, Sidx}, widgets::Widgets
};

pub mod leaf{
    use super::Rect;
    use crate::storage::{nodes::Sidx, widgets::Widx};
    #[derive(Clone, Copy, PartialEq)]
    //subset of split bounds since leaves cant have children therefore shouldnt be able to have
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
    pub struct Leaf{
        pub parent: Sidx,
        pub w: Widx,
        pub r: Rect,
    }
    impl Default for Bounds{
        fn default() -> Self {
            Self{
                min_w:Bound::Absolute(0),
                min_h:Bound::Absolute(0),
                max_w:Bound::Relative(1.0),
                max_h:Bound::Relative(1.0),
            }
        }
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

pub struct Split {
    pub parent: Option<Sidx>,
    pub focus: u16,
    pub children: Vec<Nidx>, //children cannot be left empty hard invariant
    pub direction: Direction,
    pub r: Rect,
    pub b: Bounds,
    pub background: fn(Rect, &mut ScreenBuffer),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bound{
    Absolute(u16),
    Relative(f32),//relative to parent
    Fit, //same direction: sum of children minimums, diff direction: max children minimum 
}
impl Bound{
    fn get(self, width:bool, curr:Rect, parent_r:Rect, parent_b:Bounds)->u16{
        match self{
            Self::Absolute(x) => x,
            Self::Fit => if width{curr.w}else{curr.h}
            Self::Relative(x) => {
                let b = parent_b;
                if [b.min_w, b.max_w, b.min_h, b.max_h].iter().any(|x| *x == Bound::Fit){
                    if width{curr.w}else{curr.h}
                }else{
                    if width{(parent_r.w as f32 * x) as u16 }else{(parent_r.h as f32 * x) as u16}
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Bounds{
    pub max_w: Bound,
    pub min_w: Bound,
    pub max_h: Bound,
    pub min_h: Bound,
}
impl Default for Bounds{
    fn default() -> Self {
        Self{
            min_w:Bound::Absolute(0),
            min_h:Bound::Absolute(0),
            max_w:Bound::Relative(1.0),
            max_h:Bound::Relative(1.0),
        }
    }
}
impl Bounds{
    fn max_w(self, curr:Rect,  parent_r:Rect, parent_b:Bounds)->u16{
        self.max_w.get(true, curr, parent_r, parent_b)
    }
    fn min_w(self, curr:Rect, parent_r:Rect, parent_b:Bounds)->u16{
        self.min_w.get(true, curr, parent_r, parent_b)
    }
    fn max_h(self, curr:Rect, parent_r:Rect, parent_b:Bounds)->u16{
        self.max_h.get(false, curr, parent_r, parent_b)
    }
    fn min_h(self, curr:Rect, parent_r:Rect, parent_b:Bounds)->u16{
        self.min_h.get(false, curr, parent_r, parent_b)
    }
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
        (self.y + self.h).saturating_sub(1)
    }
    pub fn left(self) -> u16 {
        self.x
    }
    pub fn right(self) -> u16 {
        (self.x + self.w).saturating_sub(1)
    }
    pub fn fill(self, c:Cell, screen:&mut ScreenBuffer){
        for x in self.x..=self.right(){
            for y in self.y..=self.bottom(){
                screen.set_cell(self, x, y, c);
            }
        }
    }
}

impl Nodes{
    pub fn resize<C, E>(&mut self, ws:&Widgets<C, E>, w: u16, h: u16) {
        self[ROOT].r = Rect { w, h, x: 0, y: 0 };
        self.recalc(ws, ROOT);
    }
    pub fn propegate_fit<C, E>(&mut self, ws:&Widgets<C, E>, i:Sidx)->Sidx{
        let (c, d) = {let Split{direction:d, children:c, ..} = &mut self[i]; (std::mem::take(c), *d)};
        let (mut w, mut h) = (0, 0);
        for c in c.iter() {
            let b = c.bounds(self, ws);
            let c = c.r(self);
            match d{
                Direction::Horizontal => {
                    match b.min_w{
                        Bound::Absolute(a) => w += a,
                        Bound::Fit         => w += c.w,
                        Bound::Relative(_) => {}
                    }
                    match b.min_h{
                        Bound::Absolute(a) => h = h.max(a),
                        Bound::Fit         => h = h.max(c.h),
                        Bound::Relative(_) => {}
                    }
                }
                Direction::Vertical => {
                    match b.min_h{
                        Bound::Absolute(a) => h += a,
                        Bound::Fit         => h += c.h,
                        Bound::Relative(_) => {} 
                    }
                    match b.min_w{
                        Bound::Absolute(a) => w = w.max(a),
                        Bound::Fit         => w = w.max(c.w),
                        Bound::Relative(_) => {},
                    }
                }
            }
        }
        let (prev_w, prev_h) = (w, h);
        for c in c.iter(){
            let b = c.bounds(self, ws);
            let r = c.r_mut(self);
            match d{
                Direction::Horizontal => {
                    if let Bound::Relative(x) = b.min_w{
                        r.w = ((prev_w as f32) * x) as u16;
                        w += r.w;
                    }
                    if let Bound::Relative(x) = b.min_h{
                        r.h = ((prev_h as f32) * x) as u16;
                        h = r.h.max(h);
                    }
                }
                Direction::Vertical => {
                    if let Bound::Relative(x) = b.min_h{
                        r.h = ((prev_h as f32) * x) as u16;
                        h += r.h;
                    }
                    if let Bound::Relative(x) = b.min_w{
                        r.w = ((prev_w as f32) * x) as u16;
                        w = r.w.max(w);
                    }
                }
            }
        }

        if let Bound::Fit = self[i].b.min_w{
            self[i].r.w = w;
        }
        if let Bound::Fit = self[i].b.max_w{
            self[i].r.w = w;
        }
        if let Bound::Fit = self[i].b.min_h{
            self[i].r.h = h;
        }
        if let Bound::Fit = self[i].b.max_h{
            self[i].r.h = h;
        }
        self[i].children = c;
        if let Some(p) = self[i].parent{
            self.propegate_fit(ws, p)
        }else{
            i
        }
    }
    fn distrobute<C, E>(&mut self, ws:&Widgets<C, E>, i:Sidx){
        let (children, d, parent_r, parent_b) = {
            let Split{children:c, direction:d, r, b, ..} = &mut self[i];
            (std::mem::take(c), *d, *r, *b)
        };
        let mut left = match d{
            Direction::Vertical => parent_r.h,
            Direction::Horizontal => parent_r.w,
        };
        for c in children.iter(){
            let b = c.bounds(self, ws);
            let r = c.r_mut(self);
            match d{
                Direction::Vertical => {
                    left -= {let x = b.min_h(*r, parent_r, parent_b); r.h = x; x};
                    let max = b.max_w(*r, parent_r, parent_b);
                    r.w = max.min(parent_r.w);
                }
                Direction::Horizontal => {
                    left -= {let x = b.min_w(*r, parent_r, parent_b); r.w = x; x};
                    let max = b.max_h(*r, parent_r, parent_b);
                    c.r_mut(self).h = max.min(parent_r.h);
                }
            }
        }
        let mut resize: Vec<_> = children.iter().map(|x| Some(*x)).collect();
        while left > 0 && !resize.is_empty(){
            let len = resize.iter().filter(|x| x.is_some()).count() as u16;
            if len == 0{break};
            let per = left / len;
            let mut rem = left % len;
            let mut used = 0;
            for child in resize.iter_mut(){
                let Some(c) = child else {continue};
                let b = c.bounds(self, ws);
                let c = c.r_mut(self);
                let (size, max) = match d{
                    Direction::Horizontal => (c.w, b.max_w(*c, parent_r, parent_b)),
                    Direction::Vertical   => (c.h, b.max_h(*c, parent_r, parent_b)),
                };
                let add = 
                    (per + if rem > 0{rem -= 1; 1}else{0})
                    .min(max.saturating_sub(size));
                match d{
                    Direction::Vertical   => c.h += add,
                    Direction::Horizontal => c.w += add,
                    }
                used += add;
                if size + add >= max{
                    *child = None;
                }
            }
            left = left.saturating_sub(used);
            if used == 0 {break}
        }
        for c in children.iter(){
            if let Nidx::S(c) = c{
                self.distrobute(ws, *c);
            }
        }
        self[i].children = children;
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
    pub fn recalc<C, E>(&mut self, ws:&Widgets<C, E>, i: Sidx) {
        let i = self.propegate_fit(ws, i);
        self.distrobute(ws, i);
        self.recalc_pos(i);
    }
}

impl Nodes{
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
                    self[curr].focus = i as u16;
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
                    let f = self[s].focus.min((children.len() as u16) - 1);
                    target = children[f as usize];
                }
            }
        }
    }
    pub fn change_widget<C, E>(&mut self, f:Lidx, widgets: &mut Widgets<C, E>, w:impl Widget<C, E> + 'static){
        widgets.replace(self[f].w, Box::new(w));
    }
    pub fn split<C, E>(
        &mut self,
        ws:&mut Widgets<C, E>,
        d:Direction,
        f:Lidx,
        w:Box<dyn Widget<C, E>>,
        background:Background,
        split_b:Bounds
    ){
        Leaf::split(self, ws, d, f, w, background, split_b);
    }
    pub fn close<C, E>(&mut self, ws:&mut Widgets<C, E>, f:&mut Lidx)->Result<(), ()>{
        let last = {
            let p = self[*f].parent;
            let Split {focus, children: c, ..} = &mut self[p];
            let pos = c.iter().position(|x| *x == (*f).into()).unwrap();
            c.remove(pos);
            *focus = if *focus > pos as u16{
                *focus - 1
            }else{
                (*focus).min((c.len().saturating_sub(1)) as u16)
            };
            self.l.remove(*f);
            prune(self, p)
        };
        let last = self.propegate_fit(ws, last);
        self.recalc(ws, last);
        return fix_focus(self, f, last);
        fn prune(ns:&mut Nodes, p:Sidx)->Sidx{
            let pp = match ns[p].parent{ Some(x) => x, _ => return p };
            match ns[p].children.len(){
                0 => {
                    let Split { focus:f, children: c, ..} = &mut ns[pp];
                    let pos = c.iter().position(|x| *x == p.into()).unwrap();
                    c.remove(pos);
                    *f = if *f > pos as u16{
                        *f - 1
                    } else {
                        (*f).min((c.len().saturating_sub(1)) as u16)
                    };
                    ns.s.remove(p);
                    prune(ns, pp)
                }
                1 => {
                    let c = ns[p].children[0];
                    match c{
                        Nidx::S(x) => ns[x].parent = pp.into(),
                        Nidx::L(x) => ns[x].parent = pp,
                    }
                    ns[pp].children.retain(|x| *x != p.into());
                    ns.s.remove(p);
                    ns[pp].children.push(c);
                    prune(ns, pp)
                }
                _ => {
                    return p
                }
            }
        }
        fn fix_focus(ns:&mut Nodes, i:&mut Lidx, last:Sidx)->Result<(), ()>{
            let mut last = last.into();
            while let Nidx::S(x) = last{
                last = *ns[x].children.get(ns[x].focus as usize).ok_or(())?
            }
            let Nidx::L(s) = last else {unreachable!()};
            *i = s;
            Ok(())
        }
    }
}
