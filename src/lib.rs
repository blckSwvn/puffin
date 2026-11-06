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

pub trait Widget<Ctx, Cmd>{
    fn new(ctx: &mut Ctx) -> Self where Self: Sized;
    fn draw(&mut self, r:Rect, screen:&mut ScreenBuffer, focused:bool);
    fn bounds(&self)->leaf::Bounds;
    fn behaviour(
        &mut self,
        k: crossterm::event::KeyEvent,
        ctx: &mut Ctx,
    ) -> Option<Cmd>;
}
pub type Background = fn(Rect, &mut ScreenBuffer);

pub struct Ui<C, E>{
    pub focus:Lidx,
    pub widgets:Widgets<C, E>,
    pub nodes:Nodes,
    old:ScreenBuffer,
    curr:ScreenBuffer,
}
use crossterm::{
    execute, 
    event::{Event, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::{MoveTo, Hide}
};
use std::io::Write;
impl<C, E> Ui<C, E>{
    pub fn run<F>(&mut self, ctx: &mut C, mut handler: F) where F: FnMut(KeyEvent, &mut Self, &mut C)->bool{
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
            match event{
                Event::Resize(w, h) => self.nodes.resize(&self.widgets, w, h),
                Event::Key(x) => if false == handler(x, self, ctx){
                    break;
                }
                _ => {}
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

pub fn new<C, E>(direction:Direction, widget:impl Widget<C, E> + 'static, background: Background)->Ui<C, E>{
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut widgets = Widgets::new();
    Ui{
        focus:FOCUS,
        nodes: Nodes::new(w, h, &mut widgets, direction, widget, background),
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
    inner(n, widgets, Nidx::S(ROOT), focus, curr);
    fn inner<Ctx, Cmd>(
        n: &Nodes,
        w: &mut Widgets<Ctx, Cmd>,
        idx: Nidx,
        focus: Lidx,
        screen: &mut ScreenBuffer,
    ) {
        match idx {
            Nidx::L(i) => {
                let l = &n[i];
                w[l.w].draw(l.r, screen, i == focus);
            }
            Nidx::S(i) => {
                let s = &n[i];
                (s.background)(s.r, screen);
                for &child in &n[i].children {
                    inner(n, w, child, focus, screen);
                }
            }
        }
    }
}
