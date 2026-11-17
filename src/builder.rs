use super::*;
use std::marker::PhantomData;
pub struct Builder<'a, C, E, T>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    parent:Parent,
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
        *match self.parent{
            Parent::R => &mut self.nodes.root.background,
            Parent::S(x) => &mut self.nodes[x].background,
            Parent::P(x) => &mut self.nodes[x].background,
        } = bg;
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
    pub fn plane<B>(
        self,
        align: Alignments,
        anchor: Anchors,
        z: u16,
        build: B,
    ) -> Builder<'a, C, E, NonEmpty>
    where
        B: FnOnce(Builder<'a, C, E, Empty>) -> Builder<'a, C, E, NonEmpty>,
    {
        let parent = add_plane(
            self.nodes,
            self.parent,
            align,
            anchor,
            z,
        );
        build(Builder {
            nodes: self.nodes,
            widgets: self.widgets,
            parent,
            marker: PhantomData,
        }).finish()
    }
}
impl<'a, C, E> Builder<'a, C, E, Empty>{
    pub fn bounds(self, bounds:Bounds)->Self{
        match self.parent{
            Parent::R => {}
            Parent::S(x) => self.nodes[x].b = bounds,
            Parent::P(x) => self.nodes[x].b = bounds,
        }
        self
    }
    pub fn new(nodes:&'a mut Nodes, widgets:&'a mut Widgets<C, E>)->Builder<'a, C, E, Empty>{
        Builder{ nodes, widgets, parent: Parent::R, marker:PhantomData}
    }
}

impl<'a, C, E> Builder<'a, C, E, NonEmpty>{
    fn finish(self)->Self{
        self.nodes.propegate_fit(self.widgets, self.parent);
        self
    }
}

fn add<C, E>(ns:&mut Nodes, ws:&mut Widgets<C, E>, p:Parent, w:impl Widget<C, E> + 'static)->Lidx{
    let w = ws.insert(Box::new(w));
    let i = ns.l.insert(Leaf{parent:p, w, r:Rect::ZERO });
    p.mut_children(ns).push(i.into());
    i
}
fn add_empty(ns:&mut Nodes, direction:Direction, p:Parent, background:Background, b:Bounds)->Parent{
    let i = ns.s.insert(Split{
        parent:p.into(),
        focus:0,
        direction,
        children:vec![],
        b,
        background,
        r:Rect::ZERO,
    });
    p.mut_children(ns).push(i.into());
    i.into()
}

fn add_plane(
    ns: &mut Nodes,
    p: Parent,
    align: Alignments,
    anchor: Anchors,
    z: u16,
) -> Parent {
    let i = ns.p.insert(Plane {
        parent: p,
        children: vec![],
        direction: Direction::Vertical,
        r: Rect::ZERO,
        b: Bounds::default(),
        background: background_default,
        align,
        anchor,
        focus: 0,
        z,
    });

    p.mut_children(ns).push(i.into());

    i.into()
}
