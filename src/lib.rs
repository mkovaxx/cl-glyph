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
pub struct Stroke {
    pub is_closed: bool,
    pub thickness: f64,
    pub elems: Vec<Elem>,
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
    pub fn new(width: f64) -> Self {
        Glyph {
            width,
            strokes: vec![],
        }
    }

    pub fn stroke_open(mut self, thickness: f64, elems: impl IntoIterator<Item = Elem>) -> Self {
        self.strokes.push(Stroke {
            is_closed: false,
            thickness,
            elems: elems.into_iter().collect(),
        });
        self
    }

    pub fn stroke_closed(mut self, thickness: f64, elems: impl IntoIterator<Item = Elem>) -> Self {
        self.strokes.push(Stroke {
            is_closed: true,
            thickness,
            elems: elems.into_iter().collect(),
        });
        self
    }
}

impl Stroke {
    /// Iterate over the elements, yielding a `(prev, curr, next)` triple centered on each.
    fn iter_elem_triples(&self) -> impl Iterator<Item = (Option<&Elem>, &Elem, Option<&Elem>)> {
        iter_triples(0..self.elems.len(), self.is_closed).map(|(prev, curr, next)| {
            (
                prev.map(|i| &self.elems[i]),
                &self.elems[curr],
                next.map(|i| &self.elems[i]),
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
