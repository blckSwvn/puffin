mod screen;
mod storage;
mod layout;
mod builder;
use std::io::{self, stdout};
pub use builder::*;

pub use layout::{Rect, Direction, leaf, Bounds, FocusDirection};
use layout::*;
use storage::*;
use storage::{
    widgets::Widgets,
    nodes::{Nodes, Sidx, Lidx, Nidx, FOCUS, ROOT}
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyEvent},
    execute,
    terminal::{
        Clear,
        ClearType,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode
    }
};
pub use screen::{ScreenBuffer, Cell};

pub trait Widget<Ctx, Cmd>{
    fn new(ctx: &mut Ctx) -> Self where Self: Sized;
    fn draw(&mut self, r:Rect, screen:&mut ScreenBuffer, focused:bool);
    fn bounds(&self)->leaf::Bounds;
    fn behaviour(
        &mut self,
        k: KeyEvent,
        ctx: &mut Ctx,
    ) -> Response<Cmd>;
}
pub type Background = fn(Rect, &mut ScreenBuffer);
pub struct Ui<C, E>{
    pub focus:Lidx,
    widgets:Widgets<C, E>,
    nodes:Nodes,
    old:ScreenBuffer,
    curr:ScreenBuffer,
    quit:bool,
}
pub enum Response<Cmd>{
    Ignored,
    Consumed,
    Cmd(Cmd),
}
use std::io::Write;

impl<Ctx, Cmd> Ui<Ctx, Cmd>{
    pub fn focus(&mut self, d:FocusDirection){
        self.nodes.focus(&mut self.focus, d);
    }
    pub fn close(&mut self){
        self.nodes.close(&mut self.widgets, &mut self.focus);
    }
    pub fn change_widget(&mut self, w:impl Widget<Ctx, Cmd> + 'static){
        self.nodes.change_widget(self.focus, &mut self.widgets, w);
    }
    pub fn split(&mut self, d:Direction, w:Box<dyn Widget<Ctx, Cmd>>, split_b: Bounds){
        self.nodes.split(&mut self.widgets, d, self.focus, w, background, split_b);
    }
    pub fn quit(&mut self){
        self.quit = false;
    }
}
impl<Ctx, Cmd> Ui<Ctx, Cmd>{
    pub fn run<F>(&mut self, ctx: &mut Ctx, mut handler: F)->io::Result<()> where F:
        FnMut(KeyEvent, Response<Cmd>, &mut Ctx, &mut Ui<Ctx, Cmd>){
        fn guard()->io::Result<impl Drop>{
            crossterm::terminal::enable_raw_mode()?;
            if let Err(x) = execute!(
                stdout(),
                EnterAlternateScreen,
                Clear(ClearType::All),
                MoveTo(0,0),
                Hide
            ){
                let _ = disable_raw_mode();
                return Err(x)
            }
            struct T;
            impl Drop for T{
                fn drop(&mut self) {
                    let mut out = stdout();
                    let _ = execute!(out, LeaveAlternateScreen, Show);
                    let _ = disable_raw_mode();
                }
            }
            Ok(T)
        }
        let _guard = guard()?;
        self.draw()?;
        while !self.quit{
            let event = crossterm::event::read()?;
            match event{
                Event::Resize(w, h) => {
                    self.old = ScreenBuffer::new(w, h);
                    self.curr = ScreenBuffer::new(w, h);
                    self.nodes.resize(&self.widgets, w, h);
                    }
                Event::Key(x) => {
                    let b = self.widgets[self.nodes[self.focus].w].behaviour(x, ctx);
                    handler(x, b, ctx, self)
                }
                _ => {}
            }
            self.draw()?;
        }
        Ok(())
    }
    fn draw(&mut self)->std::io::Result<()>{
        sketch(&self.nodes, &mut self.widgets, self.focus, &mut self.curr);
        self.curr.draw(&mut self.old)?;
        stdout().flush()?;
        Ok(())
    }
}

pub fn background(rect:Rect, screen:&mut ScreenBuffer){
    rect.fill(Cell::default(), screen);
}
pub fn new<C, E>(direction:Direction, build:impl FnOnce(Empty<'_, C, E>)->NonEmpty<'_, C, E>)->Ui<C, E>{
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut widgets = Widgets::new();
    let mut puffin = Ui{
        focus:FOCUS,
        nodes: Nodes::new(w, h, &mut widgets, direction, background),
        widgets,
        old: ScreenBuffer::new(w, h),
        curr: ScreenBuffer::new(w, h),
        quit: false,
    };
    build(Empty::new(&mut puffin.nodes, &mut puffin.widgets));
    puffin.nodes.recalc(&puffin.widgets, ROOT);
    puffin
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
