mod model;
mod polygons;

use cgmath::*;
use model::Model;
use polygons::{AbstractPolygon, IndexPolygon, Normal, Polygon, Vertex};
use std::fs::File;
use std::{fmt::Debug, io::prelude::*};

fn connect_lines(lines: &Vec<Polygon>) -> Vec<Polygon> {
    let mut vertices: Vec<Vertex> = Vec::new();
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

fn main() {
    let (p1, n1) = [
        ([0., 0.], [-1., 0.]),
        ([0., 20.], [0., 1.]),
        // ([10., 20.], [1., 0.]),
        // ([10., 10.], [0., 1.]),
        // ([20., 10.], [-1., 0.]),
        // ([20., 20.], [0., 1.]),
        ([30., 20.], [1., 0.]),
        ([30., 0.], [0., -1.]),
        ([0., 0.], [-1., 0.]),
    ]
    .iter()
    .map(|([x, y], [nx, ny])| (Point2::new(*x, *y), Vector2::new(*nx, *ny)))
    .unzip();

    let mut v1 = Polygon {
        points: p1,
        normals: n1,
    };

    let (p2, n2) = vec![
        ([-10., 15.], [-1., 0.]),
        ([-10., 30.], [0., 1.]),
        ([40., 30.], [1., 0.]),
        ([40., 15.], [0., -1.]),
        ([-10., 15.], [-1., 0.]),
    ]
    .iter()
    .map(|([x, y], [nx, ny])| (Point2::new(*x, *y), Vector2::new(*nx, *ny)))
    .unzip();

    let mut v2 = Polygon {
        points: p2,
        normals: n2,
    };

    let mut p1: usize = 0;
    let mut p2: usize = 1;
    while p1 < v1.len() {
        let mut p3: usize = 0;
        let mut p4: usize = 1;
        while p3 < v2.len() {
            if p3 < v2.len() && p4 < v2.len() {
                match polygons::get_line_intersection(
                    v1.get_point(p1),
                    v1.get_point(p2),
                    v2.get_point(p3),
                    v2.get_point(p4),
                ) {
                    Some(intersection) => {
                        v1.insert_point(p1 + 1, intersection, v1.get_normal(p1));
                        v2.insert_point(p3 + 1, intersection, v2.get_normal(p3));

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

    // we are assuming that the length of v1 and v2 is larger then 3
    let multi_graph = build_multi_graph(&v1, &v2);

    let mut union_staff: Vec<(Point2<f32>, Vector2<f32>)> =
        vec![(Point2::new(0., 0.), Vector2::new(-1., 0.))];
    for i in 0..8 {
        //while union_staff.len() == 1 || union_staff.last() != union_staff.first() {
        let l = union_staff.len();
        let (point, _) = union_staff.last().unwrap();
        let pos = v1
            .iter()
            .chain(v2.iter())
            .position(|(p, _): (&Vertex, &Normal)| *p == *point)
            .unwrap();

        let incoming_vertex = if l == 1 {
            union_staff[l - 1].0
        } else {
            union_staff[l - 2].0
        };

        let next: (Vertex, Normal) = if multi_graph[pos].len() == 1 {
            // return successor point
            multi_graph[pos][0]
        } else {
            *multi_graph[pos]
                .iter()
                .min_by_key(|(p, n)| {
                    let d_in =
                        Vector2::new(point.x - incoming_vertex.x, point.y - incoming_vertex.y)
                            .normalize();
                    let d_out = Vector2::new(p.x - point.x, p.y - point.y).normalize();

                    println!(
                        "({:?}) {:?} {:?} => {:?}",
                        p,
                        d_in,
                        d_out,
                        3.14 - (d_in.x * d_out.y - d_out.x * d_in.y)
                            .atan2(d_in.x * d_out.x + d_in.y * d_out.y)
                    );

                    return (3.14
                        - (d_in.x * d_out.y - d_out.x * d_in.y)
                            .atan2(d_in.x * d_out.x + d_in.y * d_out.y)
                            * 1000.) as i64;
                })
                .unwrap()
        };
        if multi_graph[pos].len() != 1 {
            println!("{:?}\n", multi_graph[pos]);
        }
        union_staff[l - 1].1.x = next.1.x;
        union_staff[l - 1].1.y = next.1.y;
        union_staff.push(next);
    }

    //println!("{:?}", union_staff);
    let mut layers: Vec<String> = Vec::new();

    // draw union
    layers.push(format!(
        "<g >{}</g>",
        union_staff
            .iter()
            .zip(union_staff.iter().skip(1))
            .map(|((p1, n1), (p2, n2))| to_line2((*p1, *n1), (*p2, *n2), "orange"))
            .collect::<Vec<String>>()
            .join("\n")
    ));

    //draw connections
    layers.push(format!(
        "<g opacity='0.1' >{}</g>",
        multi_graph
            .iter()
            .zip(v1.iter().chain(v2.iter()))
            .flat_map(|(n, (p1, n1))| n.iter().map(move |x| to_line2(*x, (*p1, *n1), "green")))
            .collect::<Vec<String>>()
            .join("\n")
    ));

    // draw first polygon
    // layers.push(
    //     format!(
    //         "<g >{}</g>",
    //         v1.iter().zip(v1.iter().skip(1))
    //         .map(|((p1,n1),(p2,n2))| to_line2((*p1,*n1),(*p2,*n2),"blue"))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    //     )
    // );
    // // draw second polygon
    // layers.push(
    //     format!(
    //         "<g >{}</g>",
    //         v2.iter().zip(v2.iter().skip(1))
    //         .map(|((p1,n1),(p2,n2))| to_line2((*p1,*n1),(*p2,*n2),"black"))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    //     )
    // );

    // layers.push(format!(
    //     "<g >{}</g>",
    //     [v1, v2]
    //         .iter()
    //         .map(|x| to_polygon(x))
    //         .collect::<Vec<String>>()
    //         .join("\n")
    // ));
    // layers.push(format!(
    //     "<g >{}</g>",
    //     intersections
    //         .iter()
    //         .map(|i| format!("<circle cx='{}' cy='{}' r='1' fill='red' />", i.x, i.y))
    //         .collect::<Vec<String>>()
    //         .join("/n")
    // ));
    let mut file = File::create("sliced.html").unwrap();
    file.write_all(
        format!(
            "
            <!DOCTYPE html>
            <html>
                <body>
                    <svg viewBox='-50 -50 100 100' height='800' width='800'>
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

fn build_multi_graph<T>(v1: &AbstractPolygon<T>, v2: &AbstractPolygon<T>) -> Vec<Vec<(T, Normal)>>
where
    T: std::fmt::Debug + PartialEq + Copy + Debug,
{
    fn next_prev<T>(i: usize, polygon: &AbstractPolygon<T>) -> Vec<(T, Normal)>
    where
        T: PartialEq + Copy,
    {
        let i_prev = if i == 0 {
            // since the first and last point are the same we add the second last one
            polygon.len() - 2
        } else {
            i - 1
        };

        let i_next = if i == polygon.len() - 1 { 1 } else { i + 1 };
        let p_point = polygon.get_pair(i_prev);
        let n_point = polygon.get_pair(i_next);
        vec![p_point, (n_point.0, polygon.normals[i])]
    };

    return [(v1, v2), (v2, v1)]
        .iter()
        .flat_map(
            |(v, v_other): &(&AbstractPolygon<T>, &AbstractPolygon<T>)| {
                v.iter().enumerate().map(move |(i, (p1, n1))| {
                    // intersection at this point with other polygon
                    let intersection = v_other.iter().position(|(p2, _)| *p2 == *p1);

                    let mut next_other = match intersection {
                        Some(j) => next_prev(j, *v_other),
                        None => vec![],
                    };
                    let this = next_prev(i, *v);
                    // skip first so you cant go back
                    next_other.extend(this.iter().skip(1));
                    return next_other;
                })
            },
        )
        .collect();
}

fn main_slice() {
    let model = Model::load("teapot.obj");
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
                                .map(|((p1, p2), v)| to_line(
                                    Point2::new(p1.x + p2.x, p1.y + p2.y) * 0.5,
                                    *v * 3.
                                ))
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

fn to_polygon(points: &Vec<Vertex>) -> String {
    let formatted: Vec<String> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
    return format!(
        "<polygon points='{}' style='fill:none;stroke:purple;stroke-width:0.1'/>",
        formatted.join(" ")
    );
}

fn to_line(p: Vertex, v: Normal) -> String {
    return format!(
        "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='red' stroke-width='0.1' />",
        p.x,
        p.y,
        p.x + v.x,
        p.y + v.y
    );
}

fn to_line2((p1, n1): (Vertex, Normal), (p2, _): (Vertex, Normal), color: &str) -> String {
    let center = Point2::new((p1.x + p2.x) / 2., (p1.y + p2.y) / 2.);
    return format!(
        "<line x1='{}' y1='{}' x2='{}' y2='{}' stroke='{color}' stroke-width='0.1' />
        <line x1='{}' y1='{}' x2='{}' y2='{}' stroke='red' stroke-width='0.1' />"
        //<circle cx='{}' cy='{}' r='1' fill='red' />
        ,//",
        p1.x,
        p1.y,
        p2.x,
        p2.y,
        center.x,
        center.y,
        center.x + n1.x,
        center.y + n1.y,
        // (p1.x + 3. * p2.x) / 4.,
        // (p1.y + 3. * p2.y) / 4.,
        color = color,
    );
}
