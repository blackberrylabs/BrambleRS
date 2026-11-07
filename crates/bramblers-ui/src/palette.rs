//! Cleanator's palette, shared by the family: green for "fine", orange for "look at this",
//! grey for information, amber for the third regime where a model decides.

use iced::Color;

/// `0xRRGGBB` → colour.
pub fn hex(rgb: u32) -> Color {
    Color::from_rgb8(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

/// Fine, safe, asks: green.
pub fn good(dark: bool) -> Color {
    if dark {
        hex(0x2fa32f)
    } else {
        hex(0x008300)
    }
}

/// Look at this: orange.
pub fn alert(dark: bool) -> Color {
    if dark {
        hex(0xd95926)
    } else {
        hex(0xeb6834)
    }
}

/// A model decides, a value is estimated: amber.
pub fn amber(dark: bool) -> Color {
    if dark {
        hex(0xd49a1a)
    } else {
        hex(0xb98400)
    }
}

/// Information, off, never: grey.
pub fn neutral() -> Color {
    hex(0x7a869a)
}

/// Not readable, not known: a lighter grey.
pub fn muted() -> Color {
    hex(0x9aa3b2)
}

/// Near-black or near-white, whichever reads on `bg` — Cleanator's rule, kept exactly so
/// its pills and tiles do not change.
pub fn contrast_text(bg: Color) -> Color {
    let lum = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
    if lum > 0.6 {
        Color::from_rgb(0.12, 0.12, 0.14)
    } else {
        Color::from_rgb(0.98, 0.98, 0.98)
    }
}

/// `a` towards `b` by `t` in `[0, 1]`.
pub fn blend(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_family_colours_are_cleanators() {
        assert_eq!(good(false), Color::from_rgb8(0x00, 0x83, 0x00));
        assert_eq!(good(true), Color::from_rgb8(0x2f, 0xa3, 0x2f));
        assert_eq!(alert(false), Color::from_rgb8(0xeb, 0x68, 0x34));
        assert_eq!(alert(true), Color::from_rgb8(0xd9, 0x59, 0x26));
        assert_eq!(neutral(), Color::from_rgb8(0x7a, 0x86, 0x9a));
    }

    #[test]
    fn contrast_text_reads_on_both_ends() {
        let dark = Color::from_rgb(0.12, 0.12, 0.14);
        let light = Color::from_rgb(0.98, 0.98, 0.98);
        assert_eq!(contrast_text(Color::WHITE), dark);
        assert_eq!(contrast_text(Color::BLACK), light);
        assert_eq!(contrast_text(good(false)), light);
    }

    #[test]
    fn blend_interpolates_and_clamps() {
        let m = blend(Color::BLACK, Color::WHITE, 0.5);
        assert!((m.r - 0.5).abs() < 1e-6);
        assert_eq!(blend(Color::BLACK, Color::WHITE, 2.0), Color::WHITE);
    }
}
