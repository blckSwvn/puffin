mod screen;
mod storage;
mod layout;
use std::io::{self, stdout};

pub use layout::{Rect, Direction, leaf::Bounds as LeafBounds, leaf::Bound as LeafBound};
use layout::*;
use storage::*;
use storage::{
    widgets::Widgets,
    nodes::{Nodes, Sidx, Lidx, Nidx, FOCUS, ROOT}
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
    ) -> Response<Cmd>;
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
    cursor::{Hide, MoveTo, Show}, event::Event, execute, terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}
};
pub enum Response<Cmd>{
    Ignored,
    Consumed,
    Cmd(Cmd),
    Quit,
}
use std::io::Write;

impl<Ctx, Cmd> Ui<Ctx, Cmd>{
    pub fn run<F>(&mut self, ctx: &mut Ctx, mut handler: F)->io::Result<()> where F:
        FnMut(Response<Cmd>, &mut Ctx)->bool{
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
        loop{
            let event = crossterm::event::read()?;
            match event{
                Event::Resize(w, h) => {
                    self.old = ScreenBuffer::new(w, h);
                    self.curr = ScreenBuffer::new(w, h);
                    self.nodes.resize(&self.widgets, w, h);
                    }
                Event::Key(x) => if false == handler(self.widgets[self.nodes[self.focus].w].behaviour(x, ctx), ctx){
                    break
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

pub struct Builder<'a, C, E>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    parent:Sidx,
}
fn background(rect:Rect, screen:&mut ScreenBuffer){
    rect.fill(Cell::default(), screen);
}
impl<C, E> Builder<'_, C, E>{
        pub fn background(&mut self, background: Background)->&mut Self{
            self.nodes[self.parent].background = background;
            self
        }
        pub fn bounds(&mut self, bounds:Bounds)->&mut Self{
            self.nodes[self.parent].b = bounds.into();
            self
        }
    pub fn add(&mut self, widget: impl Widget<C, E> + 'static)->&mut Self{
        Leaf::add(self.nodes, self.widgets, self.parent, Box::new(widget));
        self
    }
    pub fn vertical(&mut self, widget:impl Widget<C, E> + 'static, build: impl FnOnce(&mut Builder<'_, C, E>))->&mut Self{
        let (_, parent) = Split::add(
            self.nodes,
            self.widgets,
            Direction::Vertical,
            self.parent,
            Box::new(widget),
            background,
            Bounds::default(),
        );
        build(&mut Builder {parent, nodes:self.nodes, widgets:self.widgets});
        self
    }
    pub fn horizontal(&mut self, widget:impl Widget<C, E> + 'static, build: impl FnOnce(&mut Builder<'_, C, E>))->&mut Self{
        let (_, parent) = Split::add(
            self.nodes,
            self.widgets,
            Direction::Horizontal,
            self.parent,
            Box::new(widget),
            background,
            leaf::Bounds::default().into(),
        );
        build(&mut Builder {parent, nodes:self.nodes, widgets:self.widgets});
        self
    }
}
pub fn new<C, E>(direction:Direction, widget:impl Widget<C, E> + 'static, build:impl FnOnce(&mut Builder<'_, C, E>))->Ui<C, E>{
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut widgets = Widgets::new();
    let mut puffin = Ui{
        focus:FOCUS,
        nodes: Nodes::new(w, h, &mut widgets, direction, widget, background),
        widgets,
        old: ScreenBuffer::new(w, h),
        curr: ScreenBuffer::new(w, h),
    };
    build(&mut Builder { nodes: &mut puffin.nodes, widgets: &mut puffin.widgets, parent: ROOT });
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
