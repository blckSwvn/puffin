use super::*;
struct Builder<'a, C, E>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    parent:Sidx,
}

pub struct Empty<'a, C, E>(Builder<'a, C, E>);
pub struct NonEmpty<'a, C, E>(Builder<'a, C, E>);

impl<'a, C, E> Empty<'a, C, E>{
    pub fn new(nodes:&'a mut Nodes, widgets:&'a mut Widgets<C, E>)->Empty<'a, C, E>{
        Empty(Builder{ nodes, widgets, parent: ROOT})
    }
    pub fn add(self, w:impl Widget<C, E> + 'static)->NonEmpty<'a, C, E>{
        add(self.0.nodes, self.0.widgets, self.0.parent, w);
        NonEmpty(self.0).finish()
    }
    pub fn bounds(self, bounds:Bounds)->Empty<'a, C, E>{
        Empty(self.0.bounds(bounds))
    }
    pub fn background(&mut self, background:Background)->&mut Self{
        self.0.background(background);
        self
    }
    pub fn vertical<T>(self, build:T)->NonEmpty<'a, C, E>
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = add_empty(self.0.nodes, Direction::Vertical, self.0.parent, background, Bounds::default());
            build(Empty(Builder{nodes: self.0.nodes, widgets: self.0.widgets, parent})).finish()
    }
    pub fn horizontal<T>(self, build:T)->NonEmpty<'a, C, E>
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = add_empty(self.0.nodes, Direction::Horizontal, self.0.parent, background, Bounds::default());
            build(Empty(Builder{nodes: self.0.nodes, widgets: self.0.widgets, parent})).finish()
    }
}

impl<'a, C, E> NonEmpty<'a, C, E>{
    fn finish(self)->Self{
        self.0.nodes.propegate_fit(self.0.widgets, self.0.parent);
        self
    }
    pub fn add(self, w:impl Widget<C, E> + 'static)->Self{
        add(self.0.nodes, self.0.widgets, self.0.parent, w);
        self
    }
    pub fn bounds(self, bounds:Bounds)->Self{
        NonEmpty(self.0.bounds(bounds)).finish()
    }
    pub fn horizontal<T>(self, build:T)->Self
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = add_empty(self.0.nodes, Direction::Horizontal, self.0.parent, background, Bounds::default());
            build(Empty(Builder { nodes: self.0.nodes, widgets: self.0.widgets, parent})).finish()
    }
    pub fn vertical<T>(self, build:T)->Self
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = add_empty(self.0.nodes, Direction::Vertical, self.0.parent, background, Bounds::default());
            build(Empty(Builder { nodes: self.0.nodes, widgets: self.0.widgets, parent})).finish()
    }
}

impl<'a, C, E> Builder<'a, C, E>{
    pub fn background(&mut self, background: Background)->&mut Self{
        self.nodes[self.parent].background = background;
        self
    }
    pub fn bounds(self, bounds:Bounds)->Self{
        self.nodes[self.parent].b = bounds.into();
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
