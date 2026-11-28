pub enum FocusDirection { Left, Right, Up, Down, }
use crate::{Background, ScreenBuffer, Widget, nodes::Nodes, storage::nodes::{Lidx, Nidx, Sidx, Pidx}, widgets::Widgets
};

pub mod leaf{
    use super::{Rect, Parent};
    use crate::storage::{widgets::Widx};
    #[derive(Clone, Copy, PartialEq)]
    //subset of split bounds since leaves cant have children therefore shouldnt be able to have
    //fit sizing
    pub enum Bound{
        Absolute(u16),
        Relative(f32),
    }
    #[derive(Clone, Copy, PartialEq)]
    pub struct Bounds{
        pub max_w:Bound,
        pub min_w:Bound,
        pub max_h:Bound,
        pub min_h:Bound,
    }
    pub struct Leaf{
        pub parent: Parent,
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
pub enum Direction{
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy)]
pub enum Parent{
    S(Sidx),
    P(Pidx),
    R,
}
impl From<Sidx> for Parent{
    fn from(value: Sidx) -> Self {
        Self::S(value)
    }
}
impl From<Pidx> for Parent{
    fn from(value: Pidx) -> Self {
        Self::P(value)
    }
}
impl Parent{
    pub fn children(self, ns:&Nodes)->&Vec<Nidx>{
        match self{
            Self::R => &ns.root.children,
            Self::P(x) => &ns[x].children,
            Self::S(x) => &ns[x].children,
        }
    }
    pub fn mut_children(self, ns:&mut Nodes)->&mut Vec<Nidx>{
        match self{
            Self::R => &mut ns.root.children,
            Self::P(x) => &mut ns[x].children,
            Self::S(x) => &mut ns[x].children,
        }
    }
    fn focus(self, ns:&Nodes)->u16{
        match self{
            Self::R => ns.root.focus,
            Self::P(x) => ns[x].focus,
            Self::S(x) => ns[x].focus,
        }
    }
    fn mut_focus(self, ns:&mut Nodes)->&mut u16{
        match self{
            Self::R => &mut ns.root.focus,
            Self::P(x) => &mut ns[x].focus,
            Self::S(x) => &mut ns[x].focus,
        }
    }
    fn parent(self, ns:&Nodes)->Option<Self>{
        match self{
            Self::R => None,
            Self::P(x) => ns[x].parent.into(),
            Self::S(x) => ns[x].parent.into(),
        }
    }
}
pub struct Root{
    pub children:Vec<Nidx>,
    pub direction: Direction,
    pub background:Background,
    pub focus:u16,
    pub w:u16, pub h:u16,
}
impl Root{
    const fn b(ns:&Nodes)->Bounds{
        let w = ns.root.w;
        let h = ns.root.h;
        Bounds {
            max_w: Bound::Absolute(w),
            min_w: Bound::Absolute(w),
            max_h: Bound::Absolute(h),
            min_h: Bound::Absolute(h) 
        }
    }
    pub const fn r(ns:&Nodes)->Rect{
        Rect { x: 0, y: 0, w:ns.root.w, h:ns.root.h}
    }
}
pub struct Split {
    pub parent: Parent,
    pub children: Vec<Nidx>, //children cannot be left empty hard invariant
    pub r: Rect,
    pub b: Bounds,
    pub background: Background,
    pub focus: u16,
    pub direction: Direction,
}
pub struct Plane{
    pub parent: Parent,
    pub children: Vec<Nidx>,
    pub r:Rect,
    pub b:Bounds,
    pub background:Background,
    pub align:Alignments,
    pub anchor:Anchors,
    pub focus: u16,
    pub z:u16,
    pub direction: Direction,
}
pub struct Alignments{
    pub x:Alignment,
    pub y:Alignment,
}
pub struct Anchors{
    pub x:Anchor,
    pub y:Anchor,
}
pub enum Alignment{
    Start,
    Center,
    End,
}
pub enum Anchor{
    Absolute(u16),
    Offset(u16),
    Relative2Parent(f32),
    Relative2Screen(f32),
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
use crossterm::style::{Attributes, Color};
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
        self.y + self.h
    }
    pub fn left(self) -> u16 {
        self.x
    }
    pub fn right(self) -> u16 {
        self.x + self.w
    }
    pub fn fill(self, c:&str, fg:Color, bg:Color, a:Attributes, screen:&mut ScreenBuffer){
        for x in self.x..self.right(){
            for y in self.y..self.bottom(){
                screen.set(self, x, y, c, fg, bg, a);
            }
        }
    }
}

impl Nodes{
    pub fn resize<C, E>(&mut self, ws:&Widgets<C, E>, w: u16, h: u16) {
        self.root.w = w;
        self.root.h = h;
        let c = std::mem::take(&mut self.root.children);
        for c in &c{
            let p = match *c{
                Nidx::S(x)=>self[x].parent,
                Nidx::L(x)=>self[x].parent,
                Nidx::P(x)=>self[x].parent,
            };
            self.recalc(ws, p);
        }
        self.root.children = c;
    }
    pub fn propegate_fit<C, E>(&mut self, ws:&Widgets<C, E>, i:Parent)->Parent{
        let (c, d, propegate_further) = match i{
                Parent::S(x) => {let Split{direction:d,  children:c, ..} = &mut self[x]; (std::mem::take(c), *d, true)},
                Parent::P(x) => {let Plane{direction:d, children:c, ..}  = &mut self[x]; (std::mem::take(c), *d, false)},
                Parent::R    => return i
            };
        let (mut w, mut h) = (0, 0);
        for c in c.iter().filter(|x| !matches!(x, Nidx::P(_))) {
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
        for c in c.iter().filter(|x| !matches!(x, Nidx::P(_))){
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
        let (b, r) = match i{
                Parent::S(x) => {let Split{b, r, ..} = &mut self[x]; (*b, r)},
                Parent::P(x) => {let Plane{b, r, ..} = &mut self[x]; (*b, r)},
                Parent::R    => unreachable!()
            };
            if let Bound::Fit = b.min_w{
                r.w = w;
            }
            if let Bound::Fit = b.max_w{
               r.w = w;
            }
            if let Bound::Fit = b.max_h{
               r.h = h;
            }
            if let Bound::Fit = b.min_h{
                r.h = h;
            }
            let p = match i{
                Parent::S(x) => {self[x].children = c; self[x].parent},
                Parent::P(x) => {self[x].children = c; self[x].parent},
                Parent::R => unreachable!()
            };
            if propegate_further{
                self.propegate_fit(ws, p)
            }else{
                i
            }
    }
fn distrobute<C, E>(&mut self, ws: &Widgets<C, E>, i: Parent) {
    let (children, direction, parent_r, parent_b) = match i {
        Parent::S(x) => { let Split { children, direction, r, b, .. } = &mut self[x];
            (std::mem::take(children), *direction, *r, *b)
        }
        Parent::P(x) => { let Plane { children, direction, r, b, .. } = &mut self[x];
            (std::mem::take(children), *direction, *r, *b)
        }
        Parent::R => { let Root { children, direction, .. } = &mut self.root;
            ( std::mem::take(children), *direction, Root::r(self), Root::b(self),)
        }
    };
    let mut left = match direction {
        Direction::Vertical => parent_r.h,
        Direction::Horizontal => parent_r.w,
    };
    // Size normal children using their minimum bounds.
    for child in children.iter().filter(|c| !matches!(c, Nidx::P(_))) {
        let b = child.bounds(self, ws);
        let r = child.r_mut(self);
        match direction {
            Direction::Vertical => {
                let size = b.min_h(*r, parent_r, parent_b);
                r.h = size;
                left = left.saturating_sub(size);

                let max = b.max_w(*r, parent_r, parent_b);
                r.w = max.min(parent_r.w);
            }
            Direction::Horizontal => {
                let size = b.min_w(*r, parent_r, parent_b);
                r.w = size;
                left = left.saturating_sub(size);

                let max = b.max_h(*r, parent_r, parent_b);
                r.h = max.min(parent_r.h);
            }
        }
    }
    // Planes do not consume space in the parent's layout.
    // Their fit dimensions have already been calculated by propegate_fit.
    for child in children.iter().filter(|c| matches!(c, Nidx::P(_))) {
        let b = child.bounds(self, ws);
        let r = child.r_mut(self);
        let min_w = b.min_w(*r, parent_r, parent_b);
        let max_w = b.max_w(*r, parent_r, parent_b);
        let min_h = b.min_h(*r, parent_r, parent_b);
        let max_h = b.max_h(*r, parent_r, parent_b);
        r.w = if matches!(b.min_w, Bound::Fit) || matches!(b.max_w, Bound::Fit) {
            r.w.max(min_w).min(max_w)
        } else {
            max_w
        };
        r.h = if matches!(b.min_h, Bound::Fit) || matches!(b.max_h, Bound::Fit) {
            r.h.max(min_h).min(max_h)
        } else {
            max_h
        };
        // A plane cannot extend beyond its parent's dimensions.
        r.w = r.w.min(parent_r.w);
        r.h = r.h.min(parent_r.h);
    }

    // Distribute remaining space among normal children.
    let mut resize: Vec<Option<Nidx>> = children
        .iter()
        .filter(|c| !matches!(c, Nidx::P(_)))
        .map(|c| Some(*c))
        .collect();

    while left > 0 && !resize.is_empty() {
        let len = resize.iter().filter(|c| c.is_some()).count() as u16;
        if len == 0 {
            break;
        }
        let per = left / len;
        let mut rem = left % len;
        let mut used = 0;
        for child in resize.iter_mut() {
            let Some(child) = child else {
                continue;
            };
            let b = child.bounds(self, ws);
            let r = child.r_mut(self);
            let (size, max) = match direction {
                Direction::Horizontal => (
                    r.w,
                    b.max_w(*r, parent_r, parent_b),
                ),
                Direction::Vertical => (
                    r.h,
                    b.max_h(*r, parent_r, parent_b),
                ),
            };
            let add = (per + if rem > 0 {
                rem -= 1;
                1
            } else {
                0
            })
            .min(max.saturating_sub(size));
            match direction {
                Direction::Vertical => r.h += add,
                Direction::Horizontal => r.w += add,
            }
            used += add;
        }
        left = left.saturating_sub(used);

        if used == 0 {
            break;
        }
    }
    for child in children.iter() {
        match child {
            Nidx::S(x) => self.distrobute(ws, Parent::S(*x)),
            Nidx::P(x) => self.distrobute(ws, Parent::P(*x)),
            Nidx::L(_) => {}
        }
    }
    match i {
        Parent::P(x) => self[x].children = children,
        Parent::S(x) => self[x].children = children,
        Parent::R => self.root.children = children,
    }
}
fn recalc_pos(&mut self, i: Parent) {
    let (c, d, parent_r) = match i {
        Parent::S(i) => {
            let Split { children: c, direction: d, r, .. } = &mut self[i];
            (std::mem::take(c), *d, *r)
        }
        Parent::R => {
            let r = Root::r(self);
            let Root{ children: c, direction: d, .. } = &mut self.root;
            (std::mem::take(c), *d, r)
        }
        Parent::P(i) => {
            let Plane { children: c, direction: d, r, .. } = &mut self[i];
            (std::mem::take(c), *d, *r)
        }
    };
    // Use a separate cursor for normal children.
    // Do not mutate parent_r: plane anchors are relative to the
    // original parent rectangle, not the remaining layout space.
    let mut cursor = parent_r;
    for child in c.iter().filter(|x| !matches!(x, Nidx::P(_))) {
        let child_r = child.r(self);
        {
            let r = child.r_mut(self);
            r.x = cursor.x;
            r.y = cursor.y;
        }
        match d {
            Direction::Horizontal => cursor.x += child_r.w,
            Direction::Vertical => cursor.y += child_r.h,
        }
    }
    // Position planes relative to the original parent rectangle.
    for child in c.iter() {
        let Nidx::P(child) = child else { continue };
        let  root_r = Root::r(self);
        let Plane { r: curr_r, align, anchor, .. } = &mut self[*child];
        let (x, r) = match anchor.x{
            Anchor::Offset(x) => (x, parent_r),
            Anchor::Relative2Screen(x) => ((x * parent_r.w as f32) as u16, parent_r),
            Anchor::Absolute(x) => (x, root_r),
            Anchor::Relative2Parent(x) => ((x * root_r.w as f32) as u16, root_r)
        };
        let x = x + r.x;
        let x = x.min(r.x + r.w);
        let (y, r) = match anchor.y{
            Anchor::Offset(y) => (y, parent_r),
            Anchor::Relative2Parent(y) => ((y * parent_r.h as f32) as u16, parent_r),
            Anchor::Absolute(y) => (y, root_r),
            Anchor::Relative2Screen(y) => ((y * root_r.w as f32) as u16, root_r)
        };
        let y = y + y+r.y;
        let y = y.min(r.y + r.h);
        match align.x {
            Alignment::Start => curr_r.x = x,
            Alignment::Center => curr_r.x = x.saturating_sub(curr_r.w / 2),
            Alignment::End => curr_r.x = x.saturating_sub(curr_r.w),
        }
        match align.y {
            Alignment::Start => curr_r.y = y,
            Alignment::Center => curr_r.y = y.saturating_sub(curr_r.h / 2),
            Alignment::End => curr_r.y = y.saturating_sub(curr_r.h),
        }
    }
    for child in c.iter() {
        match *child {
            Nidx::S(child) => self.recalc_pos(child.into()),
            Nidx::P(child) => self.recalc_pos(child.into()),
            Nidx::L(_) => {}
        }
    }
    *i.mut_children(self) = c;
}
    pub fn recalc<C, E>(&mut self, ws:&Widgets<C, E>, i: Parent) {
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
            let (len, parent) = {
                match curr{
                    Parent::R => unreachable!(),
                    Parent::S(x) => (self[x].children.len(), self[x].parent),
                    Parent::P(x) => (self[x].children.len(), self[x].parent),
                }
            };

            let forward = matches!(d, FocusDirection::Right | FocusDirection::Down);
            for j in 0..len {
                let i = if forward { j } else { len - 1 - j };
                let c = curr.children(self)[i];
                if matches!(c, Nidx::L(x) if x == *f) {
                    continue;
                }
                let cr = match c {
                    Nidx::L(x) => self[x].r,
                    Nidx::S(x) => self[x].r,
                    Nidx::P(x) => self[x].r
                };
                let found = match d {
                    FocusDirection::Left => cr.x + cr.w <= r.x,
                    FocusDirection::Right => cr.x >= r.x + r.w,
                    FocusDirection::Up => cr.y + cr.h <= r.y,
                    FocusDirection::Down => cr.y >= r.y + r.h,
                };
                if found {
                    *curr.mut_focus(self) = i as u16;
                    break 'search c;
                }
            }
            match parent {
                Parent::S(x) => curr = x.into(),
                Parent::P(x) => curr = x.into(),
                Parent::R => return,
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
                Nidx::P(x) => {
                    let children = &self[x].children;
                    let f = self[x].focus.min((children.len() as u16) - 1);
                    target = children[f as usize];
                }
            }
        }
    }
    pub fn change_widget<C, E>(&mut self, f:Lidx, ws: &mut Widgets<C, E>, w:impl Widget<C, E> + 'static){
        ws.replace(self[f].w, Box::new(w));
    }
    pub fn add<C, E>(
        &mut self,
        ws:&mut Widgets<C, E>,
        w:Box<dyn Widget<C, E>>,
        p:Parent,
    ){
        Leaf::add(self, ws, p, w);
    }
    pub fn split<C, E>(
        &mut self,
        ws:&mut Widgets<C, E>,
        d:Direction,
        f:Lidx,
        w:Box<dyn Widget<C, E>>,
        background:Background,
        b:Bounds
    ){
        Leaf::split(self, ws, d, f, w, background, b);
    }
    pub fn close<C, E>(&mut self, ws:&mut Widgets<C, E>, f:&mut Lidx)->Result<(), ()>{
        let last = {
            let p = self[*f].parent;
            let (focus, c) = match p{
                Parent::R    => (&mut self.root.focus, &mut self.root.children),
                Parent::S(x) => {let Split{children:c, focus:f, ..} = &mut self[x]; (f, c)}
                Parent::P(x) => {let Plane {children:c, focus:f, ..} = &mut self[x]; (f, c)}
            };
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
        fn prune(ns:&mut Nodes, p:Parent)->Parent{
            let Some(pp) = p.parent(ns) else {return p};
            match p.children(ns).len(){
                0 => {
                    let p = match p{
                        Parent::S(x) => x.into(),
                        Parent::P(x) => x.into(),
                        Parent::R => unreachable!(),
                    };
                    let (f, c) = match pp{
                        Parent::R =>    {let Root {children:c, focus:f, ..} = &mut ns.root; (f, c)}
                        Parent::S(x) => {let Split{children:c, focus:f, ..} = &mut ns[x]; (f, c)}
                        Parent::P(x) => {let Plane{children:c, focus:f, ..} = &mut ns[x]; (f, c)}
                    };
                    let pos = c.iter().position(|x| *x == p).unwrap();
                    c.remove(pos);
                    *f = if *f > pos as u16{
                        *f - 1
                    } else {
                        (*f).min((c.len().saturating_sub(1)) as u16)
                    };
                    match p{
                        Nidx::L(_) => unreachable!(),
                        Nidx::S(x) => {ns.s.remove(x);},
                        Nidx::P(x) => {ns.p.remove(x);},
                    };
                    prune(ns, pp)
                }
                1 => {
                    let c = match p{
                        Parent::S(x) => &ns[x].children,
                        Parent::P(x) => &ns[x].children,
                        Parent::R => unreachable!()
                    }[0];
                    match c{
                        Nidx::S(x) => ns[x].parent = pp.into(),
                        Nidx::L(x) => ns[x].parent = pp,
                        Nidx::P(x) => ns[x].parent = pp,
                    }
                    let p = match p{
                        Parent::S(x) => x.into(),
                        Parent::P(x) => x.into(),
                        Parent::R    => unreachable!(),
                    };
                    pp.mut_children(ns).retain(|x| *x != p);
                    match p{
                        Nidx::P(x) => {ns.p.remove(x);}
                        Nidx::L(x) => {ns.l.remove(x);}
                        Nidx::S(x) => {ns.s.remove(x);}
                    }
                    pp.mut_children(ns).push(c);
                    prune(ns, pp)
                }
                _ => {
                    return p
                }
            }
        }
        fn fix_focus(ns:&mut Nodes, i:&mut Lidx, last:Parent)->Result<(), ()>{

            let mut last = last;
            let mut leaf = None;
            while matches!(last, Parent::R | Parent::S(_) | Parent::P(_)){
                last = match last.children(ns).get(last.focus(ns) as usize).ok_or(())?{
                    Nidx::S(x) => (*x).into(),
                    Nidx::P(x) => (*x).into(),
                    Nidx::L(x) => {leaf = x.into(); break}
                }
            }
            *i = *leaf.unwrap();
            Ok(())
        }
    }
}
