pub use svg::*;

use crate::util::{angle_from_bulge, fuzzy_eq, iter_triples};
use crate::{
    Glyph, Stroke,
    path::{Path, PathVert},
};
use std::f64::consts::PI;

impl Glyph {
    /// Render the Glyph as an SVG Group.
    pub fn to_svg(&self) -> node::element::Group {
        let mut group = node::element::Group::new();
        for stroke in &self.strokes {
            stroke.to_svg(&mut group);
        }
        group.assign("fill", "none");
        group.assign("stroke", "black");
        group
    }
}

impl Stroke {
    /// Render the Stroke as a sequence of SVG Paths.
    pub fn to_svg(&self, parent: &mut node::element::Element) {
        let path = self.to_path();
        path.to_svg(parent);
    }
}

impl Path {
    /// Convert the Path to a sequence of SVG <path> elements.
    fn to_svg(&self, parent: &mut node::element::Element) {
        if self.verts.is_empty() {
            return;
        }

        if is_internally_uniform(&self.verts, self.is_closed) {
            // Make the happy path fast.
            if self.is_closed {
                let path = lower_compatible_to_svg(self.thickness, &self.verts, None);
                parent.append(path);
                return;
            } else {
                let first_is_corner = self.verts[0].is_corner;
                let last_is_corner = self.verts[self.verts.len() - 1].is_corner;
                if first_is_corner == last_is_corner {
                    let path =
                        lower_compatible_to_svg(self.thickness, &self.verts, Some(first_is_corner));
                    parent.append(path);
                    return;
                }
            };
        }

        // Partition into compatible smaller Paths.
        let mut cut_indices = vec![];
        for (prev, curr, next) in iter_triples(0..self.verts.len(), self.is_closed) {
            let is_cut = match (prev, curr, next) {
                // A terminal point is a cut.
                (None, _, _) | (_, _, None) => true,
                (Some(p), _, Some(n)) => {
                    // An internal point is a cut if it is not a corner and has a corner neighbor.
                    !self.verts[curr].is_corner
                        && (self.verts[p].is_corner || self.verts[n].is_corner)
                }
            };
            if is_cut {
                cut_indices.push(curr);
            }
        }

        assert!(!cut_indices.is_empty());

        if !self.is_closed {
            match cut_indices.as_slice() {
                [first, last] if !self.verts[*first].is_corner && self.verts[*last].is_corner => {
                    // Special case for a Path with all corners except for the first vertex.
                    cut_indices.insert(1, *first);
                }
                [first, last] if self.verts[*first].is_corner && !self.verts[*last].is_corner => {
                    // Special case for a Path with all corners except for the last vertex.
                    cut_indices.insert(1, *last);
                }
                [first, mid, last]
                    if self.verts[*first].is_corner && self.verts[*last].is_corner =>
                {
                    // Special case for a Path with all corners except for one round vertex.
                    cut_indices.insert(1, *mid)
                }
                _ => {}
            }
        }

        for i in 1..cut_indices.len() {
            let first_index = cut_indices[i - 1];
            let last_index = cut_indices[i];
            let ends_in_corners =
                Some(self.verts[first_index].is_corner | self.verts[last_index].is_corner);
            let path = lower_compatible_to_svg(
                self.thickness,
                &self.verts[first_index..=last_index],
                ends_in_corners,
            );
            parent.append(path);
        }

        if self.is_closed {
            // Complete the wrap-around section of a closed Path.
            let first_index = cut_indices[0];
            let last_index = cut_indices[cut_indices.len() - 1];
            let ends_in_corners = Some(false);
            let verts = Vec::from_iter(
                self.verts[last_index..]
                    .iter()
                    .chain(&self.verts[0..=first_index])
                    .cloned(),
            );
            let path = lower_compatible_to_svg(self.thickness, &verts, ends_in_corners);
            parent.append(path);
        }
    }
}

/// Check if internal vertices are uniform in their corner-ness.
fn is_internally_uniform(verts: &[PathVert], is_closed: bool) -> bool {
    let internal_vertices = if is_closed {
        verts
    } else {
        verts.get(1..verts.len() - 1).unwrap_or(&[])
    };

    let corner_count = internal_vertices.iter().filter(|v| v.is_corner).count();
    corner_count == 0 || corner_count == internal_vertices.len()
}

/// Lower an SVG-compatible Path to SVG.
fn lower_compatible_to_svg(
    thickness: f64,
    verts: &[PathVert],
    ends_in_corners: Option<bool>,
) -> node::element::Path {
    let is_closed = ends_in_corners.is_none();

    assert!(is_internally_uniform(verts, is_closed));

    let mut data = node::element::path::Data::new();
    data = data.move_to((verts[0].x, verts[0].y));

    for i in 1..verts.len() {
        let v1 = &verts[i - 1];
        let v2 = &verts[i];
        data = lower_segment_to_svg(data, v1, v2);
    }

    if is_closed || verts.len() == 1 {
        let v1 = &verts[verts.len() - 1];
        let v2 = &verts[0];
        data = lower_segment_to_svg(data, v1, v2);
    }

    if is_closed {
        data = data.close();
    }

    let mut path = node::element::Path::new();

    if let Some(ends_in_corners) = ends_in_corners {
        path = path.set(
            "stroke-linecap",
            if ends_in_corners { "butt" } else { "round" },
        );
    }

    if verts.len() >= 2 {
        path = path.set(
            "stroke-linejoin",
            if verts[1].is_corner { "miter" } else { "round" },
        );
    }

    path.set("stroke-width", thickness).set("d", data)
}

fn lower_segment_to_svg(
    data: node::element::path::Data,
    v1: &PathVert,
    v2: &PathVert,
) -> node::element::path::Data {
    if fuzzy_eq(v1.bulge, 0.0) {
        data.line_to((v2.x, v2.y))
    } else {
        let theta = angle_from_bulge(v1.bulge);
        let chord = ((v2.x - v1.x).powi(2) + (v2.y - v1.y).powi(2)).sqrt();
        let radius = (chord / 2.0) / (theta / 2.0).sin().abs();
        let x_axis_rotation = 0.0;

        let sweep_flag = theta.is_sign_positive();
        let large_arc_flag = theta.abs() > PI;

        data.elliptical_arc_to((
            radius,
            radius,
            x_axis_rotation,
            large_arc_flag as u8,
            sweep_flag as u8,
            v2.x,
            v2.y,
        ))
    }
}
