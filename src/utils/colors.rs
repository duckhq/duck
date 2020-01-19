pub struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Rgb { red, green, blue }
    }

    // Converts a RGB color to coordinates in the CIE color space.
    // https://developers.meethue.com/develop/application-design-guidance/color-conversion-formulas-rgb-to-xy-and-back/
    pub fn to_cie_coordinates(&self) -> (f32, f32) {
        let red = f32::from(self.red) / 255_f32;
        let green = f32::from(self.green) / 255_f32;
        let blue = f32::from(self.blue) / 255_f32;

        let red = if red > 0.04045_f32 {
            (red + 0.055_f32) / (1.0_f32 + 0.055_f32).powf(2.4_f32)
        } else {
            red / 12.92_f32
        };
        let green = if green > 0.04045_f32 {
            (green + 0.055_f32) / (1.0_f32 + 0.055_f32).powf(2.4_f32)
        } else {
            green / 12.92_f32
        };
        let blue = if blue > 0.04045_f32 {
            (blue + 0.055_f32) / (1.0_f32 + 0.055_f32).powf(2.4_f32)
        } else {
            blue / 12.92_f32
        };

        let x = red * 0.436_074_7_f32 + green * 0.385_064_9_f32 + blue * 0.093_080_4_f32;
        let y = red * 0.222_504_5_f32 + green * 0.716_878_6_f32 + blue * 0.040_616_9_f32;
        let z = red * 0.013_932_2_f32 + green * 0.097_104_5_f32 + blue * 0.714_173_3_f32;

        let cx = x / (x + y + z);
        let cy = y / (x + y + z);

        return (cx, cy);
    }
}
