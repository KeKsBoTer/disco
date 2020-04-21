use cgmath::*;
use cgmath::{Point2, Point3};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tobj;
mod util;

fn slice_model(model: Vec<[Point3<f32>; 3]>, y: f32) -> Vec<Point2<f32>> {
    //let mut collection: Vec<Point2<f32>> = Vec::with_capacity(0);
    let collection = model
        .iter()
        .map(|[v1, v2, v3]| {
            // the 3 lines of a triangle representated as start point and vector pointing to the end
            return[(v1, v2 - v1), (v2, v3 - v2), (v3, v1 - v3)]
                .iter()
                .filter_map(|(p, d)| {
                    if d.y == 0. {
                        // line lies on a plane parallel to the xz plane
                        if p.y == y {
                            // line lies exactly on plane
                            return Some(vec![
                                Point2::new(p.x, p.z),
                                Point2::new(p.x + d.x, p.z + d.z),
                            ]);
                        } else {
                            // line does not intersect plane
                            return None;
                        }
                    } else {
                        let t: f32 = (y - p.y) / (d.y);
                        if t >= 0. && t <= 1. {
                            // the intersection is within the start and end of the line
                            let intsec = (*p) + d * t;
                            return Some(vec![Point2::new(intsec.x, intsec.z)]);
                        } else {
                            // line does not intersect plane
                            return None;
                        }
                    }
                })
                .flatten()
                .collect();
        })
        .filter(|mesh:&Vec<Point2<f32>>| mesh.len() > 0)
        .flatten()
        .collect();
    return collection;
}

fn pack_slice(vertices: Vec<Point2<f32>>) -> Vec<Vec<Point2<f32>>> {
    // convert points into list of indices => group similar points
    let mut indices: Vec<Point2<f32>> = Vec::new();
    let points = vertices
        .iter()
        .map(|p| {
            let hit = indices
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
                    indices.push(*p);
                    return indices.len() - 1;
                }
            }
        })
        .collect::<Vec<usize>>();

    let mut lines: Vec<Vec<usize>> = points.chunks(2).map(|x| vec![x[0], x[1]]).collect();
    let mut new = true;
    while new {
        new = false;
        let mut i = 0;
        while i < lines.len() {
            let mut j = i + 1;
            while j < lines.len() {
                let l1 = lines[i].clone();
                let l2 = lines[j].clone();
                if l1[0] == l2[0] {
                    // first points of each lines are the same
                    lines.remove(j);
                    lines[i] = l1
                        .iter()
                        .skip(1)
                        .rev()
                        .chain(l2.iter())
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[l1.len() - 1] == l2[l2.len() - 1] {
                    // last points of each lines are the same
                    lines.remove(j);
                    lines[i] = l1
                        .iter()
                        .chain(l2.iter().rev().skip(1))
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[l1.len() - 1] == l2[0] {
                    // last point of first line and the first of the second one are the same
                    lines.remove(j);
                    lines[i] = l1
                        .iter()
                        .chain(l2.iter().skip(1))
                        .map(|x| *x)
                        .collect::<Vec<usize>>();
                    new = true;
                } else if l1[0] == l2[l2.len() - 1] {
                    // first point of first line and the last of the second one are the same
                    lines.remove(j);
                    lines[i] = l2
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
    let line_vertices: Vec<Vec<Point2<f32>>> = lines
        .iter()
        //.filter(|l| l.len() > 2) // remove all lines (they do not cover a surface)
        .map(|l| l.iter().map(|x| indices[*x]).collect::<Vec<Point2<f32>>>())
        .collect();
    return line_vertices;
}

fn main() {
    let model = load_model("teapot.obj");
    let mut layers: Vec<String> = Vec::new();
    for z in 0..100 {
        let outline = slice_model(model.clone(), z as f32 - 40.);

        let polygons = pack_slice(outline.clone());
        layers.push(format!(
            "<g id='slice_{}'>{}</g>",
            z,
            polygons
                .iter()
                .map(|x| to_polygon(x))
                .collect::<Vec<String>>()
                .join("\n")
        ))
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
        "<polygon points='{}' style='fill:lime;stroke:purple;stroke-width:0.1'/>",
        formatted.join(" ")
    );
}

fn load_model(file: &str) -> Vec<[Point3<f32>; 3]> {
    let cornell_box = tobj::load_obj(&Path::new(&file));
    assert!(cornell_box.is_ok());

    let (models, _) = cornell_box.unwrap();
    //let mut vertices: Vec<[Point3<f32>; 3]> = Vec::with_capacity(0);
    let vertices = models
        .iter()
        .map(|m| {
            let positions: Vec<&[f32]> = m.mesh.positions.chunks(3).collect();
            let indices = m.mesh.indices.as_slice();
            return indices.chunks(3).map(move |i| {
                [
                    Point3::new(
                        positions[i[0] as usize][0],
                        positions[i[0] as usize][1],
                        positions[i[0] as usize][2],
                    ),
                    Point3::new(
                        positions[i[1] as usize][0],
                        positions[i[1] as usize][1],
                        positions[i[1] as usize][2],
                    ),
                    Point3::new(
                        positions[i[2] as usize][0],
                        positions[i[2] as usize][1],
                        positions[i[2] as usize][2],
                    ),
                ]
            });
        })
        .flatten()
        .collect::<Vec<[Point3<f32>; 3]>>();

    return vertices;
}
