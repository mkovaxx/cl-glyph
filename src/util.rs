use std::ops::Range;

pub fn iter_triples(
    range: Range<usize>,
    is_circular: bool,
) -> impl Iterator<Item = (Option<usize>, usize, Option<usize>)> {
    TripleIterator {
        range,
        index: 0,
        is_circular,
    }
}

struct TripleIterator {
    range: Range<usize>,
    index: usize,
    is_circular: bool,
}

impl Iterator for TripleIterator {
    type Item = (Option<usize>, usize, Option<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.range.end {
            return None;
        }

        let first = self.range.start;
        let last = self.range.end - 1;

        let item = match (self.index == first, self.index == last, self.is_circular) {
            (false, false, _) => (Some(self.index - 1), self.index, Some(self.index + 1)),
            (true, false, false) => (None, self.index, Some(self.index + 1)),
            (false, true, false) => (Some(self.index - 1), self.index, None),
            (true, false, true) => (Some(last), self.index, Some(self.index + 1)),
            (false, true, true) => (Some(self.index - 1), self.index, Some(first)),
            (true, true, false) => (None, self.index, None),
            (true, true, true) => (Some(self.index), self.index, Some(self.index)),
        };

        self.index += 1;

        Some(item)
    }
}

pub(crate) fn fuzzy_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON
}

pub(crate) fn fuzzy_vec_eq(a: glam::DVec2, b: glam::DVec2) -> bool {
    fuzzy_eq(a.x, b.x) && fuzzy_eq(a.y, b.y)
}

pub(crate) fn bulge_from_angle(angle: f64) -> f64 {
    (angle / 4.0).tan()
}

pub(crate) fn angle_from_bulge(bulge: f64) -> f64 {
    4.0 * bulge.atan()
}
