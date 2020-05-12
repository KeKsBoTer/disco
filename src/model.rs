use crate::polygons::Polygon;
use cgmath::*;

use std::cmp::Ordering;
use std::path::Path;
use tobj::{load_obj, Model as TobjModel};

pub type Slice = Vec<Polygon>;

#[derive(Debug)]
pub struct Model {
    // all points in model
    vertices: Vec<Point3<f32>>,
    // normal for every face
    normals: Vec<Vector3<f32>>,
    // the model's faces
    faces: Vec<[u32; 3]>,
}

impl Model {
    // loads model from file
    pub fn load(file: &str) -> Model {
        let obj_data = load_obj(&Path::new(&file));
        assert!(obj_data.is_ok());

        let (models, _) = obj_data.unwrap();
        let m: &TobjModel = &models[0];

        let vertices: Vec<Point3<f32>> = m
            .mesh
            .positions
            .chunks(3)
            .map(|v| Point3::new(v[0], v[1], v[2]))
            .collect();
        let indices: Vec<[u32; 3]> = m
            .mesh
            .indices
            .chunks(3)
            .map(|f| [f[0], f[1], f[2]])
            .collect();

        // calculate normal for each face
        let normals: Vec<Vector3<f32>> = indices
            .iter()
            .map(|[i1, i2, i3]| {
                let (v1, v2, v3) = (
                    vertices[*i1 as usize],
                    vertices[*i2 as usize],
                    vertices[*i3 as usize],
                );
                let normal: Vector3<f32> = (v2 - v1).cross(v3 - v1).normalize().into();

                return normal;
            })
            .collect();

        return Model {
            vertices: vertices,
            normals: normals,
            faces: indices,
        };
    }
    /// creates a slice of a model at a given height (y)
    pub fn slice(&self, y: f32) -> Option<Slice> {
        let outline = self
            .faces
            .iter()
            .zip(&self.normals)
            .filter_map(|([i1, i2, i3], normal)| {
                let v1 = self.vertices[*i1 as usize];
                let v2 = self.vertices[*i2 as usize];
                let v3 = self.vertices[*i3 as usize];

                // the 3 lines of a triangle represented as start point and vector pointing to the end
                let (mut points, normals): (Vec<Point2<f32>>, Vec<Vector2<f32>>) =
                    [(v1, v2 - v1), (v2, v3 - v2), (v3, v1 - v3)]
                        .iter()
                        .filter_map(|(p, d)| {
                            if d.y == 0. {
                                return None;
                            } else {
                                let t: f32 = (y - p.y) / (d.y);
                                if t >= 0. && t <= 1. {
                                    // the intersection is within the start and end of the line
                                    let intsec = p + d * t;
                                    return Some((
                                        Point2::new(intsec.x, intsec.z),
                                        Vector2::new(normal.x, normal.z),
                                    ));
                                } else {
                                    // line does not intersect plane
                                    return None;
                                }
                            }
                        })
                        .unzip();

                if points.len() > 0 {
                    if points.len() != 2 {
                        // this is a special case, we have the same point two times
                        if points[0] == points[1] {
                            points = vec![points[0], points[2]];
                        } else if points[0] == points[2] {
                            points = vec![points[0], points[1]];
                        }
                    }
                    if points[0] == points[1] {
                        return None;
                    }
                    // Only return a polygon if the triangle intersects the plane
                    return Some(Polygon {
                        points: points,
                        normals: normals, // only take first normal
                    });
                } else {
                    return None;
                }
            })
            .collect::<Vec<Polygon>>();
        if outline.len() > 0 {
            return Some(outline);
        } else {
            return None;
        }
    }
}
