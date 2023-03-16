use image::{RgbImage, Rgb};
use imageproc::drawing;

use crate::face::Face;

pub fn draw_face(image: &mut RgbImage, face: &Face) {
    drawing::draw_hollow_rect_mut(image, face.region.into(), Rgb([0, 0, 255]));

    for (_i, point) in face.shape.iter().enumerate() {
        drawing::draw_cross_mut(image, Rgb([0, 255, 0]), point.x as i32, point.y as i32);
    }

    drawing::draw_cross_mut(
        image,
        Rgb([255, 0, 0]),
        face.pupils.0.x as i32,
        face.pupils.0.y as i32,
    );
    drawing::draw_cross_mut(
        image,
        Rgb([255, 0, 0]),
        face.pupils.1.x as i32,
        face.pupils.1.y as i32,
    );
}

pub fn print_faces_data(faces: &[Face]) {
    println!("Faces detected: {}.", faces.len());
    for (i, face) in faces.iter().enumerate() {
        print!("{} ::\t", i);
        println!("location: {:?}", &face.region);
        print!("\t");
        println!("score: {}", face.score);

        for (i, point) in face.shape.iter().enumerate() {
            print!("\t");
            println!("point {}: {}", i, &point);
        }

        print!("\t");
        println!("left  pupil: {}", &face.pupils.0);
        print!("\t");
        println!("right pupil: {}", &face.pupils.1);
        print!("\n");
    }
}
