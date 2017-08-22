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

