use image::Rgb;

// D65/2deg reference observer
const XYZ_REFERENCE: [f32; 3] = [95.047, 100.00, 108.883];

pub trait Color {
    fn xyz(&self) -> XYZ;
    fn lab(&self) -> LAB;
    fn srgb(&self) -> SRGB;
    fn oklab(&self) -> OKLAB;
}

#[derive(Copy, Clone, Debug)]
pub struct SRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
#[derive(Copy, Clone, Debug)]
pub struct LAB {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}
#[derive(Copy, Clone, Debug)]
pub struct XYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Copy, Clone, Debug)]
pub struct OKLAB {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Color for LAB {
    fn xyz(&self) -> XYZ {
        let evaluate = |v: f32, i: usize| {
            (if v.powf(3.) > 0.008856 {
                v.powf(3.)
            } else {
                ((v - 16.) / 116.) / 7.787
            }) * XYZ_REFERENCE[i]
                / 100.
        };

        let y = (self.l + 16.) / 116.;
        let x = self.a / 500. + y;
        let z = y - self.b / 200.;

        XYZ {
            x: evaluate(x, 0),
            y: evaluate(y, 1),
            z: evaluate(z, 2),
        }
    }

    fn lab(&self) -> LAB {
        *self
    }

    fn srgb(&self) -> SRGB {
        self.xyz().srgb()
    }

    fn oklab(&self) -> OKLAB {
        self.xyz().oklab()
    }
}
impl Color for XYZ {
    fn xyz(&self) -> XYZ {
        *self
    }

    fn lab(&self) -> LAB {
        let evaluate = |v: f32, i: usize| {
            let v = v * 100. / XYZ_REFERENCE[i];
            if v > 0.008856 {
                v.powf(1. / 3.)
            } else {
                (7.787 * v) + (16. / 116.)
            }
        };

        let x = evaluate(self.x, 0);
        let y = evaluate(self.y, 1);
        let z = evaluate(self.z, 2);

        let l = (116. * y) - 16.;
        let a = 500. * (x - y);
        let b = 200. * (y - z);

        LAB { l, a, b }
    }

    fn srgb(&self) -> SRGB {
        let evaluate = |v: f32| {
            const GAMMA: f32 = 2.4;
            if v <= 0.0031308 {
                v * 12.92
            } else {
                1.055 * v.powf(1. / GAMMA) - 0.055
            }
            .clamp(0., 1.)
        };

        let x = self.x;
        let y = self.y;
        let z = self.z;

        SRGB {
            r: evaluate(3.2404542 * x + -1.5371385 * y + -0.4985314 * z),
            g: evaluate(-0.969266 * x + 1.8760108 * y + 0.0415560 * z),
            b: evaluate(0.0556434 * x + -0.2040259 * y + 1.0572252 * z),
        }
    }

    // https://en.wikipedia.org/wiki/Oklab_color_space#Conversion_from_CIE_XYZ
    fn oklab(&self) -> OKLAB {
        #[allow(clippy::excessive_precision)]
        let m1: [[f32; 3]; 3] = [
            [0.8189330101, 0.3618667424, -0.1288597137],
            [0.0329845436, -0.9293118715, -0.0361456387],
            [0.0482003018, -0.2643662691, -0.6338517070],
        ];

        #[allow(clippy::excessive_precision)]
        let m2 = [
            [0.2104542553, -0.7936177850, -0.0040720468],
            [1.9779984951, -2.4285922050, -0.4505937099],
            [0.0259040371, -0.7827717662, -0.8086757660],
        ];

        // [l,m,s] = m1*[x,y,z]
        let l = self.x * m1[0][0] + self.y * m1[0][1] + self.z * m1[0][2];
        let m = self.x * m1[1][0] + self.y * m1[1][1] + self.z * m1[1][2];
        let s = self.x * m1[2][0] + self.y * m1[2][1] + self.z * m1[2][2];

        // Cube root non-linearity
        let l = l.cbrt();
        let m = m.cbrt();
        let s = s.cbrt();

        // [l,a,b] = m2*[l,m,s]
        // Order changed so that l doesn't get until it is no longer needed
        let a = l * m2[1][0] + m * m2[1][1] + s * m2[1][2];
        let b = l * m2[2][0] + m * m2[2][1] + s * m2[2][2];
        let l = l * m2[0][0] + m * m2[0][1] + s * m2[0][2];

        OKLAB { l, a, b }
    }
}
impl Color for SRGB {
    fn xyz(&self) -> XYZ {
        let evaluate = |v: f32| {
            const GAMMA: f32 = 2.4;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(GAMMA)
            }
        };

        // linearize the rgb
        let r = evaluate(self.r);
        let g = evaluate(self.g);
        let b = evaluate(self.b);

        XYZ {
            // This is for a standard D65/2deg illuminant
            x: (0.4124564 * r + 0.3575761 * g + 0.1804375 * b),
            y: (0.2126729 * r + 0.7151522 * g + 0.0721750 * b),
            z: (0.0193339 * r + 0.119192 * g + 0.9503041 * b),
        }
    }

    fn lab(&self) -> LAB {
        self.xyz().lab()
    }

    fn srgb(&self) -> SRGB {
        *self
    }

    fn oklab(&self) -> OKLAB {
        self.xyz().oklab()
    }
}

impl Color for OKLAB {
    fn xyz(&self) -> XYZ {
        // Inverses of m1 and m2 from XYZ.oklab()
        #[allow(clippy::excessive_precision)]
        let m1inv = [
            [1.215245062, 0.552449894, -0.2785585068],
            [0.04019095741, -1.075538345, 0.05316231667],
            [0.07564868052, 0.4905947366, -1.621011533],
        ];
        #[allow(clippy::excessive_precision)]
        let m2inv = [
            [-1.760838934, 0.6978870164, -0.3799956598],
            [-1.735326968, 0.1858765217, -0.09483214671],
            [1.623335549, -0.157567232, -1.156967395],
        ];

        let l = self.l * m2inv[0][0] + self.a * m2inv[0][1] + self.b * m2inv[0][2];
        let m = self.l * m2inv[1][0] + self.a * m2inv[1][1] + self.b * m2inv[1][2];
        let s = self.l * m2inv[2][0] + self.a * m2inv[2][1] + self.b * m2inv[2][2];

        let l = l.powi(3);
        let m = m.powi(3);
        let s = s.powi(3);

        let x = l * m1inv[0][0] + m * m1inv[0][1] + s * m1inv[0][2];
        let y = l * m1inv[1][0] + m * m1inv[1][1] + s * m1inv[1][2];
        let z = l * m1inv[2][0] + m * m1inv[2][1] + s * m1inv[2][2];

        XYZ { x, y, z }
    }

    fn lab(&self) -> LAB {
        self.xyz().lab()
    }

    fn srgb(&self) -> SRGB {
        self.xyz().srgb()
    }

    fn oklab(&self) -> OKLAB {
        *self
    }
}

impl From<&image::Rgb<u8>> for SRGB {
    fn from(value: &image::Rgb<u8>) -> Self {
        SRGB {
            r: value[0] as f32 / 255.0,
            g: value[1] as f32 / 255.0,
            b: value[2] as f32 / 255.0,
        }
    }
}

impl From<SRGB> for image::Rgb<u8> {
    fn from(val: SRGB) -> Rgb<u8> {
        [
            (val.r * 255.) as u8,
            (val.g * 255.) as u8,
            (val.b * 255.) as u8,
        ]
        .into()
    }
}
