extern crate std;

use std::ops::Rem;

pub fn clamp_to_u8(value: f64) -> u8 {
    let value = value.round() as i32;
    std::cmp::min(std::cmp::max(0,value),255) as u8
}

pub fn min_f32(lhs: f32,rhs: f32) -> f32 {
    if lhs < rhs {
        lhs
    }
    else {
        rhs
    }
}

pub fn max_f32(lhs: f32,rhs: f32) -> f32 {
    if lhs < rhs {
        rhs
    }
    else {
        lhs
    }
}

pub fn difference_theta(theta1: f32,theta2: f32) -> f32 {
	//Find angle difference while taking wrapping into account.
	return min_f32(
        (theta1 - theta2).abs(),
        min_f32(theta1,theta2) + 2.0 * std::f32::consts::PI - max_f32(theta1,theta2));

}

pub fn mix(lhs: u8,rhs: u8) -> u8 {
    let sum = lhs as u32 + rhs as u32;
    (sum / 2) as u8
}

pub fn rgb_to_hsl(red: u8,green: u8,blue: u8) -> (f32,f32,f32) {
    let red = red as f32 / 255.0;
    let green = green as f32 / 255.0;
    let blue = blue as f32 / 255.0;
    let max_value = max_f32(max_f32(red,green),blue);
    let min_value = min_f32(min_f32(red,green),blue);
    let delta = max_value - min_value;

    let lightness = (min_value + max_value) * 0.5;

    let saturation = if delta == 0.0 {
        0.0
    }
    else {
        let divisor = 1.0 - (2.0 * lightness - 1.0).abs();
        if divisor == 0.0 {
            0.0
        }
        else {
            delta / divisor
        }
    };

    if saturation == 0.0 {
        return (0.0,saturation,lightness);
    }

    let mut hue = if red == max_value {
        ((green - blue) / delta).rem(6.0)
    }
    else if green == max_value {
        ((blue - red) / delta) + 2.0
    }
    else if blue == max_value {
        ((red - green) / delta) + 4.0
    }
    else {
        0.0
    };

    hue *= 60.0;
    if hue < 0.0 {
        hue += 360.0;
    }

    (hue,saturation,lightness)
}

pub fn hsl_to_rgb(hue: f32,saturation: f32,lightness: f32) -> (u8,u8,u8) {
    if saturation == 0.0 {
        let v = clamp_to_u8((lightness * 255.0) as f64);
        return (v,v,v);
    }

    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue = hue / 60.0;
    let x = (1.0 - (hue.rem(2.0) - 1.0).abs()) * chroma;

    let (red,green,blue) = if 0.0 <= hue && hue < 1.0 {
        (chroma,x,0.0)
    }
    else if 1.0 <= hue && hue < 2.0 {
        (x,chroma,0.0)
    }
    else if 2.0 <= hue && hue < 3.0 {
        (0.0,chroma,x)
    }
    else if 3.0 <= hue && hue < 4.0 {
        (0.0,x,chroma)
    }
    else if 4.0 <= hue && hue < 5.0 {
        (x,0.0,chroma)
    }
    else if 5.0 <= hue && hue <= 6.0 {
        (chroma,0.0,x)
    }
    else {
        (0.0,0.0,0.0)
    };

    let m = lightness - 0.5 * chroma;
    (clamp_to_u8(((red + m) * 255.0) as f64),
     clamp_to_u8(((green + m) * 255.0) as f64),
     clamp_to_u8(((blue + m) * 255.0) as f64))
}

