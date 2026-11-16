use crate::util::iter_triples;

mod path;

#[cfg(feature = "svg")]
pub mod svg;

mod util;

#[cfg(feature = "wasm")]
pub mod wasm;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Glyph {
    pub width: f64,
    pub strokes: Vec<Stroke>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Stroke {
    Open(f64, Vec<Elem>),
    Closed(f64, Vec<Elem>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Elem {
    /// Round point
    Node((f64, f64)),
    /// Sharp point
    Cusp((f64, f64)),
    /// Arc given by a radius and one off-curve point
    Arc(f64, (f64, f64)),
    /// Arc given by a radius, a first point, and an off-curve point
    ArcF(f64, (f64, f64), (f64, f64)),
    /// Arc given by a radius, an off-curve point, and a last point
    ArcL(f64, (f64, f64), (f64, f64)),
    /// Arc given by a radius and two off-curve points
    Arc2(f64, (f64, f64), (f64, f64)),
}

impl Glyph {
    /// Create a new glyph, containing no strokes.
    pub fn new(width: f64) -> Self {
        Glyph {
            width,
            strokes: vec![],
        }
    }

    /// Extend the glyph by drawing a stroke.
    pub fn draw(mut self, stroke: Stroke) -> Self {
        self.strokes.push(stroke);
        self
    }
}

impl Stroke {
    /// Create a new open (line) stroke, containing no elements.
    pub fn open(thickness: f64) -> Self {
        Stroke::Open(thickness, vec![])
    }

    /// Create a new closed (loop) stroke, containing no elements.
    pub fn closed(thickness: f64) -> Self {
        Stroke::Closed(thickness, vec![])
    }

    /// Extend the stroke with a (rounded) node.
    pub fn node(mut self, x: f64, y: f64) -> Self {
        self.elems_mut().push(Elem::Node((x, y)));
        self
    }

    /// Extend the stroke with a (mitered) cusp.
    pub fn cusp(mut self, x: f64, y: f64) -> Self {
        self.elems_mut().push(Elem::Cusp((x, y)));
        self
    }

    /// Extend the stroke with an arc with `radius` and off-curve point `p`.
    pub fn arc(mut self, radius: f64, p: [f64; 2]) -> Self {
        self.elems_mut().push(Elem::Arc(radius, (p[0], p[1])));
        self
    }

    /// Extend the stroke with an arc with `radius`, from `f` towards off-curve point `p`.
    pub fn arc_f(mut self, radius: f64, f: [f64; 2], p: [f64; 2]) -> Self {
        self.elems_mut()
            .push(Elem::ArcF(radius, (f[0], f[1]), (p[0], p[1])));
        self
    }

    /// Extend the stroke with an arc with `radius`, towards off-curve point `p` into `l`.
    pub fn arc_l(mut self, radius: f64, p: [f64; 2], l: [f64; 2]) -> Self {
        self.elems_mut()
            .push(Elem::ArcL(radius, (p[0], p[1]), (l[0], l[1])));
        self
    }

    /// Extend the stroke with an arc of a given `radius` and off-curve points `p` and `q`.
    pub fn arc_2(mut self, radius: f64, p: [f64; 2], q: [f64; 2]) -> Self {
        self.elems_mut()
            .push(Elem::Arc2(radius, (p[0], p[1]), (q[0], q[1])));
        self
    }

    /// Whether the stroke is closed (a loop) or open (a line).
    pub fn is_closed(&self) -> bool {
        matches!(self, Stroke::Closed(_, _))
    }

    /// The thickness of the stroke.
    pub fn thickness(&self) -> f64 {
        match self {
            Stroke::Open(thickness, _) | Stroke::Closed(thickness, _) => *thickness,
        }
    }

    fn elems(&self) -> &[Elem] {
        match self {
            Stroke::Open(_, elems) | Stroke::Closed(_, elems) => elems,
        }
    }

    fn elems_mut(&mut self) -> &mut Vec<Elem> {
        match self {
            Stroke::Open(_, elems) | Stroke::Closed(_, elems) => elems,
        }
    }

    /// Iterate over the elements, yielding a `(prev, curr, next)` triple centered on each.
    fn iter_elem_triples(&self) -> impl Iterator<Item = (Option<&Elem>, &Elem, Option<&Elem>)> {
        iter_triples(0..self.elems().len(), self.is_closed()).map(|(prev, curr, next)| {
            (
                prev.map(|i| &self.elems()[i]),
                &self.elems()[curr],
                next.map(|i| &self.elems()[i]),
            )
        })
    }
}

impl Elem {
    fn first_pt(&self) -> (f64, f64) {
        use Elem::*;
        match self {
            Node(p) | Cusp(p) | Arc(_, p) | ArcF(_, p, _) | ArcL(_, p, _) | Arc2(_, p, _) => *p,
        }
    }

    fn last_pt(&self) -> (f64, f64) {
        use Elem::*;
        match self {
            Node(p) | Cusp(p) | Arc(_, p) | ArcF(_, _, p) | ArcL(_, _, p) | Arc2(_, _, p) => *p,
        }
    }
}
