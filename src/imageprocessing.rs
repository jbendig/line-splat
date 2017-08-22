extern crate std;

pub fn gradient(buffer: &[u8],width: u32,height: u32) -> Vec<f32> {
    let mut result = Vec::with_capacity(buffer.len());
    result.resize(buffer.len(),0.0);

    let value_at = |x: u32,y: u32| -> f32 {
        let x = std::cmp::min(x,width - 1);
        let y = std::cmp::min(y,height - 1);

        let index = ((y * width + x) * 3) as usize;
        0.299 * buffer[index + 0] as f32 + 0.587 * buffer[index + 1] as f32 + 0.114 * buffer[index + 2] as f32
    };

    for y in 0..height {
        for x in 0..width {
            let (a,b,c,d,_,f,g,h,i) = (
                value_at(x.saturating_sub(1),y.saturating_sub(1)),value_at(x,y.saturating_sub(1)),value_at(x + 1,y.saturating_sub(1)),
                value_at(x.saturating_sub(1),y),value_at(x,y),value_at(x + 1,y),
                value_at(x.saturating_sub(1),y + 1),value_at(x,y + 1),value_at(x + 1,y + 1));
            let hsum = a * -1.0 + c + d * -2.0 + f * 2.0 + g * -1.0 + i;
            let vsum = a * -1.0 + b * -2.0 + c * -1.0 + g + h * 2.0 + i;

            let magnitude = hsum.hypot(vsum);
            let theta = vsum.atan2(hsum);

            let output_index = ((y * width + x) * 2) as usize;
            result[output_index + 0] = magnitude;
            result[output_index + 1] = theta;
        }
    }

    result
}

pub fn angle_to_direction(angle: f32) -> u32 {
    let mut angle = angle;
    if angle < 0.0 {
        angle += std::f32::consts::PI;
    }
    angle = angle * 4.0 / std::f32::consts::PI;
    angle.round() as u32 % 4
}

pub fn non_maximum_suppression(gradient: &[f32],width: u32,height: u32) -> Vec<u8> {
    if width == 0 || height == 0 {
        return vec![];
    }

    let width = width as usize;
    let height = height as usize;

    const THRESHOLD_HIGH: f32 = 110.0;
    const THRESHOLD_LOW: f32 = THRESHOLD_HIGH / 2.0;

    let result_size = width * height;
    let mut result = Vec::with_capacity(result_size);
    result.resize(result_size,0);

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let base_index = y * width + x;
            let input_index = base_index * 2;
            let output_index = base_index;

            let magnitude = gradient[input_index];
            let direction = angle_to_direction(gradient[input_index + 1]);

            let suppress = match direction {
                0 => magnitude < gradient[input_index - 2] || magnitude < gradient[input_index + 2],
                1 => magnitude < gradient[input_index - width * 2 - 2] || magnitude < gradient[input_index + width * 2 + 2],
                2 => magnitude < gradient[input_index - width * 2] || magnitude < gradient[input_index + width * 2],
                3 => magnitude < gradient[input_index - width * 2 + 2] || magnitude < gradient[input_index + width * 2 - 2],
                _ => unreachable!(),
            };

            if suppress || magnitude < THRESHOLD_LOW {
                result[output_index] = 0;
            }
            else {
                //Note: High threshold is usually used here to divide up the edge into strong and
                //weak portions but we want lots of edges so it's left out.
                result[output_index] = 255;
            }
        }
    }

    result
}
