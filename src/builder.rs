use super::*;
use std::marker::PhantomData;
pub struct Builder<'a, C, E, T>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    parent:Sidx,
    marker:PhantomData<fn()->T>,
}

pub struct Empty;
pub struct NonEmpty;

impl <'a, C, E, T> Builder<'a, C, E, T>{
    pub fn add(self, w:impl Widget<C, E> + 'static)->Builder<'a, C, E, NonEmpty>{
        add(self.nodes, self.widgets, self.parent, w);
        Builder{nodes:self.nodes, widgets:self.widgets, parent:self.parent, marker:PhantomData}
    }
    pub fn background(self, bg:Background)->Self{
        self.nodes[self.parent].background = bg;
        self
    }
    pub fn vertical<B>(self, build:B)->Builder<'a, C, E, NonEmpty>
        where B: FnOnce(Builder<'a, C, E, T>)->Builder<'a, C, E, NonEmpty>{
            let parent = add_empty(self.nodes, Direction::Vertical, self.parent, background_default, Bounds::default());
            build(Builder{nodes: self.nodes, widgets: self.widgets, parent, marker:PhantomData}).finish()
    }
    pub fn horizontal<B>(self, build:B)->Builder<'a, C, E, NonEmpty>
        where B: FnOnce(Builder<'a, C, E, T>)->Builder<'a, C, E, NonEmpty>{
            let parent = add_empty(self.nodes, Direction::Horizontal, self.parent, background_default, Bounds::default());
            build(Builder{nodes: self.nodes, widgets: self.widgets, parent, marker:PhantomData}).finish()
    }
}
impl<'a, C, E> Builder<'a, C, E, Empty>{
    pub fn bounds(self, bounds:Bounds){
        self.nodes[self.parent].b = bounds;
    }
    pub fn new(nodes:&'a mut Nodes, widgets:&'a mut Widgets<C, E>)->Builder<'a, C, E, Empty>{
        Builder{ nodes, widgets, parent: ROOT, marker:PhantomData}
    }
}

impl<'a, C, E> Builder<'a, C, E, NonEmpty>{
    fn finish(self)->Self{
        self.nodes.propegate_fit(self.widgets, self.parent);
        self
    }
}

fn add<C, E>(ns:&mut Nodes, ws:&mut Widgets<C, E>, p:Sidx, w:impl Widget<C, E> + 'static)->Lidx{
    let w = ws.insert(Box::new(w));
    let i = ns.l.insert(Leaf{parent:p, w, r:Rect::ZERO });
    ns[p].children.push(i.into());
    i
}
fn add_empty(ns:&mut Nodes, direction:Direction, p:Sidx, background:Background, b:Bounds)->Sidx{
    let i = ns.s.insert(Split{
        parent:p.into(),
        focus:0,
        direction,
        children:vec![],
        b,
        background,
        r:Rect::ZERO,
    });
    ns[p].children.push(i.into());
    i
}
