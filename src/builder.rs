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
        Leaf::add(self.0.nodes, self.0.widgets, self.0.parent, Box::new(w));
        NonEmpty(self.0)
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
            let parent = Split::add_empty(self.0.nodes, self.0.widgets, Direction::Vertical, self.0.parent, background, Bounds::default());
            build(Empty(Builder{nodes: self.0.nodes, widgets: self.0.widgets, parent}))
    }
    pub fn horizontal<T>(self, build:T)->NonEmpty<'a, C, E>
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = Split::add_empty(self.0.nodes, self.0.widgets, Direction::Horizontal, self.0.parent, background, Bounds::default());
            build(Empty(Builder{nodes: self.0.nodes, widgets: self.0.widgets, parent}))
    }
}

impl<'a, C, E> NonEmpty<'a, C, E>{
    pub fn add(self, w:impl Widget<C, E> + 'static)->Self{
        Leaf::add(self.0.nodes, self.0.widgets, self.0.parent, Box::new(w));
        self
    }
    pub fn bounds(self, bounds:Bounds)->Self{
        NonEmpty(self.0.bounds(bounds))
    }
    pub fn horizontal<T>(self, build:T)->Self
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = Split::add_empty(self.0.nodes, self.0.widgets, Direction::Horizontal, self.0.parent, background, Bounds::default());
            build(Empty(Builder { nodes: self.0.nodes, widgets: self.0.widgets, parent}))
    }
    pub fn vertical<T>(self, build:T)->Self
        where T: FnOnce(Empty<'a, C, E>)->NonEmpty<'a, C, E>{
            let parent = Split::add_empty(self.0.nodes, self.0.widgets, Direction::Vertical, self.0.parent, background, Bounds::default());
            build(Empty(Builder { nodes: self.0.nodes, widgets: self.0.widgets, parent}))
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
