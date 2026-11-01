mod screen;
mod storage;
mod layout;
// pub use layout::{Rect, Direction};
pub use storage::*;
pub use layout::*;
use storage::{
    widgets::Widgets,
    nodes::{Nodes, Lidx, Nidx, FOCUS, ROOT}
};

pub use screen::{ScreenBuffer, Cell};

use crate::layout::Bound::Absolute;
pub trait Widget<Ctx, Cmd> {
    fn new(ctx: &Ctx) -> Self
    where
        Self: Sized;
    fn unfocused_sketch(&mut self, r: Rect, curr: &mut ScreenBuffer);
    fn focused_sketch(&mut self, r: Rect, curr: &mut ScreenBuffer);
    fn behaviour(
        &mut self,
        k: crossterm::event::KeyEvent,
        ctx: &mut Ctx,
    ) -> Option<Cmd>;
}

pub struct Ui<C, E>{
    pub focus:Lidx,
    pub widgets:Widgets<C, E>,
    pub nodes:Nodes,
    pub old:ScreenBuffer,
    pub curr:ScreenBuffer,
}

impl<C, E> Ui<C, E>{
    pub fn new(direction: Direction, k:impl Widget<C, E> + 'static)->Self{
        let (w, h) = crossterm::terminal::size().unwrap();
        let mut widgets = Widgets::new();
        Self {
            focus:FOCUS,
            nodes: Nodes::new(w, h, &mut widgets, direction, Bounds{
                min_w:Bound::Absolute(10),
                min_h:Bound::Absolute(10),
                max_h:Bound::Relative(1.0),
                max_w:Bound::Relative(1.0),
            }, k),
            widgets,
            old: ScreenBuffer::new(w, h),
            curr: ScreenBuffer::new(w, h),
        }
    }
}

pub fn sketch<Ctx, Cmd>(
    n: &Nodes,
    widgets: &mut Widgets<Ctx, Cmd>,
    focus: Lidx,
    curr: &mut ScreenBuffer,
) {
    fn inner<Ctx, Cmd>(
        n: &Nodes,
        w: &mut Widgets<Ctx, Cmd>,
        idx: Nidx,
        focus: Lidx,
        curr: &mut ScreenBuffer,
    ) {
        match idx {
            Nidx::L(l) if l != focus && n[l].r.w != 0 && n[l].r.h != 0 => {
                let leaf = &n[l];
                w[leaf.w].unfocused_sketch(leaf.r, curr);
            }
            Nidx::L(_) => {}
            Nidx::S(s) => {
                for &child in &n[s].children {
                    inner(n, w, child, focus, curr);
                }
            }
        }
    }

    inner(n, widgets, Nidx::S(ROOT), focus, curr);
    let l = &n[focus];
    if l.r.w != 0 && l.r.h != 0 {
        widgets[l.w].focused_sketch(l.r, curr);
    }
}
