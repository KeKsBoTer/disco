use cgmath::*;
use cgmath::{Point2, Point3, Vector2};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tobj::{load_obj, Model as TobjModel};

/// converts a list of vertices into a list of indices to group similar points
fn vertices_to_indices(vertices: Vec<Point2<f32>>) -> (Vec<usize>, Vec<Point2<f32>>) {
    let mut points: Vec<Point2<f32>> = Vec::new();
    let indices = vertices
        .iter()
        .map(|p| {
            let hit = points
                .iter()
                .enumerate()
                .filter(|(_, x)| (**x - p).magnitude() < 0.01) // smaler than 0.1 mm
                .map(|(i, _)| i)
                .next();
            match hit {
                Some(h) => {
                    return h;
                }
                _ => {
                    points.push(*p);
                    return points.len() - 1;
                }
            }
        })
        .collect::<Vec<usize>>();
    return (indices, points);
}

fn connect_lines(lines: &Vec<Polygon>) -> Vec<Polygon> {
    let vertices: Vec<Point2<f32>> = lines.iter().map(|x| x.points.clone()).flatten().collect();
    // convert points into list of indices => group similar points
    let (points, indices) = vertices_to_indices(vertices);

    let mut polygons: Vec<Vec<usize>> = points.chunks(2).map(|x| vec![x[0], x[1]]).collect();

    // combine lines into closed polygons
    // this is done by iterativly joining lines together until no new lines can be joined
    let mut new = true;
    while new {
        new = false;
        let mut i = 0;
        while i < polygons.len() {
            let mut j = i + 1;
            while j < polygons.len() {
                let l1 = polygons[i].clone();
                let l2 = polygons[j].clone();
                if l1[0] == l2[0] {
                    // first points of each lines are the same
                    polygons.remove(j);
                    polygons[i] = l1
                        .iter()
                        .skip(1)
                        .rev()
                        .chain(l2.iter())
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[l1.len() - 1] == l2[l2.len() - 1] {
                    // last points of each lines are the same
                    polygons.remove(j);
                    polygons[i] = l1
                        .iter()
                        .chain(l2.iter().rev().skip(1))
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[l1.len() - 1] == l2[0] {
                    // last point of first line and the first of the second one are the same
                    polygons.remove(j);
                    polygons[i] = l1
                        .iter()
                        .chain(l2.iter().skip(1))
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[0] == l2[l2.len() - 1] {
                    // first point of first line and the last of the second one are the same
                    polygons.remove(j);
                    polygons[i] = l2
                        .iter()
                        .chain(l1.iter().skip(1))
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }
    // convert the indices pack to actual vertices
    let line_vertices: Vec<Polygon> = polygons
        .iter()
        .map(|l| Polygon {
            points: l.iter().map(|x| indices[*x]).collect::<Vec<Point2<f32>>>(),
            normals: Vec::new(),
        })
        .collect();
    return line_vertices;
}
fn get_line_intersection(
    p1: Point2<f32>,
    p2: Point2<f32>,
    p3: Point2<f32>,
    p4: Point2<f32>,
) -> Option<Point2<f32>> {
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

fn main_join() {
    let mut v1: Vec<Point2<f32>> = [
        [0., 0.],
        [0., 20.],
        [10., 20.],
        [10., 10.],
        [20., 10.],
        [20., 20.],
        [30., 20.],
        [30., 0.],
    ]
    .iter()
    .map(|[x, y]| Point2::new(*x, *y))
    .collect();

    let mut v2: Vec<Point2<f32>> = vec![[-10., 15.], [-10., 30.], [40., 30.], [40., 15.]]
        .iter()
        .map(|[x, y]| Point2::new(*x, *y))
        .collect();

    let mut cuts_1: Vec<usize> = Vec::new();
    let mut cuts_2: Vec<usize> = Vec::new();
    let mut intersections: Vec<Point2<f32>> = Vec::new();

    let mut p1: usize = 0;
    let mut p2: usize = 1;
    while p1 < v1.len() {
        let mut p3: usize = 0;
        let mut p4: usize = 1;
        while p3 < v2.len() {
            if p3 < v2.len() && p4 < v2.len() {
                match get_line_intersection(v1[p1], v1[p2], v2[p3], v2[p4]) {
                    Some(i) => {
                        intersections.push(i);
                        //TODO insert does not work, solve it another way
                        v1.insert(p1 + 1, i);
                        v2.insert(p3 + 1, i);

                        // inserted new, hence skipp
                        p1 += 1;
                        p2 += 1;
                        p3 += 1;
                        p4 += 1;
                        cuts_1.push(p1);
                        cuts_2.push(p3);
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
    println!("{:?}", intersections);

    println!("{:?} {:?}", cuts_1, cuts_2);

    let mut layers: Vec<String> = Vec::new();
    layers.push(format!(
        "<g >{}</g>",
        [v1, v2]
            .iter()
            .map(|x| to_polygon(x))
            .collect::<Vec<String>>()
            .join("\n")
    ));
    layers.push(format!(
        "<g >{}</g>",
        intersections
            .iter()
            .map(|i| format!("<circle cx='{}' cy='{}' r='1' fill='red' />", i.x, i.y))
            .collect::<Vec<String>>()
            .join("/n")
    ));
    let mut file = File::create("sliced.html").unwrap();
    file.write_all(
        format!(
            "
    <!DOCTYPE html>
    <html>
        <body>
            <svg viewBox='-100 -100 200 200' height='500' width='500'>
                {}
            </svg>
        </body>
    </html>
    
    ",
            layers.join("\n")
        )
        .as_bytes(),
    )
    .unwrap();
}

fn main() {
    let model = load_model("teapot.obj");
    let mut layers: Vec<String> = Vec::new();
    for z in 0..100 {
        let outline = model.slice(z as f32 - 40.);

        let polygons = connect_lines(&outline);
        layers.push(format!(
            "<g id='slice_{}'>{}</g>",
            z,
            polygons
                .iter()
                .map(|x| to_polygon(&x.points))
                .chain(
                    outline.iter().map(|poly|
                        poly.points.iter()
                        .zip(poly.normals.iter()).map(|(p1,v)|to_line(*p1, *v*5.)).collect::<Vec<String>>()
                    ).flatten()
                )
                .collect::<Vec<String>>()
                .join("\n")
        ));
    }
    let mut file = File::create("sliced.html").unwrap();
    file.write_all(
        format!(
            "
    <!DOCTYPE html>
    <html>
        <body>
            <svg viewBox='-100 -100 200 200' height='500' width='500'>
                {}
            </g>
            </svg>
            <input type='range' min='0' max='99' value='0' class='slider' id='range'>
            <script>
            var slider = document.getElementById('range');
            slider.oninput = function() {{
                for(var i=0;i<1000;i++){{
                    var elem = document.getElementById('slice_'+i);
                    if(!elem)
                        break;
                    elem.style.visibility = i== slider.value ? 'visible' : 'hidden';
                }}
            }}
            </script>
        </body>
    </html>

    ",
            layers.join("\n")
        )
        .as_bytes(),
    )
    .unwrap();
}

fn to_polygon(points: &Vec<Point2<f32>>) -> String {
    let formatted: Vec<String> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
    return format!(
        "<polygon points='{}' style='fill:none;stroke:purple;stroke-width:0.1'/>",
        formatted.join(" ")
    );
}
fn to_line(p: Point2<f32>, v: Vector2<f32>) -> String {
    return format!(
        "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='red' stroke-width='0.1' />",
        p.x,
        p.y,
        p.x + v.x,
        p.y + v.y
    );
}

#[derive(Debug)]
struct Polygon {
    points: Vec<Point2<f32>>,
    normals: Vec<Vector2<f32>>,
}

type Slice = Vec<Polygon>;

#[derive(Debug)]
struct Model {
    vertices: Vec<Point3<f32>>,
    normals: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
}

impl Model {
    /// creates a slice of a model at a given height (y)
    pub fn slice(&self, y: f32) -> Slice {
        return self
            .faces
            .iter()
            .zip(&self.normals)
            .map(|([i1, i2, i3], normal)| {
                let v1 = self.vertices[*i1 as usize];
                let v2 = self.vertices[*i2 as usize];
                let v3 = self.vertices[*i3 as usize];

                // the 3 lines of a triangle representated as start point and vector pointing to the end
                let polygons: Vec<Polygon> = [(v1, v2 - v1), (v2, v3 - v2), (v3, v1 - v3)]
                    .iter()
                    .filter_map(|(p, d)| {
                        if d.y == 0. {
                            // line lies on a plane parallel to the xz plane
                            if p.y == y {
                                // line lies exactly on plane
                                return Some(Polygon {
                                    points: vec![
                                        Point2::new(p.x, p.z),
                                        Point2::new(p.x + d.x, p.z + d.z),
                                    ],
                                    normals: vec![Vector2::new(normal.x, normal.z).normalize().into()],
                                });
                            } else {
                                // line does not intersect plane
                                return None;
                            }
                        } else {
                            let t: f32 = (y - p.y) / (d.y);
                            if t >= 0. && t <= 1. {
                                // the intersection is within the start and end of the line
                                let intsec = (*p) + d * t;
                                return Some(Polygon {
                                    points: vec![Point2::new(intsec.x, intsec.z)],
                                    normals: vec![Vector2::new(normal.x, normal.z).normalize().into()],
                                });
                            } else {
                                // line does not intersect plane
                                return None;
                            }
                        }
                    })
                    .collect();
                return polygons;
            })
            .flatten()
            .collect::<Vec<Polygon>>();
    }
}

fn load_model(file: &str) -> Model {
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
            let normal: Vector3<f32> = (v1 - v2).cross(v3 - v1).normalize().into();

            return normal;
        })
        .collect();

    return Model {
        vertices: vertices,
        normals: normals,
        faces: indices,
    };
}
