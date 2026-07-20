use crate::{Error, Rgb};

// These are the matrices constructed by grDevices for its D65 sRGB color
// converter. Keeping the same values and operation order is important for
// matching convertColor() at the final eight-bit channel level.
const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [
        0.416_821_341_885_316_9,
        0.214_923_504_409_616_52,
        0.019_538_500_400_874_244,
    ],
    [
        0.356_576_717_077_974_6,
        0.713_153_434_155_949_2,
        0.118_858_905_692_658_32,
    ],
    [
        0.179_807_653_586_085_4,
        0.071_923_061_434_434_15,
        0.946_986_975_553_383_1,
    ],
];

const XYZ_TO_SRGB: [[f64; 3]; 3] = [
    [
        3.206_520_517_144_464_4,
        -0.971_982_546_201_232,
        0.055_838_338_593_097_92,
    ],
    [
        -1.521_041_783_773_656,
        1.881_268_651_608_487_3,
        -0.204_740_574_841_359,
    ],
    [
        -0.493_310_848_791_456,
        0.041_672_484_599_589_32,
        1.060_928_433_268_859,
    ],
];

const D65_WHITE: [f64; 3] = [0.953_205_712_549_377, 1.0, 1.085_384_381_646_915_8];
const LAB_EPSILON: f64 = 216.0 / 24_389.0;
const LAB_KAPPA: f64 = 24_389.0 / 27.0;

pub(crate) fn interpolate(colors: &[Rgb], n: usize) -> Vec<Rgb> {
    if n == 0 {
        return Vec::new();
    }

    let labs = colors.iter().copied().map(srgb_to_lab).collect::<Vec<_>>();
    let anchor_count = labs.len();
    debug_assert!(anchor_count >= 2);

    let anchor_x = evenly_spaced(anchor_count);
    let sample_x = evenly_spaced(n);
    let mut channels = [Vec::new(), Vec::new(), Vec::new()];

    for channel in 0..3 {
        let values = labs.iter().map(|lab| lab[channel]).collect::<Vec<_>>();
        let coefficients = FmmSpline::new(&anchor_x, &values);
        channels[channel] = coefficients.evaluate(&sample_x);
    }

    (0..n)
        .map(|index| lab_to_srgb([channels[0][index], channels[1][index], channels[2][index]]))
        .collect()
}

pub(crate) fn continuous_alpha(alpha: f32) -> Result<u8, Error> {
    if !(alpha.is_finite() && 0.0 < alpha && alpha <= 1.0) {
        return Err(Error::InvalidAlpha { alpha });
    }

    // grDevices::rgb() truncates components after scaling by maxColorValue.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok((f64::from(alpha) * 255.0) as u8)
}

#[allow(clippy::cast_precision_loss)]
fn evenly_spaced(n: usize) -> Vec<f64> {
    match n {
        0 => Vec::new(),
        1 => vec![0.0],
        _ => {
            let denominator = (n - 1) as f64;
            (0..n)
                .map(|index| {
                    if index == n - 1 {
                        1.0
                    } else {
                        index as f64 / denominator
                    }
                })
                .collect()
        }
    }
}

fn srgb_to_lab(color: Rgb) -> [f64; 3] {
    let srgb = [
        f64::from(color.r()) / 255.0,
        f64::from(color.g()) / 255.0,
        f64::from(color.b()) / 255.0,
    ];
    let linear = srgb.map(srgb_decode);

    let xyz = [
        linear[0] * SRGB_TO_XYZ[0][0]
            + linear[1] * SRGB_TO_XYZ[1][0]
            + linear[2] * SRGB_TO_XYZ[2][0],
        linear[0] * SRGB_TO_XYZ[0][1]
            + linear[1] * SRGB_TO_XYZ[1][1]
            + linear[2] * SRGB_TO_XYZ[2][1],
        linear[0] * SRGB_TO_XYZ[0][2]
            + linear[1] * SRGB_TO_XYZ[1][2]
            + linear[2] * SRGB_TO_XYZ[2][2],
    ];
    let relative = [
        xyz[0] / D65_WHITE[0],
        xyz[1] / D65_WHITE[1],
        xyz[2] / D65_WHITE[2],
    ];
    let transformed = relative.map(|value| {
        if value <= LAB_EPSILON {
            (LAB_KAPPA * value + 16.0) / 116.0
        } else {
            value.powf(1.0 / 3.0)
        }
    });

    [
        116.0 * transformed[1] - 16.0,
        500.0 * (transformed[0] - transformed[1]),
        200.0 * (transformed[1] - transformed[2]),
    ]
}

fn lab_to_srgb(lab: [f64; 3]) -> Rgb {
    let lightness = lab[0];
    let relative_y = if lightness < LAB_KAPPA * LAB_EPSILON {
        lightness / LAB_KAPPA
    } else {
        cube((lightness + 16.0) / 116.0)
    };
    let transformed_y = (if relative_y <= LAB_EPSILON {
        LAB_KAPPA * relative_y
    } else {
        lightness
    } + 16.0)
        / 116.0;
    let transformed_x = lab[1] / 500.0 + transformed_y;
    let transformed_z = transformed_y - lab[2] / 200.0;
    let x_cubed = cube(transformed_x);
    let z_cubed = cube(transformed_z);
    let relative_x = if x_cubed <= LAB_EPSILON {
        (116.0 * transformed_x - 16.0) / LAB_KAPPA
    } else {
        x_cubed
    };
    let relative_z = if z_cubed <= LAB_EPSILON {
        (116.0 * transformed_z - 16.0) / LAB_KAPPA
    } else {
        z_cubed
    };
    let xyz = [
        relative_x * D65_WHITE[0],
        relative_y * D65_WHITE[1],
        relative_z * D65_WHITE[2],
    ];
    let linear = [
        xyz[0] * XYZ_TO_SRGB[0][0] + xyz[1] * XYZ_TO_SRGB[1][0] + xyz[2] * XYZ_TO_SRGB[2][0],
        xyz[0] * XYZ_TO_SRGB[0][1] + xyz[1] * XYZ_TO_SRGB[1][1] + xyz[2] * XYZ_TO_SRGB[2][1],
        xyz[0] * XYZ_TO_SRGB[0][2] + xyz[1] * XYZ_TO_SRGB[1][2] + xyz[2] * XYZ_TO_SRGB[2][2],
    ];
    let srgb = linear.map(srgb_encode);
    let channels = srgb.map(|value| {
        let rounded = round_to_five(value).clamp(0.0, 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let channel = (rounded * 255.0) as u8;
        channel
    });

    Rgb::from_hex(
        (u32::from(channels[0]) << 16) | (u32::from(channels[1]) << 8) | u32::from(channels[2]),
    )
}

fn srgb_decode(value: f64) -> f64 {
    if value < 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_encode(value: f64) -> f64 {
    if value <= 0.003_130_8 {
        12.92 * value
    } else {
        1.055 * value.abs().powf(1.0 / 2.4).copysign(value) - 0.055
    }
}

const fn cube(value: f64) -> f64 {
    value * value * value
}

// This is the digits = 5 path through R's fround implementation.
#[allow(clippy::float_cmp)]
fn round_to_five(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }

    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    let absolute = value.abs();
    let scaled = absolute * 100_000.0;
    let integer = scaled.floor();
    let down = integer / 100_000.0;
    let up = scaled.ceil() / 100_000.0;
    let distance_up = up - absolute;
    let distance_down = absolute - down;
    let use_up =
        distance_up < distance_down || (integer % 2.0 == 1.0 && distance_up == distance_down);

    sign * if use_up { up } else { down }
}

/// Coefficients for R's default FMM cubic spline.
struct FmmSpline<'a> {
    x: &'a [f64],
    y: &'a [f64],
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
}

impl<'a> FmmSpline<'a> {
    #[allow(clippy::many_single_char_names)]
    fn new(x: &'a [f64], y: &'a [f64]) -> Self {
        debug_assert_eq!(x.len(), y.len());
        debug_assert!(x.len() >= 2);

        let n = x.len();
        let mut b = vec![0.0; n];
        let mut c = vec![0.0; n];
        let mut d = vec![0.0; n];

        if n == 2 {
            let slope = (y[1] - y[0]) / (x[1] - x[0]);
            b[0] = slope;
            b[1] = slope;
            return Self { x, y, b, c, d };
        }

        let last = n - 1;
        d[0] = x[1] - x[0];
        c[1] = (y[1] - y[0]) / d[0];
        for index in 1..last {
            d[index] = x[index + 1] - x[index];
            b[index] = 2.0 * (d[index - 1] + d[index]);
            c[index + 1] = (y[index + 1] - y[index]) / d[index];
            c[index] = c[index + 1] - c[index];
        }

        b[0] = -d[0];
        b[last] = -d[last - 1];
        c[0] = 0.0;
        c[last] = 0.0;
        if n > 3 {
            c[0] = c[2] / (x[3] - x[1]) - c[1] / (x[2] - x[0]);
            c[last] =
                c[last - 1] / (x[last] - x[last - 2]) - c[last - 2] / (x[last - 1] - x[last - 3]);
            c[0] = c[0] * d[0] * d[0] / (x[3] - x[0]);
            c[last] = -c[last] * d[last - 1] * d[last - 1] / (x[last] - x[last - 3]);
        }

        for index in 1..n {
            let factor = d[index - 1] / b[index - 1];
            b[index] -= factor * d[index - 1];
            c[index] -= factor * c[index - 1];
        }

        c[last] /= b[last];
        for index in (0..last).rev() {
            c[index] = (c[index] - d[index] * c[index + 1]) / b[index];
        }

        b[last] =
            (y[last] - y[last - 1]) / d[last - 1] + d[last - 1] * (c[last - 1] + 2.0 * c[last]);
        for index in 0..last {
            b[index] =
                (y[index + 1] - y[index]) / d[index] - d[index] * (c[index + 1] + 2.0 * c[index]);
            d[index] = (c[index + 1] - c[index]) / d[index];
            c[index] *= 3.0;
        }
        c[last] *= 3.0;
        d[last] = d[last - 1];

        Self { x, y, b, c, d }
    }

    fn evaluate(&self, values: &[f64]) -> Vec<f64> {
        let last = self.x.len() - 1;
        let mut interval = 0;
        let mut output = Vec::with_capacity(values.len());

        for &value in values {
            if value < self.x[interval] || (interval < last && self.x[interval + 1] < value) {
                interval = 0;
                let mut upper = self.x.len();
                while upper > interval + 1 {
                    let middle = (interval + upper) / 2;
                    if value < self.x[middle] {
                        upper = middle;
                    } else {
                        interval = middle;
                    }
                }
            }

            let distance = value - self.x[interval];
            output.push(
                self.y[interval]
                    + distance
                        * (self.b[interval]
                            + distance * (self.c[interval] + distance * self.d[interval])),
            );
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::{round_to_five, FmmSpline};

    #[test]
    fn r_rounding_uses_ties_to_even() {
        assert_eq!(round_to_five(0.123_445).to_bits(), 0.123_44_f64.to_bits());
        assert_eq!(round_to_five(0.123_455).to_bits(), 0.123_45_f64.to_bits());
    }

    #[test]
    fn two_point_fmm_spline_is_linear() {
        let x = [0.0, 1.0];
        let y = [2.0, 4.0];
        let spline = FmmSpline::new(&x, &y);
        assert_eq!(spline.evaluate(&[0.0, 0.25, 1.0]), [2.0, 2.5, 4.0]);
    }
}
