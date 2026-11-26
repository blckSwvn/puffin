use super::*;
use std::marker::PhantomData;
pub struct Builder<'a, C, E, S, K: Into<Parent> + Copy>{
    nodes: &'a mut Nodes,
    widgets: &'a mut Widgets<C, E>,
    k:K,
    z:u16,
    marker:PhantomData<fn()->S>,
}

pub mod k{
    #[derive(Clone, Copy)]
    pub struct Root;
    impl From<Root> for super::Parent{
        fn from(_: Root) -> Self {
            Self::R
        }
    }
}

pub struct Empty;
pub struct NonEmpty;

impl <'a, C, E, S, K: Into<Parent> + Copy> Builder<'a, C, E, S, K>{
    pub fn add(self, w:impl Widget<C, E> + 'static)->Builder<'a, C, E, NonEmpty, K>{
        add(self.nodes, self.widgets, self.k, w);
        Builder{nodes:self.nodes, widgets:self.widgets, k:self.k, z:self.z, marker:PhantomData}
    }
    pub fn background(self, bg:Background)->Self{
        *match self.k.into(){
            Parent::R => &mut self.nodes.root.background,
            Parent::S(x) => &mut self.nodes[x].background,
            Parent::P(x) => &mut self.nodes[x].background,
        } = bg;
        self
    }
    fn direction<B>(self, d:Direction, build:B)->Builder<'a, C, E, NonEmpty, Sidx>
        where B: FnOnce(Builder<'a, C, E, Empty, Sidx>)->Builder<'a, C, E, NonEmpty, Sidx>{
            let k = add_empty(self.nodes, d, self.k, background_default, Bounds::default());
            build(Builder { nodes: self.nodes, widgets: self.widgets, k, z:self.z, marker: PhantomData })
    }
    pub fn vertical<B>(self, build:B)->Builder<'a, C, E, NonEmpty, Sidx>
        where B: FnOnce(Builder<'a, C, E, Empty, Sidx>)->Builder<'a, C, E, NonEmpty, Sidx>{
            self.direction(Direction::Vertical, build)
    }
    pub fn horizontal<B>(self, build:B)->Builder<'a, C, E, NonEmpty, Sidx>
        where B: FnOnce(Builder<'a, C, E, Empty, Sidx>)->Builder<'a, C, E, NonEmpty, Sidx>{
            self.direction(Direction::Horizontal, build)
    }
    pub fn plane<B>(
        self,
        build: B,
    ) -> Builder<'a, C, E, NonEmpty, Pidx>
    where
        B: FnOnce(Builder<'a, C, E, Empty, Pidx>) -> Builder<'a, C, E, NonEmpty, Pidx>,
    {
        let k = add_plane(
            self.nodes, self.k, Alignments::default(), Anchors::default(), self.z+1,
        );
        build(Builder {
            nodes: self.nodes,
            widgets: self.widgets,
            k,
            z:self.z+1,
            marker: PhantomData,
        }).finish()
    }
}
impl<'a, C, E, K: Into<Parent> + Copy> Builder<'a, C, E, Empty, K>{
    pub fn bounds(self, bounds:Bounds)->Self{
        match self.k.into(){
            Parent::R => {}
            Parent::S(x) => self.nodes[x].b = bounds,
            Parent::P(x) => self.nodes[x].b = bounds,
        }
        self
    }
}
impl<'a, C, E> Builder<'a, C, E, Empty, k::Root>{
    pub fn new(
        nodes:&'a mut Nodes, widgets:&'a mut Widgets<C, E>
        )->Builder<'a, C, E, Empty, k::Root>{
        Builder{ nodes, widgets, k: k::Root, z:0, marker:PhantomData}
    }
}
impl<'a, C, E, K: Into<Parent> + Copy> Builder<'a, C, E, NonEmpty, K>{
    fn finish(self)->Self{
        self.nodes.propegate_fit(self.widgets, self.k.into());
        self
    }
}
impl<'a, C, E> Builder<'a, C, E, Empty, Pidx>{
    pub fn align(self, align:Alignments)->Self{
        self.nodes[self.k].align = align;
        self
    }
    pub fn anchors(self, anchors:Anchors)->Self{
        self.nodes[self.k].anchor = anchors;
        self
    }
    pub fn z(self, z_off:u16)->Self{
        let z = (self.z-1).checked_add(z_off).expect("z overflow");
        self.nodes[self.k].z = self.z-1+z_off;
        Self{
            nodes: self.nodes,
            widgets: self.widgets,
            k:self.k,
            z,
            marker: PhantomData 
        }
    }
}

fn add<C, E, P:Into<Parent>>(ns:&mut Nodes, ws:&mut Widgets<C, E>, p:P, w:impl Widget<C, E> + 'static)->Lidx{
    let w = ws.insert(Box::new(w));
    let p = p.into();
    let i = ns.l.insert(Leaf{parent:p, w, r:Rect::ZERO });
    p.mut_children(ns).push(i.into());
    i
}
fn add_empty<P:Into<Parent>+Copy>(ns:&mut Nodes, direction:Direction, p:P, background:Background, b:Bounds)->Sidx{
    let i = ns.s.insert(Split{
        parent:p.into(),
        focus:0,
        direction,
        children:vec![],
        b,
        background,
        r:Rect::ZERO,
    });
    p.into().mut_children(ns).push(i.into());
    i
}

fn add_plane<P: Into<Parent> + Copy>(
    ns: &mut Nodes,
    p:P,
    align: Alignments,
    anchor: Anchors,
    z: u16,
) -> Pidx{
    let i = ns.p.insert(Plane {
        parent: p.into(),
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
    p.into().mut_children(ns).push(i.into());
    i
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
