mod fvec{ //freelist vec
    use std::marker::PhantomData;
    pub struct Idx<T>{
        i:u16,
        m:PhantomData<fn()->T>,
        #[cfg(debug_assertions)]
        generation:u16,
    }
    impl<T> Idx<T>{
        //shall only be used for ROOT and initial focus
        pub const fn new(i:u16)->Self{
            Self{
                i, m:PhantomData,
                #[cfg(debug_assertions)]
                generation:0,
            } 
        }
    }
    impl<T> PartialEq for Idx<T>{
        fn eq(&self, other: &Self) -> bool {
            self.i == other.i
            && {
                #[cfg(debug_assertions)]{
                    self.generation == other.generation
                }
                #[cfg(not(debug_assertions))]{
                    true
                }
            }
        }
    }
    impl<T> Eq for Idx<T>{}
    impl<T> Copy for Idx<T>{}
    impl<T> Clone for Idx<T>{ fn clone(&self)-> Self{ *self } }

    struct Slot<T>{
        val:Option<T>,
        #[cfg(debug_assertions)]
        generation:u16,
    }
    pub struct Fvec<T>{
        used:Vec<Slot<T>>,
        free:Vec<u16>,
    }
    use std::ops::{Index, IndexMut};
    impl<T> Index<Idx<T>> for Fvec<T>{
        type Output = T;
        fn index(&self, i:Idx<T>)->&T{
            let slot = &self.used[i.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!(slot.generation, i.generation, "Stale fvec index");
            slot.val.as_ref().unwrap()
        }
    }
    impl<T> IndexMut<Idx<T>> for Fvec<T> {
        fn index_mut(&mut self, i:Idx<T>)->&mut T{
            let slot = &mut self.used[i.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!( slot.generation, i.generation, "Stale fvec index");
            slot.val.as_mut().unwrap()
        }
    }
    impl<T> Fvec<T>{
        pub fn new()->Self{Self{used:vec![], free:vec![]}}
        pub fn insert(&mut self, t:T)->Idx<T>{
            if let Some(i) = self.free.pop(){
                let slot = &mut self.used[i as usize];
                slot.val = Some(t);
                Idx{
                    i,
                    #[cfg(debug_assertions)]
                    generation:slot.generation,
                    m:PhantomData,
                }
            }else{
                let i = self.used.len();
                self.used.push(Slot{
                    val:Some(t),
                    #[cfg(debug_assertions)]
                    generation:0,
                });
                Idx{
                    i:i as u16,
                    m:PhantomData,
                    #[cfg(debug_assertions)]
                    generation:0
                }
            }
        }
        pub fn remove(&mut self, i:Idx<T>)->T{
            let slot = &mut self.used[i.i as usize];
            #[cfg(debug_assertions)]{
                assert_eq!(slot.generation, i.generation, "Stale fvec index");
                slot.generation = slot.generation.wrapping_add(1);
            }
            let val = slot.val.take().unwrap();
            self.free.push(i.i);
            val
        }
    }
}

pub mod widgets{
    use crate::Widget;
#[derive(PartialEq, Clone, Copy)]
    pub struct Widx{
        i:u16,
        #[cfg(debug_assertions)]
        generation:u16,
    }
    struct Slot<C, E>{
        widget:Option<Box<dyn Widget<C, E>>>,
        #[cfg(debug_assertions)]
        generation:u16,
    }
    pub struct Widgets<C, E>{
        w:Vec<Slot<C, E>>,
        free:Vec<u16>,
    }
    impl<C, E> Widgets<C, E>{
        pub fn new()->Self{Self{w:vec![], free:vec![]}}
        pub fn insert(&mut self, w:Box<dyn Widget<C, E>>)->Widx{
            if let Some(i) = self.free.pop(){
                let slot = &mut self.w[i as usize];
                slot.widget = Some(w);
                Widx{
                    i,
                    #[cfg(debug_assertions)]
                    generation:slot.generation,
                }
            }else{
                let i = self.w.len() as u16;
                self.w.push(Slot{
                    widget:Some(w),
                    #[cfg(debug_assertions)]
                    generation:0,
                });
                Widx{
                    i,
                    #[cfg(debug_assertions)]
                    generation:0,
                }
            }
        }
        pub fn remove(&mut self, w: Widx) -> Box<dyn Widget<C, E>> {
            let slot = &mut self.w[w.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!( slot.generation, w.generation, "Stale widget index!");
            let widget = slot.widget.take().expect("remove on freed widget");
            #[cfg(debug_assertions)]{
                slot.generation = slot.generation.wrapping_add(1);
            }
            self.free.push(w.i);
            widget
        }
        pub fn replace(&mut self, i:Widx, widget:Box<dyn Widget<C, E>>)->Box<dyn Widget<C, E>>{
            let slot = &mut self.w[i.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!(slot.generation, i.generation, "Stale Widx index");
            std::mem::replace(slot.widget.as_mut().unwrap(), widget)
        }
    }
    impl<C, E> std::ops::Index<Widx> for Widgets<C, E>{
        type Output = Box<dyn Widget<C, E>>;
        fn index(&self, i:Widx)->&Self::Output{
            let slot = &self.w[i.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!( slot.generation, i.generation, "Stale Widx index");
            slot.widget.as_ref().unwrap()
        }
    }
    impl<C, E> std::ops::IndexMut<Widx> for Widgets<C, E>{
        fn index_mut(&mut self, i:Widx)->&mut Self::Output{
            let slot = &mut self.w[i.i as usize];
            #[cfg(debug_assertions)]
            assert_eq!(slot.generation, i.generation, "Stale Widx index");
            slot.widget.as_mut().unwrap()
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
    use crate::Widget as W;
    use crate::Background;

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
        pub fn bounds<C, E>(self, n:&Nodes, ws:&Widgets<C, E>)->Bounds{
            match self{
                Self::S(s)=>n[s].b,
                Self::L(i)=>ws[n[i].w].bounds().into(),
            }
        }
    }
    impl Nodes{
        pub fn is_empty(&self) -> bool {
            self.s[ROOT].children.is_empty()
        }

        pub fn new<C, E>(w:u16, h:u16, widgets:&mut Widgets<C, E>, direction:Direction, background: Background)->Self{
            let (l, mut s) = (Fvec::new(), Fvec::new());
            s.insert(S{parent:None, r:Rect{x:0, y:0, w, h}, direction, children:vec![], focus:0, background, 
                b:Bounds{
                    min_w:Bound::Absolute(w),
                    max_w:Bound::Absolute(w),
                    min_h:Bound::Absolute(h),
                    max_h:Bound::Absolute(h),
                }
            });
            let mut s = Self { l, s };
            s.recalc(widgets, ROOT);
            s
        }
    }

    impl L{
        pub fn add<C, E>(n: &mut Nodes, ws:&mut Widgets<C, E>, parent:Sidx, w:Box<dyn W<C, E>>)->Lidx{
            let w = ws.insert(w);
            let lidx = n.l.insert(Self{parent, r:Rect::ZERO, w});
            n[parent].children.push(lidx.into());
            n.recalc(ws, n[lidx].parent);
            lidx
        }
        pub fn split<C, E>(
            n:&mut Nodes,
            ws:&mut Widgets<C, E>,
            d:Direction,
            f:Lidx,
            w:Box<dyn W<C, E>>,
            background:Background,
            split_b:Bounds,
        )->(Lidx, Sidx){
            let p = n[f].parent;
            let i = n[p].children.iter()
                .position(|c| *c == f.into())
                .unwrap();
            let l = match n[p].children[i] {Nidx::L(l) =>l, _ => unreachable!()};
            let (new_l, s) = S::add(n, ws, d, p, w, background, split_b);
            n[l].parent = s;
            n[p].children.swap_remove(i);
            n[s].children.insert(0, l.into());
            n.recalc(ws, p);
            (new_l, s)
        }
    }
    impl S{
        pub fn add<C, E>(
            ns:&mut Nodes,
            ws:&mut Widgets<C, E>,
            direction:Direction,
            p:Sidx,
            w:Box<dyn W<C, E>>,
            background:Background,
            b:Bounds,
        )->(Lidx, Sidx){
            // let background = Box::new(background);
            let s = ns.s.insert(S{parent:p.into(), focus:0, direction, r:Rect::ZERO, children:vec![], b, background});
            ns[p].children.push(s.into());
            let w = ws.insert(w);
            let lidx = ns.l.insert(L{parent:s, r:Rect::ZERO, w});
            ns[s].children.push(lidx.into());
            ns.recalc(ws, p);
            (lidx, s)
        }
    }
}
