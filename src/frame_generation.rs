use glam::Vec2;
use ndarray::Array2;

#[derive(Clone, Copy)]
pub struct RGB {
    pub r: f32,
    pub b: f32,
    pub g: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct YCbCr {
    pub y: f32,
    pub cb: f32,
    pub cr: f32,
}

#[derive(Clone, Copy)]
struct YUVtoRGB {
    pub y: [f32; 3],
    pub cb: [f32; 3],
    pub cr: [f32; 3],
    // pub b: f32,
    // pub c: f32,
    // pub d: f32,
    // pub e: f32,
}

// const BT601: YUVtoRGB = YUVtoRGB {
//     a: 0.299,
//     b: 0.587,
//     c: 0.114,
//     d: 1.772,
//     e: 1.402,
// };

const BT709: YUVtoRGB = YUVtoRGB {
    y: [0.2126, 0.7152, 0.0722],
    cb: [-0.1146, -0.3854, 0.5],
    cr: [0.5, -0.4542, -0.0458],
    // b: 0.7152,
    // c: 0.0722,
    // d: 1.8556,
    // e: 1.5748,
};

// const BT2020: YUVtoRGB = YUVtoRGB {
//     a: 0.2627,
//     b: 0.6780,
//     c: 0.0593,
//     d: 1.8814,
//     e: 1.4746,
// };

fn rgb_to_yuv(rgb: RGB, scheme: YUVtoRGB) -> YCbCr {
    // let y = scheme.a * rgb.r + scheme.b * rgb.g + scheme.c * rgb.b;
    // let u = (rgb.b - y) / scheme.d;
    // let v = (rgb.r - y) / scheme.e;
    let y = scheme.y[0] * rgb.r + scheme.y[1] * rgb.g + scheme.y[2] * rgb.b;
    let cb = scheme.cb[0] * rgb.r + scheme.cb[1] * rgb.g + scheme.cb[2] * rgb.b;
    let cr = scheme.cr[0] * rgb.r + scheme.cr[1] * rgb.g + scheme.cr[2] * rgb.b;
    // let u = (rgb.b - y) / scheme.d;
    // let v = (rgb.r - y) / scheme.e;

    YCbCr { y, cb, cr }
}

pub struct FrameIterator {
    pub width: usize,
    pub height: usize,
    pub position: Vec2,
    pub velocity: Vec2,
    pub circle_color: RGB,
    pub background: RGB,
    pub radius: f32,
    pub delta_t: f32,
}

impl Iterator for FrameIterator {
    type Item = Array2<YCbCr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.position += self.delta_t * self.velocity;

        let outside = self.position.cmple(Vec2::ZERO) | self.position.cmpge(Vec2::ONE);

        if outside.x {
            self.velocity.x *= -1.;
        }
        if outside.y {
            self.velocity.y *= -1.;
        }
        self.position = self.position.clamp(Vec2::ZERO, Vec2::ONE);

        let scheme = BT709;
        let background = rgb_to_yuv(self.background, scheme);
        let circle_yuv = rgb_to_yuv(self.circle_color, scheme);
        let out = Array2::from_shape_fn((self.height, self.width), |(row, col)| {
            let cell_pos = Vec2::new(
                (row as f32) / (self.height as f32),
                (col as f32) / (self.width as f32),
            );

            if (cell_pos - self.position).length() <= self.radius{
                circle_yuv
            } else {
                background
            }
        });

        Some(out)
    }
}
