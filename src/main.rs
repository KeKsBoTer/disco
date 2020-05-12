use crate::polygons::{IndexPolygon, Polygon};
use cgmath::*;
use model::Model;
use std::fs::File;
use std::io::prelude::*;
mod model;
mod polygons;

fn connect_lines(lines: &Vec<Polygon>) -> Vec<Polygon> {
    let mut vertices: Vec<Point2<f32>> = Vec::new();
    // convert points into list of indices => group similar points
    let mut polygons: Vec<IndexPolygon> = lines
        .iter()
        // convert vectors to indices
        .map(|p| p.to_indices(&mut vertices))
        .collect::<Vec<IndexPolygon>>();

    // remove duplicates
    polygons.dedup_by(|p1, p2| p1.points == p2.points);

    // combine lines into closed polygons
    // this is done by iteratively joining lines together until no new lines can be joined
    let mut new = true;
    while new {
        new = false;
        let mut i = 0;
        while i < polygons.len() {
            let mut j = i + 1;
            while j < polygons.len() {
                let l1 = polygons[i].clone();
                let l2 = polygons[j].clone();
                match l1.join(l2) {
                    Some(result) => {
                        polygons.remove(j);
                        polygons[i] = result;
                        new = true;
                    }
                    None => {
                        j += 1;
                    }
                }
            }
            i += 1;
        }
    }
    // convert the indices pack to actual vertices
    let line_vertices: Vec<Polygon> = polygons
        .iter()
        .filter(|l| l.points.len() > 2)
        //.map(|l|l.fuse_normals()) // TODO
        .map(|l| l.to_polygon(vertices.clone()))
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

                        // inserted new, hence skip
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
                    <svg viewBox='-100 -100 100 100' height='500' width='500'>
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
    let model = Model::load("cube.obj");
    let mut layers: Vec<String> = Vec::new();
    for z in 0..100 {
        if let Some(outline) = model.slice(z as f32 - 40.) {
            let polygons = connect_lines(&outline);
            layers.push(format!(
                "<g id='slice_{}'>{}</g>",
                z,
                polygons
                    .iter()
                    .map(|x| to_polygon(&x.points))
                    .chain(
                        polygons
                            .iter()
                            .map(|poly| poly
                                .points
                                .iter()
                                .zip(poly.points.iter().skip(1))
                                .zip(poly.normals.iter())
                                .map(|((p1, _), v)| to_line(*p1, *v * 3.))
                                .collect::<Vec<String>>())
                            .flatten()
                    )
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }
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
            </svg>
            <input type='range' min='0' max='99' value='0' class='slider' id='range'>
            <script>
            var slider = document.getElementById('range');
            let slices = document.querySelectorAll('[id^=\"slice_\"]');
            let min = parseInt(slices[0].id.substr(6));
            slider.min = min;
            slider.max = min+slices.length;
            slider.oninput = function() {{
                for(var i=min;i<(min+slices.length);i++){{
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
