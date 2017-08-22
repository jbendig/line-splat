extern crate line_splat;
#[macro_use(arg_enum,value_t,_clap_count_exprs)]
extern crate clap;
extern crate image;
extern crate rand;

use std::path::Path;
use std::str::FromStr;

use clap::{App,Arg};
use image::ImageBuffer;
use rand::{Closed01,Open01,Rand,Rng,ThreadRng};
use rand::distributions::{IndependentSample,Normal};

use line_splat::imageprocessing::{gradient,angle_to_direction,non_maximum_suppression};
use line_splat::painter::Painter;
use line_splat::utility::{clamp_to_u8,min_f32,max_f32,difference_theta,mix,rgb_to_hsl,hsl_to_rgb};

arg_enum! {
    enum Style {
        Random,
        Steered,
        Energy,
        EdgeWeb
    }
}

//Sample a 3x3 region at point and return the average color.
fn color_at(buffer: &[u8],width: u32,height: u32,x: usize,y: usize) -> (u8,u8,u8) {
    assert!(x < width as usize);
    assert!(y < height as usize);

    let min_x = if x == 0 { x } else { x - 1 };
    let min_y = if y == 0 { y } else { y - 1 };
    let max_x = if x == (width as usize - 1) { x } else { x + 1 };
    let max_y = if y == (height as usize - 1) { y } else { y + 1 };

    let mut total = 0.0;
    let (mut red_sum,mut green_sum,mut blue_sum) = (0.0,0.0,0.0);
    for y in min_y..(max_y + 1) {
        for x in min_x..(max_x + 1) {
            let index = (y * width as usize + x) * 3;
            red_sum += buffer[index + 0] as f32;
            green_sum += buffer[index + 1] as f32;
            blue_sum += buffer[index + 2] as f32;
            total += 1.0;
        }
    }

    assert!(total != 0.0);
    ((red_sum / total) as u8,(green_sum / total) as u8,(blue_sum / total) as u8)
}

fn shift_color(rng: &mut ThreadRng,red: u8,green: u8,blue: u8) -> (u8,u8,u8) {
    const STD_DEV: f64 = 10.0;

    let (red,green,blue) = (red as f64,green as f64,blue as f64);
    let (red,green,blue) = (Normal::new(red,STD_DEV).ind_sample(rng),
                            Normal::new(green,STD_DEV).ind_sample(rng),
                            Normal::new(blue,STD_DEV).ind_sample(rng));

    (clamp_to_u8(red),clamp_to_u8(green),clamp_to_u8(blue))
}

fn shift_lightness(rng: &mut ThreadRng,red: u8,green: u8,blue: u8) -> (u8,u8,u8) {
    const STD_DEV: f64 = 0.03;

    let (h,s,l) = rgb_to_hsl(red,green,blue);
    let l = Normal::new(l as f64,STD_DEV).ind_sample(rng) as f32;
    let l = max_f32(min_f32(l,1.0),0.0);

    hsl_to_rgb(h,s,l)
}

fn random_line(rng: &mut ThreadRng,width: u32,height: u32) -> (usize,usize,usize,usize) {
    const DISTANCE_MAX: f32 = 128.0;

    let x1 = rng.gen::<usize>() % width as usize;
    let y1 = rng.gen::<usize>() % height as usize;

    loop {
        let angle = Closed01::<f32>::rand(rng).0 * std::f32::consts::PI * 2.0;
        let distance = Open01::<f32>::rand(rng).0 * DISTANCE_MAX;

        let x2 = (x1 as f32 + distance * angle.cos()) as usize;
        let y2 = (y1 as f32 + distance * angle.sin()) as usize;

        if x2 < width as usize && y2 < height as usize {
            return (x1,y1,x2,y2);
        }
    }
}

fn random_steered_line(rng: &mut ThreadRng,gradient: &[f32],width: u32,height: u32) -> (usize,usize,usize,usize) {
    const DISTANCE_MAX: f32 = 64.0;

    loop {
        let x1 = rng.gen::<usize>() % width as usize;
        let y1 = rng.gen::<usize>() % height as usize;

        let index = (y1 * width as usize + x1) * 2;
        let angle = gradient[index + 1] + std::f32::consts::PI / 2.0;
        let distance = Open01::<f32>::rand(rng).0 * DISTANCE_MAX;

        let x2 = (x1 as f32 + distance * angle.cos()) as usize;
        let y2 = (y1 as f32 + distance * angle.sin()) as usize;

        if x2 < width as usize && y2 < height as usize {
            return (x1,y1,x2,y2);
        }
    }
}

fn random_energy_line(rng: &mut ThreadRng,gradient: &[f32],width: u32,height: u32) -> (usize,usize,usize,usize,usize,usize) {
    const ENERGY_MIN: f32 = 10.0;
    const ENERGY_MAX: f32 = 80.0;
    const ENERGY_DIFF: f32 = ENERGY_MAX - ENERGY_MIN;

    let xc = rng.gen::<usize>() % width as usize;
    let yc = rng.gen::<usize>() % height as usize;
    let angle = Closed01::<f32>::rand(rng).0 * std::f32::consts::PI * 2.0;

    let mut fire_ray = |x,y,angle: f32| -> (usize,usize) {
        let mut energy = Open01::<f32>::rand(rng).0 * ENERGY_DIFF + ENERGY_MIN;

        let xe = (x as f32 + energy * angle.cos()) as i32;
        let ye = (y as f32 + energy * angle.sin()) as i32;

        let mut last_x = x;
        let mut last_y = y;
        Painter::line_foreach(width as usize,height as usize,xc as i32,yc as i32,xe,ye,|x,y| {
            if energy >= 0.0 {
                let gradient_index = (y * width as usize + x) * 2;
                let mut dampening = 1.0 - difference_theta(angle,gradient[gradient_index + 1]) / (std::f32::consts::PI);
                dampening *= Open01::<f32>::rand(rng).0;
                energy -= gradient[gradient_index] * dampening;
                last_x = x;
                last_y = y;
            }
        });

        (last_x,last_y)
    };

    let (x1,y1) = fire_ray(xc,yc,angle);
    let (x2,y2) = fire_ray(xc,yc,angle + std::f32::consts::PI);

    (xc,yc,x1,y1,x2,y2)
}

fn edge_web(rng: &mut ThreadRng,source_image_pixels: &[u8],source_image_width: u32,source_image_height: u32,gradient: &[f32],work_image_pixels: &mut [u8]) {
    const DISTANCE_MIN: f32 = 4.0;
    const DISTANCE_MAX: f32 = 50.0;

    let edge_pixels = non_maximum_suppression(gradient,source_image_width,source_image_height);

    let width = source_image_width as usize;
    let height = source_image_height as usize;

    //Find all edge points.
    let mut edge_points = vec![];
    for y in 0..height {
        for x in 0..width {
            if edge_pixels[y * width + x] == 255 {
                edge_points.push((x,y));
            }
        }
    }

    let lines_per_point = 1;//TODO: Maybe support variable lines again? 1000000 / edge_points.len();

    //Draw a line between points that are between DISTANCE_MIN and DISTANCE_MAX of each other and
    //have the same relative orientation. Only the closest lines_per_point lines are drawn.
    //TODO: Use a quadtree to speed up this part.
    let mut painter = Painter::new();
    let mut other_points = vec![];
    for y in 0..edge_points.len() {
        let first_point = edge_points[y];

        //Find potential matching points.
        other_points.clear();
        for x in y + 1..edge_points.len() {
            let second_point = edge_points[x];

            let first_direction = angle_to_direction(gradient[first_point.1 * width + first_point.0 + 1]);
            let second_direction = angle_to_direction(gradient[second_point.1 * width + second_point.0 + 1]);

            let diff_x = second_point.0 as f32 - first_point.0 as f32;
            let diff_y = second_point.1 as f32 - first_point.1 as f32;
            let distance = diff_x.hypot(diff_y);

            if distance >= DISTANCE_MIN && distance <= DISTANCE_MAX && first_direction == second_direction {
                other_points.push((second_point,diff_x,diff_y,distance));
            }
        }

        //Sort by distance.
        other_points.sort_by(|lhs,rhs| lhs.3.partial_cmp(&rhs.3).unwrap_or(std::cmp::Ordering::Equal));
        if other_points.len() > lines_per_point {
            other_points.drain(lines_per_point..);
        }

        for &(second_point,diff_x,diff_y,_) in &other_points {
            //Randomly pick a pixel between (inclusive) the two points to sample and use as the
            //color for the line.
            let range = Closed01::<f32>::rand(rng).0;
            let xc = (first_point.0 as f32 + diff_x * range).round() as i32;
            let yc = (first_point.1 as f32 + diff_y * range).round() as i32;
            let xc = std::cmp::min(std::cmp::max(0,xc),width as i32 - 1) as usize;
            let yc = std::cmp::min(std::cmp::max(0,yc),height as i32 - 1) as usize;

            let (red,green,blue) = color_at(&source_image_pixels,source_image_width,source_image_height,xc,yc);
            painter.set_pen(red,green,blue);

            //Draw the line.
            painter.line(work_image_pixels,source_image_width as usize,source_image_height as usize,first_point.0 as i32,first_point.1 as i32,second_point.0 as i32,second_point.1 as i32);
        }
    }


    //Debug code for showing the detected edge pixels.
    /*for y in 0..source_image_height {
        for x in 0..source_image_width {
            let index = (y * source_image_width + x) as usize;

            if edge_pixels[index] == 255 {
                let index = index * 3;
                work_image_pixels[index + 0] = 0;
                work_image_pixels[index + 1] = 255;
                work_image_pixels[index + 2] = 0;
            }
        }
    }*/
}

fn main() {
    let matches = App::new("line-splat")
        .version("0.1")
        .about("Stylize images by drawing random lines. Supports JPEG and PNG images.")
        .author("James Bendig")
        .arg(Arg::with_name("line-count")
             .short("l")
             .long("line-count")
             .default_value("1000000")
             .help("Number of lines to draw")
             .required(false))
        .arg(Arg::with_name("style")
             .short("s")
             .long("style")
             .default_value("random")
             .help("Style to use. Must be random, steered, energy, or edgeweb.")
             .required(false))
        .arg(Arg::with_name("INPUT")
             .help("Input image file")
             .required(true)
             .index(1))
        .arg(Arg::with_name("OUTPUT")
             .help("Output image file")
             .required(true)
             .index(2))
        .get_matches();

    //Extract and validate parameters from command line.
    let input_path = Path::new(matches.value_of("INPUT").unwrap());
    let output_path = Path::new(matches.value_of("OUTPUT").unwrap());
    if input_path == output_path {
        eprintln!("Input and output file paths cannot be the same");
        return;
    }

    let line_count = matches.value_of("line-count").unwrap();
    let line_count = if let Ok(line_count) = u64::from_str(line_count) {
        line_count
    }
    else {
        eprintln!("Line count must be a positive integer.");
        return;
    };

    let style = match value_t!(matches,"style",Style) {
        Ok(style) => style,
        Err(e) => {
            eprintln!("{}       See --help",e);
            return;
        }
    };

    //Make sure a supported file extension was selected before wasting time generating an image.
    match output_path.extension() {
        Some(extension) => {
            match extension.to_string_lossy().to_lowercase().as_str() {
                "jpg" | "jpeg" => (),
                "png" => (),
                _ => {
                    eprintln!("Unsupported output file format. Must have a .png or .jpg extension");
                    return;
                }
            }
        },
        None => {
            eprintln!("Output file format must have a .png or .jpg extension");
            return;
        }
    };

    //Open source file.
    let source_image = match image::open(&input_path) {
        Ok(image) => image.to_rgb(),
        Err(e) => {
            eprintln!("Could not open input file: {}",e);
            return;
        }
    };
    let (source_image_width,source_image_height) = source_image.dimensions();
    let mut source_image_pixels = source_image.into_raw();

    //Generate gradient for source image. It's used by the energy style to determine how far to
    //shoot the rays. It's also used by the edgeweb style to detect edges in the image.
    let source_image_gradient = gradient(&mut source_image_pixels,source_image_width,source_image_height);

    //Create a canvas to write the generated image to.
    let mut work_image_pixels = Vec::with_capacity(source_image_pixels.capacity());
    work_image_pixels.resize(source_image_pixels.len(),0);

    //Generate image using the selected style.
    let mut rng = rand::thread_rng();
    let mut painter = Painter::new();
    for _ in 0..line_count {
        let (x1,y1,x2,y2) = match style {
            Style::Random => {
                let (x1,y1,x2,y2) = random_line(&mut rng,source_image_width,source_image_height);

                let (red1,green1,blue1) = color_at(&source_image_pixels,source_image_width,source_image_height,x1,y1);
                let (red2,green2,blue2) = color_at(&source_image_pixels,source_image_width,source_image_height,x2,y2);
                painter.set_pen(mix(red1,red2),mix(green1,green2),mix(blue1,blue2));

                (x1,y1,x2,y2)
            },
            Style::Steered => {
                let (x1,y1,x2,y2) = random_steered_line(&mut rng,source_image_gradient.as_slice(),source_image_width,source_image_height);

                let (red1,green1,blue1) = color_at(&source_image_pixels,source_image_width,source_image_height,x1,y1);
                let (red2,green2,blue2) = color_at(&source_image_pixels,source_image_width,source_image_height,x2,y2);
                painter.set_pen(mix(red1,red2),mix(green1,green2),mix(blue1,blue2));

                (x1,y1,x2,y2)
            },
            Style::Energy => {
                let (xc,yc,x1,y1,x2,y2) = random_energy_line(&mut rng,source_image_gradient.as_slice(),source_image_width,source_image_height);
                let (red,green,blue) = color_at(&source_image_pixels,source_image_width,source_image_height,xc,yc);
                let (red,green,blue) = shift_lightness(&mut rng,red,green,blue);
                painter.set_pen(red,green,blue);

                (x1,y1,x2,y2)
            },
            Style::EdgeWeb => {
                edge_web(&mut rng,source_image_pixels.as_slice(),source_image_width,source_image_height,source_image_gradient.as_slice(),work_image_pixels.as_mut_slice());
                break;
            },
        };
        painter.line(&mut work_image_pixels,source_image_width as usize,source_image_height as usize,x1 as i32,y1 as i32,x2 as i32,y2 as i32);
    }

    //Save the results.
    let output_image = ImageBuffer::<image::Rgb<u8>,std::vec::Vec<u8>>::from_raw(source_image_width,source_image_height,work_image_pixels).unwrap();
    if let Err(e) = output_image.save(&output_path) {
        eprintln!("Could not write output to file: {}",e);
        return;
    }
}
