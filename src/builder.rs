use super::*;
use std::marker::PhantomData;
pub struct Builder<'a, C, E, S, K>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    parent:Parent,
    marker:PhantomData<fn()->(S, K)>,
}

pub mod k{
    pub struct Plane;
    pub struct Split;
    pub struct Root;
}

pub struct Empty;
pub struct NonEmpty;

impl <'a, C, E, S, K> Builder<'a, C, E, S, K>{
    pub fn add(self, w:impl Widget<C, E> + 'static)->Builder<'a, C, E, NonEmpty, K>{
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
    pub fn vertical<B>(self, build:B)->Builder<'a, C, E, NonEmpty, k::Split>
        where B: FnOnce(Builder<'a, C, E, Empty, K>)->Builder<'a, C, E, NonEmpty, k::Split>{
            let parent = add_empty(self.nodes, Direction::Vertical, self.parent, background_default, Bounds::default());
            build(Builder{nodes: self.nodes, widgets: self.widgets, parent, marker:PhantomData}).finish()
    }
    pub fn horizontal<B>(self, build:B)->Builder<'a, C, E, NonEmpty, k::Split>
        where B: FnOnce(Builder<'a, C, E, Empty, K>)->Builder<'a, C, E, NonEmpty, k::Split>{
            let parent = add_empty(self.nodes, Direction::Horizontal, self.parent, background_default, Bounds::default());
            build(Builder{nodes: self.nodes, widgets: self.widgets, parent, marker:PhantomData}).finish()
    }
    pub fn plane<B>(
        self,
        z: u16,
        build: B,
    ) -> Builder<'a, C, E, NonEmpty, k::Plane>
    where
        B: FnOnce(Builder<'a, C, E, Empty, k::Plane>) -> Builder<'a, C, E, NonEmpty, k::Plane>,
    {
        let parent = add_plane(
            self.nodes,
            self.parent,
            Alignments::default(),
            Anchors::default(),
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
impl<'a, C, E, K> Builder<'a, C, E, Empty, K>{
    pub fn bounds(self, bounds:Bounds)->Self{
        match self.parent{
            Parent::R => {}
            Parent::S(x) => self.nodes[x].b = bounds,
            Parent::P(x) => self.nodes[x].b = bounds,
        }
        self
    }
}
impl<'a, C, E> Builder<'a, C, E, Empty, k::Root>{
    pub fn new(nodes:&'a mut Nodes, widgets:&'a mut Widgets<C, E>)->Builder<'a, C, E, Empty, k::Root>{
        Builder{ nodes, widgets, parent: Parent::R, marker:PhantomData}
    }
}
impl<'a, C, E, K> Builder<'a, C, E, NonEmpty, K>{
    fn finish(self)->Self{
        self.nodes.propegate_fit(self.widgets, self.parent);
        self
    }
}
impl<'a, C, E> Builder<'a, C, E, NonEmpty, k::Plane>{
    pub fn align(self, align:Alignments)->Self{
        let Parent::P(p) = self.parent else{unreachable!()};
        self.nodes[p].align = align;
        self
    }
    pub fn anchors(self, anchors:Anchors)->Self{
        let Parent::P(p) = self.parent else{unreachable!()};
        self.nodes[p].anchor = anchors;
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

impl Default for Alignments{
    fn default() -> Self {
        Self { x: Alignment::default(), y: Alignment::default()}
    }
}
impl Default for Alignment{
    fn default() -> Self {
        Self::Start
    }
}
impl Default for Anchors{
    fn default() -> Self {
        Self { x: Anchor::default(), y: Anchor::default() }
    }
}
impl Default for Anchor{
    fn default() -> Self {
        Self::Relative2Parent(0.0)
    }
}
