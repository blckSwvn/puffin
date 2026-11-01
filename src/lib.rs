mod screen;
mod storage;
mod layout;
use std::io::stdout;

pub use storage::*;
pub use layout::*;
use storage::{
    widgets::Widgets,
    nodes::{Nodes, Lidx, Nidx, FOCUS, ROOT}
};

pub use screen::{ScreenBuffer, Cell};

pub trait Widget<Ctx, Cmd> {
    fn new(ctx: &mut Ctx) -> Self
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
    old:ScreenBuffer,
    curr:ScreenBuffer,
}
use crossterm::{
    execute, 
    event::Event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::{MoveTo, Hide}
};
use std::io::Write;
impl<C, E> Ui<C, E>{
    pub fn run<F>(&mut self, ctx: &mut C, mut handler: F) where F: FnMut(Event, &mut Self, &mut C)->bool{
        crossterm::terminal::enable_raw_mode().unwrap();
        execute!(
            stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0,0),
            Hide
        ).unwrap();
        self.draw();
        loop{
            let event = crossterm::event::read().unwrap();
            if false == handler(event, self, ctx){
                break;
            }
            self.draw();
        }
        execute!(stdout(), LeaveAlternateScreen).unwrap()
    }
    fn draw(&mut self){
        sketch(&self.nodes, &mut self.widgets, self.focus, &mut self.curr);
        self.curr.draw(&mut self.old).unwrap();
        stdout().flush().unwrap();
    }
}

pub fn new<C, E>(direction:Direction, widget:impl Widget<C, E> + 'static)->Ui<C, E>{
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut widgets = Widgets::new();
    Ui{
        focus:FOCUS,
        nodes: Nodes::new(w, h, &mut widgets, direction, Bounds{
            min_w:Bound::Absolute(10),
            min_h:Bound::Absolute(10),
            max_h:Bound::Relative(1.0),
            max_w:Bound::Relative(1.0),
        }, widget),
        widgets,
        old: ScreenBuffer::new(w, h),
        curr: ScreenBuffer::new(w, h),
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
