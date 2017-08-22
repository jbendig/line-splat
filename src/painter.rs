extern crate std;

struct Color {
    red: u8,
    green: u8,
    blue: u8
}

//Painter is used for drawing lines. I ripped this out of an old side-project I wrote when I was
//learning Rust. This implementation is ugly and inefficient but it works fine for splatting lines.
pub struct Painter {
    pen: Color,
}

impl Painter {
    pub fn new() -> Painter {
        Painter {
            pen: Color {
                red: 0,
                green: 0,
                blue: 0,
            },
        }
    }

    pub fn set_pen(&mut self,red: u8,green: u8,blue: u8) {
        self.pen.red = red;
        self.pen.green = green;
        self.pen.blue = blue;
    }

    fn clip_line_from_outside(width: usize,height: usize,x1: i32,y1: i32,x2: i32,y2: i32) -> Result<(i32, i32, i32, i32), &'static str>  {
        //Shorten the box slightly so the end points end up inside of the box.
        let width = width - 1;
        let height = height - 1;

        //Calculate line's normal.
        let diff_x = (x2 - x1) as f32;
        let diff_y = (y2 - y1) as f32;

        if diff_x == 0.0 || diff_y == 0.0 {
            return Err("Line is a point");
        }

        let length = (diff_x * diff_x + diff_y * diff_y).sqrt();
        let nx = diff_x / length;
        let ny = diff_y / length;

        //Based off of https://psgraphics.blogspot.com/2016/02/new-simple-ray-box-test-from-andrew.html
        let inverted_direction = 1.0 / nx;
        let mut t = (
            -1.0 * x1 as f32 * inverted_direction,
            (width as i32 - x1) as f32 * inverted_direction
        );
        if inverted_direction < 0.0 {
            t = (t.1, t.0);
        }
        let t_min_x = t.0.max(0.0);
        let t_max_x = t.1.min(length);
        if t_max_x < t_min_x {
            return Err("Line does not intersect box");
        }

        let inverted_direction = 1.0 / ny;
        let mut t = (
            -1.0 * y1 as f32 * inverted_direction,
            (height as i32 - y1) as f32 * inverted_direction
        );
        if inverted_direction < 0.0 {
            t = (t.1, t.0);
        }
        let t_min_y = t.0.max(0.0);
        let t_max_y = t.1.min(length);
        if t_max_y < t_min_y {
            return Err("Line does not intersect box");
        }

        let result = (
            x1 + (nx * t_min_x).round() as i32,
            y1 + (ny * t_min_y).round() as i32,
            x1 + (nx * t_max_x).round() as i32,
            y1 + (ny * t_max_y).round() as i32
        );
        Ok(result)
    }

    fn clip_line_from_inside(width: usize,height: usize,x1: i32,y1: i32,x2: i32,y2: i32) -> (i32, i32)  {
        //Shorten the box slightly so the end points end up inside of the box.
        let width = width - 1;
        let height = height - 1;

        if x1 == x2 {
            if y2 > y1 {
                return (x1,height as i32);
            }
            else {
                return (x1,0);
            }
        }
        else if y1 == y2 {
            if x2 > x1 {
                return (width as i32,y1);
            }
            else {
                return (0,y1);
            }
        }

        let slope = (y2 - y1) as f32 / (x2 - x1) as f32;
        let b = y1 as f32 - slope * x1 as f32;

        let top_intersection = ((-b / slope) as i32,0);
        let bottom_intersection = (((height as f32 - b) / slope) as i32,height as i32);
        let left_intersection = (0,b as i32);
        let right_intersection = (width as i32,(slope * width as f32 + b) as i32);

        //Check distance between x1,y1 and each of these. Use the closest positive one.
        let distance_squared = |intersection: (i32,i32)| {
            let diff_x = (intersection.0 - x1) as f32;
            let diff_y = (intersection.1 - y1) as f32;
            diff_x * diff_x + diff_y * diff_y
        };

        let top_intersection_distance = distance_squared(top_intersection);
        let bottom_intersection_distance = distance_squared(bottom_intersection);
        let left_intersection_distance = distance_squared(left_intersection);
        let right_intersection_distance = distance_squared(right_intersection);

        let line_down = y2 - y1 > 0;
        let line_right = x2 - x1 > 0;

        if line_down {
            if line_right {
                return if bottom_intersection_distance < right_intersection_distance { bottom_intersection } else { right_intersection };
            }
            else {
                return if bottom_intersection_distance < left_intersection_distance { bottom_intersection } else { left_intersection };
            }
        }
        else {
           if line_right {
                return if top_intersection_distance < right_intersection_distance { top_intersection } else { right_intersection };
            }
            else {
                return if top_intersection_distance < left_intersection_distance { top_intersection } else { left_intersection };
            }
        }
    }

    pub fn line_foreach<F>(width: usize,height: usize,x1: i32,y1: i32,x2: i32,y2: i32,mut func: F)
        where F: FnMut(usize,usize) {
        //Get out early if a line cannot be drawn because it does not fit
        //within the image.
        if width == 0 || height == 0 || (x1 < 0 && x2 < 0) || (y1 < 0 && y2 < 0) || (x1 >= width as i32 && x2 >= width as i32) || (y1 >= height as i32 && y2 >= height as i32) {
            return;
        }

        let (mut x1, mut x2) = (x1,x2);
        let (mut y1, mut y2) = (y1,y2);

        //Find the two points the line would intersect the image if it had an
        //infinite length.
        if y1 == y2 {
            //Special case, a horizontal line. Simply clip to the left and right
            //sides.
            if x2 < x1 {
                std::mem::swap(&mut x1,&mut x2);
            }
            x1 = std::cmp::max(0,x1);
            x2 = std::cmp::min(x2,width as i32 - 1);
        }
        else if x1 == x2 {
            //Special case, a vertical line. Simply clip to the top and bottom
            //sides.
            if y2 < y1 {
                std::mem::swap(&mut y1,&mut y2);
            }
            y1 = std::cmp::max(0,y1);
            y2 = std::cmp::min(y2,height as i32 - 1);
        }
        else {
            //If both points are in the image, do nothing.
            let mut p1_in_box = x1 >= 0 && x1 < width as i32 && y1 >= 0 && y1 < height as i32;
            let p2_in_box = x2 >= 0 && x2 < width as i32 && y2 >= 0 && y2 < height as i32;
            if !p1_in_box || !p2_in_box {
                //If point1 is outside of image, fire ray to edge of image and
                //use as new point1.
                if !p1_in_box {
                    let intersection = Self::clip_line_from_outside(width,height,x1,y1,x2,y2);
                    match intersection {
                        Ok(point) => { x1 = point.0; y1 = point.1; },
                        Err(_) => return,
                    }
                    p1_in_box = true;
                }

                //If point1 is in image (which must be true after above), fire
                //to edge of image and use as second point if further than
                //second point.
                if p1_in_box && !p2_in_box {
                    let intersection = Self::clip_line_from_inside(width,height,x1,y1,x2,y2);
                    x2 = intersection.0;
                    y2 = intersection.1;
                }
            }
        }

        if x1 == x2 {
            if y2 < y1 {
                std::mem::swap(&mut y1,&mut y2);
            }

            //Handle the special case of a point.
            if y1 == y2 {
                y2 += 1;
            }

            //Handle special case of a vertical line.
            for y in y1..y2 {
                func(x1 as usize,y as usize);
            }
            return;
        }

        //Always draw from left-to-right.
        if x2 < x1 {
            std::mem::swap(&mut x1,&mut x2);
            std::mem::swap(&mut y1,&mut y2);
        }

        //Bresenham's Line Algorithm
        //https://en.wikipedia.org/wiki/Bresenham's_line_algorithm
        let delta_x = x2 as f32 - x1 as f32;
        let delta_y = y2 as f32 - y1 as f32;
        let mut error = 0.0;
        let delta_error = (delta_y / delta_x).abs();

        let y_increment: i32 = if y2 > y1 { 1 } else { -1 };
        let mut y = y1 as usize;
        let (x1,x2) = (x1 as usize,x2 as usize);
        for x in x1..x2 {
            assert!(x < width);
            assert!(y < height);

            func(x,y);

            error += delta_error;
            while error >= 0.5 {
                func(x,y);
                if y_increment != -1 || y > 0 {
                    y = (y as i32 + y_increment) as usize;
                }
                error -= 1.0;
            }
        }
    }

    pub fn line(&self,buffer: &mut [u8],width: usize,height: usize,x1: i32,y1: i32,x2: i32,y2: i32) {
        Self::line_foreach(width,height,x1,y1,x2,y2,|x,y| {
            let index: usize = (y * width + x) * 3;
            buffer[index + 0] = self.pen.red;
            buffer[index + 1] = self.pen.green;
            buffer[index + 2] = self.pen.blue;
        });
    }
 }

