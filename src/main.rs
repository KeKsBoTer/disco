use cgmath::*;
use cgmath::{dot, Point2, Point3, Vector3};
use std::path::Path;
use tobj;
use std::fs::File;
use std::io::prelude::*;

fn slice_model(model: &Vec<[Point3<f32>; 3]>, z:f32) -> Vec<Point2<f32>>{
    let mut outline: Vec<Point2<f32>> = Vec::with_capacity(0);
    for v in model.iter() {
        let lines = [
            (v[0], v[1] - v[0]),
            (v[1], v[2] - v[1]),
            (v[2], v[0] - v[2]),
        ];
        for (p, d) in lines.iter() {
            let t = (z - p.y) / (d.y);
            if t >= 0. && t <= 1. {
                let intsec = p+d*t;
                if intsec.x == 0. && intsec.z == 0.0{
                    println!("{:?}:   {:?} {:?}",v, p,d)
                }
                outline.push(Point2::new(intsec.x,intsec.z))
            }
        }
    }
    return outline
}

fn main() {
    let model = load_model("teapot.obj");
    let outline = slice_model(&model,0.);

    let mut file = File::create("test.html").unwrap();
    file.write_all(format!("
    <!DOCTYPE html>
    <html>
        <body>
            <svg viewBox='-100 -100 200 200' height='500' width='500'>
                {}
            </svg> 
        </body>
    </html>
    
    ",to_lines(&outline).join("\n")/*polygons.iter().map(|x| to_polygon(x)).collect::<Vec<String>>().join("\n")*/).as_bytes()).unwrap();
}


fn to_lines(points: &Vec<Point2<f32>>)-> Vec<String>{
    return points
    .chunks(2)
    .map(|p|format!("<line x1='{}' y1='{}' x2='{}' y2='{}' style='stroke:rgb(255,0,0)'/>",p[0].x,p[0].y,p[1].x,p[1].y))
    .collect()
}

fn to_polygon(points: &Vec<Point2<f32>>)-> String{
    let formatted : Vec<String> = points.iter().map(|p|format!("{},{}", p.x,p.y)).collect();
    return format!("<polygon points='{}' style='fill:lime;stroke:purple;stroke-width:1'/>",formatted.join(" "))
}

fn load_model(file: &str) -> Vec<[Point3<f32>; 3]> {
    let cornell_box = tobj::load_obj(&Path::new(&file));
    assert!(cornell_box.is_ok());

    let (models, _) = cornell_box.unwrap();
    let mut vertices: Vec<[Point3<f32>; 3]> = Vec::with_capacity(0);
    for m in models {
        let positions: Vec<&[f32]> = m.mesh.positions.chunks(3).collect();
        let indices = m.mesh.indices.as_slice();

        for i in indices.chunks(3) {
            vertices.push([
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
            ])
        }
    }

    return vertices;
}