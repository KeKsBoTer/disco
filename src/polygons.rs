use cgmath::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AbstractPolygon<T> {
    /// list of n points
    pub points: Vec<T>,
    /// list of n normals, one for each line
    pub normals: Vec<Vector2<f32>>,
}

pub type Polygon = AbstractPolygon<Point2<f32>>;

impl Polygon {
    pub fn to_indices(&self, vertices: &mut Vec<Point2<f32>>) -> IndexPolygon {
        let indices = self
            .points
            .iter()
            .map(|p| {
                let hit = vertices
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| (**x - p).magnitude() < 0.01) // smaler than 0.1 mm
                    .map(|(i, _)| i)
                    .next();
                match hit {
                    Some(h) => {
                        return h;
                    }
                    None => {
                        vertices.push(*p);
                        return vertices.len() - 1;
                    }
                }
            })
            .collect::<Vec<usize>>();
        return IndexPolygon {
            points: indices,
            normals: self.normals.clone(),
        };
    }
}

pub type IndexPolygon = AbstractPolygon<usize>;

impl IndexPolygon {
    pub fn to_polygon(&self, vertices: Vec<Point2<f32>>) -> Polygon {
        return Polygon {
            points: self
                .points
                .iter()
                .map(|x| vertices[*x])
                .collect::<Vec<Point2<f32>>>(),
            normals: self.normals.clone(),
        };
    }

    pub fn fuse_normals(&self) -> IndexPolygon {
        let n_points: Vec<(usize, Vector2<f32>)> = self
            .points
            .iter()
            .zip(self.normals.iter())
            .map(|(p, n)| (*p, *n))
            .collect();
        let f_points: Vec<(usize, Vector2<f32>)> =
            n_points
                .iter()
                .fold(Vec::new(), |acc: Vec<(usize, Vector2<f32>)>, (p, n)| {
                    let a: Vec<(usize, Vector2<f32>)> = vec![(*p, *n)];
                    let last = acc.last();
                    if let Some(last_item) = last {
                        if last_item.1 == *n {
                            println!("{:?}, {:?}", acc, a);
                            return acc.iter().map(|(a, b)| (*a, *b)).chain(a).collect();
                        }
                    }
                    let a: Vec<(usize, Vector2<f32>)> = vec![(*p, *n)];
                    return acc.iter().map(|(a, b)| (*a, *b)).chain(a).collect();
                });
        let (points, normals): (Vec<usize>, Vec<Vector2<f32>>) =
            f_points.iter().map(|(p, n)| (*p, *n)).unzip();

        return IndexPolygon {
            points: points,
            normals: normals,
        };
    }

    pub fn join(&self, l2: IndexPolygon) -> Option<IndexPolygon> {
        if self.points[0] == l2.points[0] {
            // first points of each lines are the same
            return Some(IndexPolygon {
                points: self
                    .points
                    .iter()
                    .skip(1)
                    .rev()
                    .chain(l2.points.iter())
                    .map(|x| *x)
                    .collect(),
                normals: self
                    .normals
                    .iter()
                    .skip(1)
                    .rev()
                    .chain(l2.normals.iter())
                    .map(|x| *x)
                    .collect(),
            });
        } else if self.points[self.points.len() - 1] == l2.points[l2.points.len() - 1] {
            // last points of each lines are the same
            return Some(IndexPolygon {
                points: self
                    .points
                    .iter()
                    .chain(l2.points.iter().rev().skip(1))
                    .map(|x| *x)
                    .collect(),
                normals: self
                    .normals
                    .iter()
                    .chain(l2.normals.iter().rev().skip(1))
                    .map(|x| *x)
                    .collect(),
            });
        } else if self.points[self.points.len() - 1] == l2.points[0] {
            // last point of first line and the first of the second one are the same
            return Some(IndexPolygon {
                points: self
                    .points
                    .iter()
                    .chain(l2.points.iter().skip(1))
                    .map(|x| *x)
                    .collect(),
                normals: self
                    .normals
                    .iter()
                    .chain(l2.normals.iter().skip(1))
                    .map(|x| *x)
                    .collect(),
            });
        } else if self.points[0] == l2.points[l2.points.len() - 1] {
            // first point of first line and the last of the second one are the same
            return Some(IndexPolygon {
                points: l2
                    .points
                    .iter()
                    .chain(self.points.iter().skip(1))
                    .map(|x| *x)
                    .collect(),
                normals: l2
                    .normals
                    .iter()
                    .chain(self.normals.iter().skip(1))
                    .map(|x| *x)
                    .collect(),
            });
        } else {
            return None;
        }
    }
}
