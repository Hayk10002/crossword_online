use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct ColorRGBA
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRGBA
{
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> ColorRGBA
    {
        ColorRGBA { r: r, g: g, b: b, a: a}
    }

    pub fn opaque(r: u8, g: u8, b: u8) -> ColorRGBA
    {
        ColorRGBA { r: r, g: g, b: b, a: 255}
    }
}

impl fmt::Display for ColorRGBA
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a as f32 / 255f32)
    }
}


