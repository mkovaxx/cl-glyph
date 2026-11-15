use glam::DVec2;

use crate::{
    Elem, Stroke,
    util::{bulge_from_angle, fuzzy_eq, fuzzy_vec_eq},
};

impl Stroke {
    /// Convert into a path of vertices.
    pub fn to_path(&self) -> Path {
        let mut verts = vec![];

        for (prev, curr, next) in self.iter_elem_triples() {
            process_elem(prev, curr, next, &mut verts);
        }

        Path {
            is_closed: self.is_closed,
            thickness: self.thickness,
            verts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    /// Indicates whether the path is closed or open.
    pub is_closed: bool,
    /// Thickness of the path.
    pub thickness: f64,
    /// Sequence of path vertices.
    pub verts: Vec<PathVert>,
}

#[derive(Debug, Copy, Clone)]
pub struct PathVert {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Bulge of the path segment that starts with this vertex.
    /// This representation is inspired by the `cavalier_contours` crate.
    /// https://docs.rs/cavalier_contours/0.6.0/cavalier_contours/polyline/struct.PlineVertex.html
    pub bulge: f64,
    /// Indicates whether the path has a corner or is smooth at this vertex.
    pub is_corner: bool,
}

impl PathVert {
    /// Position as a DVec2.
    pub fn pos(&self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }
}

fn process_elem(prev: Option<&Elem>, curr: &Elem, next: Option<&Elem>, verts: &mut Vec<PathVert>) {
    use Elem::*;
    match (prev, curr, next) {
        (_, Node(p), _) => verts.push(PathVert {
            x: p.0,
            y: p.1,
            bulge: 0.0,
            is_corner: false,
        }),
        (_, Cusp(p), _) => verts.push(PathVert {
            x: p.0,
            y: p.1,
            bulge: 0.0,
            is_corner: true,
        }),
        (Some(prev), Arc(radius, p), Some(next)) => {
            let Some((v1, v2)) = make_arc_in_triangle(*radius, prev.last_pt(), *p, next.first_pt())
            else {
                println!("WARNING: could not construct arc in triangle");
                return;
            };
            verts.push(v1);
            verts.push(v2);
        }
        (_, ArcF(radius, p1, p2), Some(next)) => {
            let (v1, v2) = make_arc_from_point_to_line(*radius, *p1, *p2, next.first_pt()).unwrap();
            verts.push(v1);
            verts.push(v2);
        }
        (Some(prev), ArcL(radius, p1, p2), _) => {
            let (mut v2, mut v1) =
                make_arc_from_point_to_line(*radius, *p2, *p1, prev.last_pt()).unwrap();
            v1.bulge = -v2.bulge;
            v2.bulge = 0.0;
            verts.push(v1);
            verts.push(v2);
        }
        (Some(prev), Arc2(radius, p1, p2), Some(next)) => {
            let Some((v1, v2)) =
                make_arc_in_quad(*radius, prev.last_pt(), *p1, *p2, next.first_pt())
            else {
                println!("WARNING: could not construct arc in quad");
                return;
            };
            verts.push(v1);
            verts.push(v2);
        }
        (None, Arc(_, _), None) => todo!(),
        (None, Arc(_, _), Some(_)) => todo!(),
        (None, ArcF(_, _, _), None) => todo!(),
        (None, ArcL(_, _, _), None) => todo!(),
        (None, ArcL(_, _, _), Some(_)) => todo!(),
        (None, Arc2(_, _, _), None) => todo!(),
        (None, Arc2(_, _, _), Some(_)) => todo!(),
        (Some(_), Arc(_, _), None) => todo!(),
        (Some(_), ArcF(_, _, _), None) => todo!(),
        (Some(_), Arc2(_, _, _), None) => todo!(),
    }
}

/// Construct an arc spanning at most 180 degrees, given a radius and three points.
/// At its first point, the arc's tangent is pointing from `a` to `b`.
/// At its last point, the arc's tangent is pointing from `b` to `c`.
fn make_arc_in_triangle(
    radius: f64,
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
) -> Option<(PathVert, PathVert)> {
    let a = DVec2::new(a.0, a.1);
    let b = DVec2::new(b.0, b.1);
    let c = DVec2::new(c.0, c.1);

    if fuzzy_vec_eq(a, b) || fuzzy_vec_eq(b, c) || fuzzy_vec_eq(a, c) {
        return None;
    }

    let t1 = (b - a).normalize();
    let t2 = (c - b).normalize();

    let cos_angle = t1.dot(t2);
    let d = radius * ((1.0 - cos_angle) / (1.0 + cos_angle)).sqrt();

    if fuzzy_eq(d, 0.0) {
        return None;
    }

    // Construct the arc
    let p = b - t1 * d;
    let q = b + t2 * d;
    let sin_angle = t1.perp_dot(t2);
    let angle = sin_angle.atan2(cos_angle);
    let bulge = bulge_from_angle(angle);

    Some((
        PathVert {
            x: p.x,
            y: p.y,
            bulge,
            is_corner: false,
        },
        PathVert {
            x: q.x,
            y: q.y,
            bulge: 0.0,
            is_corner: false,
        },
    ))
}

/// Construct an arc spanning at most 180 degrees, given a radius and two tangent lines.
/// At its first point, the arc's tangent is pointing from `a` to `b`.
/// At its last point, the arc's tangent is pointing from `c` to `d`.
/// If the ray from `a` to `b` and the ray from `d` to `c` agree in direction,
///   then the arc must also be tangent to the line through `b` and `c`.
fn make_arc_in_quad(
    radius: f64,
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
    d: (f64, f64),
) -> Option<(PathVert, PathVert)> {
    let a = DVec2::new(a.0, a.1);
    let b = DVec2::new(b.0, b.1);
    let c = DVec2::new(c.0, c.1);
    let d = DVec2::new(d.0, d.1);

    if fuzzy_vec_eq(a, b)
        || fuzzy_vec_eq(b, c)
        || fuzzy_vec_eq(c, d)
        || fuzzy_vec_eq(a, c)
        || fuzzy_vec_eq(b, d)
        || fuzzy_vec_eq(a, d)
    {
        return None;
    }

    let t_ab = (b - a).normalize();
    let t_cd = (d - c).normalize();

    let cos_angle = t_ab.dot(t_cd);

    if fuzzy_eq(cos_angle, 1.0) {
        // The tangents agree in direction, so an arc cannot be constructed.
        return None;
    }

    if fuzzy_eq(cos_angle, -1.0) {
        // When the two tangents are opposites, compute an arc also tangent to BC.
        let t_bc = (c - b).normalize();

        let cos_theta = t_ab.dot(t_bc);
        let sin_theta = t_ab.perp_dot(t_bc);

        let dist_ab = radius * (sin_theta / (1.0 + cos_theta)).abs();
        let dist_cd = radius * (sin_theta / (1.0 - cos_theta)).abs();
        let p = b - t_ab * dist_ab;
        let q = c + t_cd * dist_cd;
        let bulge = sin_theta.signum();

        return Some((
            PathVert {
                x: p.x,
                y: p.y,
                bulge,
                is_corner: false,
            },
            PathVert {
                x: q.x,
                y: q.y,
                bulge: 0.0,
                is_corner: false,
            },
        ));
    }

    let dist = radius * ((1.0 - cos_angle) / (1.0 + cos_angle)).sqrt();

    if fuzzy_eq(dist, 0.0) {
        panic!("Invalid arc: dist is zero");
    }

    let Some(apex) = intersect_rays(a, t_ab, d, -t_cd) else {
        println!("WARNING: Invalid arc: tangent rays diverge.");
        return Some((
            PathVert {
                x: a.x,
                y: a.y,
                bulge: 0.0,
                is_corner: false,
            },
            PathVert {
                x: d.x,
                y: d.y,
                bulge: 0.0,
                is_corner: false,
            },
        ));
    };

    // Construct the arc
    let p = apex - t_ab * dist;
    let q = apex + t_cd * dist;
    let sin_angle = t_ab.perp_dot(t_cd);
    let angle = sin_angle.atan2(cos_angle);
    let bulge = bulge_from_angle(angle);

    Some((
        PathVert {
            x: p.x,
            y: p.y,
            bulge,
            is_corner: false,
        },
        PathVert {
            x: q.x,
            y: q.y,
            bulge: 0.0,
            is_corner: false,
        },
    ))
}

/// Construct an arc spanning at most 180 degrees, given a radius and three points.
/// The arc's first point is `p1`.
/// At its last point, the arc's tangent is pointing from `p2` to `p3`.
fn make_arc_from_point_to_line(
    radius: f64,
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
) -> Option<(PathVert, PathVert)> {
    let p1 = DVec2::new(p1.0, p1.1);
    let p2 = DVec2::new(p2.0, p2.1);
    let p3 = DVec2::new(p3.0, p3.1);

    if fuzzy_vec_eq(p1, p2) || fuzzy_vec_eq(p2, p3) || fuzzy_vec_eq(p1, p3) {
        return None;
    }

    let t_23 = (p3 - p2).normalize();
    let u_23 = t_23.perp();
    let signed_radius = radius.copysign(u_23.dot(p1 - p2));
    let m1 = p1 - u_23 * signed_radius;

    let q1 = intersect_ray_circle(p2, t_23, m1, radius)?;

    // Work in the tangent frame (t_23, u_23) to avoid explicitly computing the center.
    let v = p1 - q1;
    let vt = v.dot(t_23);
    let vu = v.dot(u_23);

    let angle = (-vt * signed_radius).atan2((signed_radius - vu) * signed_radius);
    let bulge = bulge_from_angle(angle);

    Some((
        PathVert {
            x: p1.x,
            y: p1.y,
            bulge,
            is_corner: false,
        },
        PathVert {
            x: q1.x,
            y: q1.y,
            bulge: 0.0,
            is_corner: false,
        },
    ))
}

fn intersect_ray_circle(orig: DVec2, dir: DVec2, center: DVec2, radius: f64) -> Option<DVec2> {
    let b = (orig - center).dot(dir);
    let c = (orig - center).dot(orig - center) - radius.powi(2);

    let d = b * b - c;
    if d < 0.0 {
        return None;
    }
    let r = d.sqrt();

    let t = b.abs() + r;
    Some(orig + t * dir)
}

fn intersect_rays(o1: DVec2, d1: DVec2, o2: DVec2, d2: DVec2) -> Option<DVec2> {
    let denom = d1.perp_dot(d2);
    if fuzzy_eq(denom, 0.0) {
        return None;
    }

    let delta = o2 - o1;
    let t = delta.perp_dot(d2) / denom;
    let u = delta.perp_dot(d1) / denom;

    if t < 0.0 || u < 0.0 {
        return None;
    }

    // Make computation symmetrical for better stability
    let p = (o1 + d1 * t + o2 + d2 * u) * 0.5;

    Some(p)
}
