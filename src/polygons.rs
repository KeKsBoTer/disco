use cgmath::*;

pub type Vertex = Point2<f32>;
pub type Normal = Vector2<f32>;

#[derive(Debug, Clone, PartialEq)]
pub struct AbstractPolygon<T> where T:PartialEq+Copy{
    /// list of n points
    pub points: Vec<T>,
    /// list of n normals, one for each line
    pub normals: Vec<Normal>,
}

impl<T> AbstractPolygon<T>  where T:PartialEq+Copy{
    pub fn len(&self) -> usize {
        return self.points.len();
    }

    pub fn get_point(&self,i: usize) -> T{
        return self.points[i];
    }
    
    pub fn get_normal(&self,i: usize) -> Normal{
        return  self.normals[i];
    }

    pub fn get_pair(&self,i: usize) -> (T,Normal){
        return  (self.points[i],self.normals[i]);
    }

    pub fn insert_point(&mut self,i: usize,point: T, normal: Normal){
        self.points.insert(i, point);
        self.normals.insert(i, normal);
    }

    pub fn iter(&self) -> std::iter::Zip<std::slice::Iter<'_, T>, std::slice::Iter<'_, Normal>>{
        return self.points.iter().zip(self.normals.iter())
    }
}

pub type Polygon = AbstractPolygon<Vertex>;

impl Polygon {
    pub fn to_indices(&self, vertices: &mut Vec<Vertex>) -> IndexPolygon {
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

    pub fn union(&self, other: Polygon) -> Vec<Polygon> {
        let mut v1: Polygon = self.clone();
        let mut v2: Polygon = other.clone();

        let mut p1: usize = 0;
        let mut p2: usize = 1;
        while p1 < v1.len() {
            let mut p3: usize = 0;
            let mut p4: usize = 1;
            while p3 < v2.len() {
                if p3 < v2.len() && p4 < v2.len() {
                    match get_line_intersection(
                        v1.points[p1],
                        v1.points[p2],
                        v2.points[p3],
                        v2.points[p4],
                    ) {
                        Some(i) => {
                            //TODO insert does not work, solve it another way
                            v1.points.insert(p1 + 1, i);
                            v2.points.insert(p3 + 1, i);

                            // inserted new, hence skip
                            p1 += 1;
                            p2 += 1;
                            p3 += 1;
                            p4 += 1;
                        }
                        None => (),
                    }
                }
                p3 += 1;
                p4 = if p4 == v2.len() - 1 { 0 } else { p4 + 1 };
            }
            p1 += 1;
            p2 = if p2 == v1.len() - 1 { 0 } else { p1 + 1 };
        }
        return vec![v1, v2];
    }
}

pub type IndexPolygon = AbstractPolygon<usize>;

impl IndexPolygon {
    pub fn to_polygon(&self, vertices: Vec<Vertex>) -> Polygon {
        return Polygon {
            points: self
                .points
                .iter()
                .map(|x| vertices[*x])
                .collect::<Vec<Vertex>>(),
            normals: self.normals.clone(),
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
                    .chain(l2.normals.iter().rev())
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
                    .chain(l2.normals.iter())
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
                    .chain(self.normals.iter())
                    .map(|x| *x)
                    .collect(),
            });
        } else {
            return None;
        }
    }
}

pub fn get_line_intersection(
    p1: Vertex,
    p2: Vertex,
    p3: Vertex,
    p4: Vertex,
) -> Option<Vertex> {
    // source: https://stackoverflow.com/a/1968345
    let s1 = p2 - p1;
    let s2 = p4 - p3;

    let d = -s2.x * s1.y + s1.x * s2.y;

    let s = (-s1.y * (p1.x - p3.x) + s1.x * (p1.y - p3.y)) / d;
    let t = (s2.x * (p1.y - p3.y) - s2.y * (p1.x - p3.x)) / d;

    if s >= 0. && s <= 1. && t >= 0. && t <= 1. {
        return Some(p1 + (p2 - p1) * t);
    }

    return None;
}
