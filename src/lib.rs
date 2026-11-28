mod screen;
mod storage;
mod layout;
mod builder;
use std::io::{self, stdout};
use std::time::Duration;
use builder::*;
pub use builder::Builder;
use crossterm::style::Attributes;
pub use layout::{Rect, Direction, leaf, Bounds, Bound, Alignment, Alignments, Anchor, Anchors, FocusDirection};
use layout::*;
use storage::*;
use storage::{
    widgets::Widgets,
    nodes::{Nodes, Sidx, Lidx, Nidx, Pidx, FOCUS}
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
pub use crossterm::{
    event::KeyCode,
    style::Color,
};
pub use screen::ScreenBuffer;

pub trait Widget<C, E>{
    fn draw(&mut self, r:Rect, screen:&mut ScreenBuffer, focused:bool);
    fn bounds(&self)->leaf::Bounds{
        leaf::Bounds::default()
    }
    fn behaviour(
        &mut self,
        event: KeyEvent,
        ctx: &mut C,
    ) -> Response<E>;
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

impl<C, E> Ui<C, E>{
    pub fn focus(&mut self, d:FocusDirection){
        self.nodes.focus(&mut self.focus, d);
    }
    pub fn close(&mut self){
        if let Err(_) = self.nodes.close(&mut self.widgets, &mut self.focus){
            self.quit()
        }
    }
    pub fn change_widget(&mut self, w:impl Widget<C, E> + 'static){
        self.nodes.change_widget(self.focus, &mut self.widgets, w);
    }
    pub fn split(&mut self, d:Direction, w:Box<dyn Widget<C, E>>, split_b: Bounds){
        self.nodes.split(&mut self.widgets, d, self.focus, w, background_default, split_b);
    }
    pub fn add(&mut self, w:Box<dyn Widget<C, E>>){
        let p = self.nodes[self.focus].parent;
        Leaf::add(&mut self.nodes, &mut self.widgets, p, w);
    }
    pub fn spawn<S, K, B>(&mut self, build:B)
        where
            K: Into<Parent> + Copy,
            B: FnOnce(Builder<'_, C, E, Empty, Parent>)->Builder<'_, C, E, S, K>,
    {
        let p = self.nodes[self.focus].parent;
        build(Builder::new(&mut self.nodes, &mut self.widgets, p));
        self.nodes.recalc(&self.widgets, p);
    }
    pub fn quit(&mut self){
        self.quit = true;
    }
}
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering}
};
impl<C, E> Ui<C, E>{
    pub fn run<F>(&mut self, ctx: &mut C, mut handler: F)->io::Result<()> where F:
        FnMut(KeyEvent, Response<E>, &mut C, &mut Ui<C, E>){
        fn guard()->io::Result<impl Drop>{
            let x = Arc::new(AtomicBool::new(false));
            fn install_panic_hook(restored:Arc<AtomicBool>){
                let original_hook = std::panic::take_hook();

                std::panic::set_hook(Box::new(move |info| {
                    if !restored.swap(true, Ordering::Acquire){
                        let mut out = stdout();
                        let _ = execute!(
                            out,
                            LeaveAlternateScreen,
                            Show
                        );
                        let _ = disable_raw_mode();
                        let _ = out.flush();
                    }
                    original_hook(info);
                }));
            }
            install_panic_hook(x.clone());
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
            struct T(Arc<AtomicBool>);
            impl Drop for T{
                fn drop(&mut self) {
                    if !self.0.swap(true, Ordering::SeqCst){
                        let mut out = stdout();
                        let _ = execute!(out, LeaveAlternateScreen, Show);
                        let _ = disable_raw_mode();
                        let _ = out.flush();
                    }
                }
            }
            Ok(T(x.clone()))
        }
        let _guard = guard()?;
        self.draw()?;
        while !self.quit{
            if crossterm::event::poll(Duration::from_millis(16))?{
                let event = crossterm::event::read()?;
                let mut handler = |x| {
                    let b = self.widgets[self.nodes[self.focus].w].behaviour(x, ctx);
                    handler(x, b, ctx, self)
                };
                match event{
                    Event::Resize(w, h) => {
                        self.old = ScreenBuffer::new(w, h);
                        self.curr = ScreenBuffer::new(w, h);
                        self.nodes.resize(&self.widgets, w, h);
                    }
                    Event::Key(x) => {
                        handler(x)
                    }
                    Event::Mouse(_) => {
                        panic!("puffin does not and will not support mouse events")
                    }
                    _ => {}
                }
            }
            self.draw()?;
        }
        Ok(())
    }
    fn draw(&mut self)->std::io::Result<()>{
        sketch(&self.nodes, &mut self.widgets, self.focus, &mut self.curr);
        self.curr.draw(&mut self.old)?;
        Ok(())
    }
}

pub fn background_default(rect:Rect, screen:&mut ScreenBuffer){
    rect.fill(" ".into(), Color::Black, Color::Black, Attributes::none(), screen);
}
pub fn new<C, E>(direction:Direction, build:impl FnOnce(Builder<'_, C, E, Empty, Parent>)->Builder<'_, C, E, NonEmpty, Parent>)->Ui<C, E>{
    let (w, h) = crossterm::terminal::size().unwrap();
    let mut widgets = Widgets::new();
    let mut puffin = Ui{
        focus:FOCUS,
        nodes: Nodes::new(w, h, &mut widgets, direction, background_default),
        widgets,
        old: ScreenBuffer::new(w, h),
        curr: ScreenBuffer::new(w, h),
        quit: false,
    };
    build(Builder::new(&mut puffin.nodes,&mut puffin.widgets, Parent::R));
    puffin.nodes.recalc(&puffin.widgets, Parent::R);
    puffin
} 

pub fn sketch<Ctx, Cmd>(
    n: &Nodes,
    widgets: &mut Widgets<Ctx, Cmd>,
    focus: Lidx,
    curr: &mut ScreenBuffer,
) {
    fn draw_content<Ctx, Cmd>(
        n: &Nodes,
        w: &mut Widgets<Ctx, Cmd>,
        idx: Nidx,
        focus: Lidx,
        screen: &mut ScreenBuffer,
        planes: &mut Vec<Pidx>,
    ) {
        match idx {
            Nidx::L(i) => {
                let l = &n[i];
                w[l.w].draw(l.r, screen, i == focus);
            }
            Nidx::S(i) => {
                let s = &n[i];
                (s.background)(s.r, screen);

                for &child in &s.children {
                    draw_content(n, w, child, focus, screen, planes);
                }
            }
            Nidx::P(i) => {
                planes.push(i);
            }
        }
    }

    fn draw_layer<Ctx, Cmd>(
        n: &Nodes,
        w: &mut Widgets<Ctx, Cmd>,
        mut planes: Vec<Pidx>,
        focus: Lidx,
        screen: &mut ScreenBuffer,
    ) {
        planes.sort_unstable_by_key(|&i| n[i].z);

        for i in planes {
            let p = &n[i];
            (p.background)(p.r, screen);
            let mut nested = Vec::new();
            for &child in &p.children {
                draw_content(n, w, child, focus, screen, &mut nested);
            }
            draw_layer(n, w, nested, focus, screen);
        }
    }
    (n.root.background)(Root::r(n), curr);
    let mut planes = Vec::new();
    for &child in &n.root.children {
        draw_content(n, widgets, child, focus, curr, &mut planes);
    }
    draw_layer(n, widgets, planes, focus, curr);
}
