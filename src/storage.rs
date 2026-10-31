    mod fvec{
        use std::marker::PhantomData;
        #[derive(PartialEq)]
        pub struct Idx<T>{
            i:u16,
            m:PhantomData<fn()->T>,
        }
        impl<T> Idx<T>{
            //shall only be used for ROOT and initial focus
            pub const fn new(i:u16)->Self{ Self{i, m:PhantomData} }
        }
        impl<T> Copy for Idx<T>{}
        impl<T> Clone for Idx<T>{ fn clone(&self)-> Self{ *self } }

        pub struct Fvec<T>{
            used:Vec<Option<T>>,
            free:Vec<Idx<T>>,
        }
        use std::ops::{Index, IndexMut};
        impl<T> Index<Idx<T>> for Fvec<T>{
            type Output = T;
            fn index(&self, i:Idx<T>)->&T{
                self.used[i.i as usize].as_ref().unwrap()
            }
        }
        impl<T> IndexMut<Idx<T>> for Fvec<T> {
            fn index_mut(&mut self, i:Idx<T>)->&mut T{
                self.used[i.i as usize].as_mut().unwrap()
            }
        }
        impl<T> Fvec<T>{
            pub fn new()->Self{Self{used:vec![], free:vec![]}}
            pub fn insert(&mut self, t:T)->Idx<T>{
                if let Some(i) = self.free.pop(){
                    self.used[i.i as usize] = Some(t);
                    i
                }else{
                    let i = self.used.len();
                    self.used.push(Some(t));
                    Idx { i:i as u16, m: PhantomData }
                }
            }
            pub fn remove(&mut self, i:Idx<T>)->T{
                self.free.push(i);
                self.used[i.i as usize].take().unwrap()
            }
        }
    }

pub mod widgets{
    use crate::Widget;
    pub struct Widgets<C, E>{
        w:Vec<Option<Box<dyn Widget<C, E>>>>,
        free:Vec<Widx>,
    }
    impl<C, E> Widgets<C, E>{
        pub fn new()->Self{Self{w:vec![], free:vec![]}}
        pub fn insert(&mut self, w:Box<dyn Widget<C, E>>)->Widx{
            if let Some(f) = self.free.pop(){
                self.w[f.0 as usize] = Some(w);
                f
            }else{
                let i = self.w.len();
                self.w.push(Some(w));
                Widx(i as u16)
            }
        }
        pub fn remove(&mut self, w:Widx)->Box<dyn Widget<C, E>>{
            self.free.push(w);
            self.w[w.0 as usize].take().unwrap()
        }
        pub fn replace(&mut self, w:Widx, widget:Box<dyn Widget<C, E>>)->Box<dyn Widget<C, E>>{
            std::mem::replace(&mut self.w[w.0 as usize].as_mut().unwrap(), widget)
        }
    }
#[derive(PartialEq, Clone, Copy)]
    pub struct Widx(u16);
    impl<C, E> std::ops::Index<Widx> for Widgets<C, E>{
        type Output = Box<dyn Widget<C, E>>;
        fn index(&self, i:Widx)->&Self::Output{
            self.w[i.0 as usize].as_ref().expect("index on freed widget")
        }
    }
    impl<C, E> std::ops::IndexMut<Widx> for Widgets<C, E>{
        fn index_mut(&mut self, i:Widx)->&mut Self::Output{
            self.w[i.0 as usize].as_mut().expect("index mut on freed widget")
        }
    }

}

pub mod nodes{
    use super::fvec::*;
    use super::widgets::*;
    pub const FOCUS: Lidx = Idx::new(0);
    pub const ROOT: Sidx = Idx::new(0);
    use crate::layout::{
        Leaf as L, Split as S,
        Rect,
        Bounds, Bound,
        Direction,
    };
    use super::super::Widget as W;

    pub struct Nodes{
        pub l:Fvec<L>,
        pub s:Fvec<S>,
    }
    pub type Sidx = Idx<S>;
    pub type Lidx = Idx<L>;
    use std::ops::{Index, IndexMut};
    impl Index<Sidx> for Nodes{
        type Output = S;
        fn index(&self, i:Sidx)->&S{
            &self.s[i]
        }
    }
    impl IndexMut<Sidx> for Nodes{
        fn index_mut(&mut self, i:Sidx)->&mut S{
            &mut self.s[i]
        }
    }
    impl Index<Lidx> for Nodes{
        type Output = L;
        fn index(&self, i:Lidx)->&L{
            &self.l[i]
        }
    }
    impl IndexMut<Lidx> for Nodes{
        fn index_mut(&mut self, i:Lidx)->&mut L{
            &mut self.l[i]
        }
    }

#[derive(Clone, Copy, PartialEq)]
    pub enum Nidx{
        S(Sidx),
        L(Lidx),
    }
    impl From<Sidx> for Nidx{
        fn from(s: Sidx) -> Self {
            Self::S(s)
        }
    }
    impl From<Lidx> for Nidx{
        fn from(value: Lidx) -> Self {
            Self::L(value)
        }
    }

    impl Nidx{
        pub fn r(self, n:&Nodes)->Rect{
            match self{
                Self::S(s)=>n[s].r,
                Self::L(l)=>n[l].r,
            }
        }
        pub fn r_mut(self, n:&mut Nodes)->&mut Rect{
            match self{
                Self::S(s)=>&mut n[s].r,
                Self::L(l)=>&mut n[l].r,
            }
        }
        pub fn bounds(self, n:&Nodes)->Bounds{
            match self{
                Self::S(s)=>n[s].b,
                Self::L(l)=>n[l].c.into(),
            }
        }
    }
    impl Nodes{
        pub fn new<C, E>(w:u16, h:u16, widgets:&mut Widgets<C, E>, direction:Direction, widget:impl W<C,E> + 'static)->Self{
            let (mut l, mut s) = (Fvec::new(), Fvec::new());
            let wi = widgets.insert(Box::new(widget));
            let li = l.insert(L{parent:ROOT, r:Rect::ZERO, w:wi, 
                c:Bounds{
                    min_w:Bound::Absolute(0),
                    min_h:Bound::Absolute(0),
                    max_w:Bound::Relative(1.0),
                    max_h:Bound::Relative(1.0),
                }.try_into().unwrap()
            });
            s.insert(S{parent:None, r:Rect{x:0, y:0, w, h}, direction, children:vec![li.into()], focus:0, 
                b:Bounds{
                    min_w:Bound::Absolute(w),
                    max_w:Bound::Absolute(w),
                    min_h:Bound::Absolute(h),
                    max_h:Bound::Absolute(h),
                }
            });
            let mut s = Self { l, s };
            s.recalc(ROOT);
            s
        }
    }

    impl L{
        pub fn add<C, E>(n: &mut Nodes, w:&mut Widgets<C, E>, parent:Sidx, k:Box<dyn W<C, E>>, c:Bounds)->Lidx{
            let w = w.insert(k);
            let lidx = n.l.insert(Self{parent, r:Rect::ZERO, w, c:c.try_into().unwrap()});
            n[parent].children.push(lidx.into());
            lidx
        }
        pub fn append<C, E>(n:&mut Nodes, w:&mut Widgets<C, E>, f:Lidx, k:Box<dyn W<C, E>>, c:Bounds)->Lidx{
            let p = n[f].parent;
            let l = L::add(n, w, p, k, c);
            n.recalc(p);
            l
        }
        pub fn split<C, E>(
            n:&mut Nodes,
            w:&mut Widgets<C, E>,
            d:Direction,
            f:Lidx,
            k:Box<dyn W<C, E>>,
            split_c:Bounds,
            leaf_c:Bounds
        )->(Lidx, Sidx){
            let p = n[f].parent;
            let i = n[p].children.iter()
                .position(|c| *c == f.into())
                .unwrap();
            let l = match n[p].children[i] {Nidx::L(l) =>l, _ => unreachable!()};
            let (new_l, s) = S::add(n, w, d, p, k, split_c, leaf_c);
            n[l].parent = s;
            n[p].children.swap_remove(i);
            n[s].children.insert(0, l.into());
            n.recalc(p);
            (new_l, s)
        }
    }
    impl S{
        pub fn add<C, E>(
            n:&mut Nodes,
            ws:&mut Widgets<C, E>,
            direction:Direction,
            p:Sidx,
            w:Box<dyn W<C, E>>,
            split_c:Bounds,
            leaf_c:Bounds
        )->(Lidx, Sidx){
            let s = n.s.insert(S{parent:p.into(), focus:0, direction, r:Rect::ZERO, children:vec![], b:split_c});
            n[p].children.push(s.into());
            let w = ws.insert(w);
            let lidx = n.l.insert(L{parent:s, r:Rect::ZERO, w, c:leaf_c.try_into().unwrap()});
            n[s].children.push(lidx.into());
            n.recalc(p);
            (lidx, s)
        }
    }
}
