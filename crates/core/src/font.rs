//! Our own single-stroke lettering for plan annotation: capitals, digits, a few marks and the
//! accents of Latin languages, drawn on an 8 x 12 grid (y up). Lower case is drawn as capitals,
//! as architectural lettering is. A project may write in [`Lettering::Iso`] instead.

mod iso;

use serde::{Deserialize, Serialize};

type Glyph = &'static [&'static [(i8, i8)]];

const O: &[(i8, i8)] = &[(2, 0), (0, 2), (0, 10), (2, 12), (6, 12), (8, 10), (8, 2), (6, 0), (2, 0)];
const P: &[(i8, i8)] = &[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)];
const QUESTION: Glyph = &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (4, 5), (4, 3)], &[(4, 1), (4, 0)]];

fn glyph(c: char) -> Glyph {
    match c {
        'A' => &[&[(0, 0), (4, 12), (8, 0)], &[(1, 3), (7, 3)]],
        'B' => &[&[(0, 0), (0, 12), (6, 12), (8, 10), (8, 8), (6, 6), (0, 6)], &[(6, 6), (8, 4), (8, 2), (6, 0), (0, 0)]],
        'C' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2)]],
        'D' => &[&[(0, 0), (0, 12), (5, 12), (8, 9), (8, 3), (5, 0), (0, 0)]],
        'E' => &[&[(8, 12), (0, 12), (0, 0), (8, 0)], &[(0, 6), (6, 6)]],
        'F' => &[&[(8, 12), (0, 12), (0, 0)], &[(0, 6), (6, 6)]],
        'G' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (5, 5)]],
        'H' => &[&[(0, 0), (0, 12)], &[(8, 0), (8, 12)], &[(0, 6), (8, 6)]],
        'I' => &[&[(2, 12), (6, 12)], &[(4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        'J' => &[&[(8, 12), (8, 2), (6, 0), (2, 0), (0, 2)]],
        'K' => &[&[(0, 0), (0, 12)], &[(8, 12), (0, 4)], &[(3, 7), (8, 0)]],
        'L' => &[&[(0, 12), (0, 0), (8, 0)]],
        'M' => &[&[(0, 0), (0, 12), (4, 6), (8, 12), (8, 0)]],
        'N' => &[&[(0, 0), (0, 12), (8, 0), (8, 12)]],
        'O' => &[O],
        'P' => &[P],
        'Q' => &[O, &[(5, 3), (8, 0)]],
        'R' => &[P, &[(4, 6), (8, 0)]],
        'S' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 8), (2, 6), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)]],
        'T' => &[&[(0, 12), (8, 12)], &[(4, 12), (4, 0)]],
        'U' => &[&[(0, 12), (0, 2), (2, 0), (6, 0), (8, 2), (8, 12)]],
        'V' => &[&[(0, 12), (4, 0), (8, 12)]],
        'W' => &[&[(0, 12), (2, 0), (4, 6), (6, 0), (8, 12)]],
        'X' => &[&[(0, 0), (8, 12)], &[(0, 12), (8, 0)]],
        'Y' => &[&[(0, 12), (4, 6), (8, 12)], &[(4, 6), (4, 0)]],
        'Z' => &[&[(0, 12), (8, 12), (0, 0), (8, 0)]],
        '0' => &[&[(2, 0), (1, 2), (1, 10), (2, 12), (6, 12), (7, 10), (7, 2), (6, 0), (2, 0)]],
        '1' => &[&[(2, 10), (4, 12), (4, 0)], &[(2, 0), (6, 0)]],
        '2' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (0, 0), (8, 0)]],
        '3' => &[&[(0, 10), (2, 12), (6, 12), (8, 10), (8, 8), (6, 6), (8, 4), (8, 2), (6, 0), (2, 0), (0, 2)], &[(3, 6), (6, 6)]],
        '4' => &[&[(6, 0), (6, 12), (0, 4), (8, 4)]],
        '5' => &[&[(8, 12), (0, 12), (0, 7), (6, 7), (8, 5), (8, 2), (6, 0), (2, 0), (0, 2)]],
        '6' => &[&[(8, 10), (6, 12), (2, 12), (0, 10), (0, 2), (2, 0), (6, 0), (8, 2), (8, 5), (6, 7), (2, 7), (0, 5)]],
        '7' => &[&[(0, 12), (8, 12), (3, 0)]],
        '8' => &[&[
            (2, 6),
            (0, 8),
            (0, 10),
            (2, 12),
            (6, 12),
            (8, 10),
            (8, 8),
            (6, 6),
            (2, 6),
            (0, 4),
            (0, 2),
            (2, 0),
            (6, 0),
            (8, 2),
            (8, 4),
            (6, 6),
        ]],
        '9' => &[&[(0, 2), (2, 0), (6, 0), (8, 2), (8, 10), (6, 12), (2, 12), (0, 10), (0, 7), (2, 5), (6, 5), (8, 7)]],
        ' ' => &[],
        '.' => &[&[(3, 0), (4, 0), (4, 1), (3, 1), (3, 0)]],
        ',' => &[&[(4, 1), (4, 0), (3, -2)]],
        '-' => &[&[(1, 6), (7, 6)]],
        '+' => &[&[(4, 2), (4, 10)], &[(0, 6), (8, 6)]],
        '=' => &[&[(1, 4), (7, 4)], &[(1, 8), (7, 8)]],
        '_' => &[&[(0, -1), (8, -1)]],
        '/' => &[&[(0, 0), (8, 12)]],
        '(' => &[&[(6, 12), (4, 9), (4, 3), (6, 0)]],
        '{' => &[&[(6, 12), (4, 11), (4, 7), (2, 6), (4, 5), (4, 1), (6, 0)]],
        '}' => &[&[(2, 12), (4, 11), (4, 7), (6, 6), (4, 5), (4, 1), (2, 0)]],
        ')' => &[&[(2, 12), (4, 9), (4, 3), (2, 0)]],
        ':' => &[&[(4, 2), (4, 3)], &[(4, 8), (4, 9)]],
        '\'' => &[&[(4, 12), (4, 9)]],
        '"' => &[&[(3, 12), (3, 9)], &[(5, 12), (5, 9)]],
        '#' => &[&[(2, 0), (3, 12)], &[(5, 0), (6, 12)], &[(0, 4), (8, 4)], &[(0, 8), (8, 8)]],
        '×' => &[&[(1, 2), (7, 10)], &[(1, 10), (7, 2)]],
        '²' => &[&[(2, 11), (3, 12), (5, 12), (6, 11), (6, 10), (2, 7), (6, 7)]],
        '°' => &[&[(3, 12), (5, 12), (6, 11), (6, 9), (5, 8), (3, 8), (2, 9), (2, 11), (3, 12)]],
        '±' => &[&[(4, 5), (4, 11)], &[(1, 8), (7, 8)], &[(1, 2), (7, 2)]],
        _ => QUESTION,
    }
}

/// An accented capital as its base letter and accent.
fn accent(c: char) -> Option<(char, Glyph)> {
    const ACUTE: Glyph = &[&[(3, 13), (5, 15)]];
    const GRAVE: Glyph = &[&[(3, 15), (5, 13)]];
    const CIRCUMFLEX: Glyph = &[&[(2, 13), (4, 15), (6, 13)]];
    const TILDE: Glyph = &[&[(1, 13), (3, 15), (5, 13), (7, 15)]];
    const DIAERESIS: Glyph = &[&[(2, 14), (2, 15)], &[(6, 14), (6, 15)]];
    const CEDILLA: Glyph = &[&[(4, 0), (5, -1), (3, -3)]];
    Some(match c {
        'Á' | 'É' | 'Í' | 'Ó' | 'Ú' | 'Ý' => (base(c)?, ACUTE),
        'À' | 'È' | 'Ì' | 'Ò' | 'Ù' => (base(c)?, GRAVE),
        'Â' | 'Ê' | 'Î' | 'Ô' | 'Û' => (base(c)?, CIRCUMFLEX),
        'Ã' | 'Õ' | 'Ñ' => (base(c)?, TILDE),
        'Ä' | 'Ë' | 'Ï' | 'Ö' | 'Ü' => (base(c)?, DIAERESIS),
        'Ç' => ('C', CEDILLA),
        _ => return None,
    })
}

fn base(c: char) -> Option<char> {
    Some(match c {
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'Ý' => 'Y',
        'Ñ' => 'N',
        _ => return None,
    })
}

/// The lettering a project's drawings and sheets are written in (`Model::font`, the `font` command).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lettering {
    /// Capitals on an 8 x 12 grid, every character as wide as the next: what drawings always had.
    #[default]
    Block,
    /// After ISO 3098 and NBR 8402: lower case, proportional widths, rounds as arcs, more marks ([`iso`]).
    Iso,
}

impl Lettering {
    pub const ALL: [Lettering; 2] = [Lettering::Block, Lettering::Iso];

    pub fn name(self) -> &'static str {
        match self {
            Lettering::Block => "block",
            Lettering::Iso => "iso",
        }
    }

    /// Whether it's the default, which a project file leaves out.
    pub fn is_block(&self) -> bool {
        *self == Lettering::Block
    }

    /// The lettering of a name, in any case.
    pub fn from_name(name: &str) -> Option<Lettering> {
        Lettering::ALL.into_iter().find(|f| f.name().eq_ignore_ascii_case(name.trim()))
    }

    /// How wide `text` is with capitals `height` px tall.
    pub fn width(self, text: &str, height: f32) -> f32 {
        match self {
            Lettering::Block => width(text, height),
            Lettering::Iso => {
                let units: f32 = text.chars().map(|c| iso::width(c) + iso::GAP).sum();
                (units - iso::GAP).max(0.0) * height / 10.0
            }
        }
    }

    /// Polylines drawing `text` with capitals `height` px tall, from `(x, y)` on the baseline, y down.
    pub fn strokes(self, text: &str, x: f32, y: f32, height: f32) -> Vec<Vec<(f32, f32)>> {
        match self {
            Lettering::Block => strokes(text, x, y, height),
            Lettering::Iso => {
                let k = height / 10.0;
                let mut x0 = x;
                let mut out = Vec::new();
                for c in text.chars() {
                    out.extend(iso::strokes(c).into_iter().map(|s| s.into_iter().map(|(gx, gy)| (x0 + gx * k, y - gy * k)).collect()));
                    x0 += (iso::width(c) + iso::GAP) * k;
                }
                out
            }
        }
    }
}

/// Grid units from one character to the next.
const ADVANCE: f32 = 11.0;

/// How wide `text` is at `height` px, in block letters.
fn width(text: &str, height: f32) -> f32 {
    let n = text.chars().count() as f32;
    if n == 0.0 { 0.0 } else { (n * ADVANCE - 3.0) * height / 12.0 }
}

/// Polylines drawing `text` in block letters with capitals `height` px tall, from `(x, y)` on the baseline, y down.
fn strokes(text: &str, x: f32, y: f32, height: f32) -> Vec<Vec<(f32, f32)>> {
    let k = height / 12.0;
    let mut out = Vec::new();
    for (i, c) in text.chars().flat_map(char::to_uppercase).enumerate() {
        let x0 = x + i as f32 * ADVANCE * k;
        let (letter, mark): (Glyph, Glyph) = match accent(c) {
            Some((b, m)) => (glyph(b), m),
            None => (glyph(c), &[]),
        };
        for line in letter.iter().chain(mark) {
            out.push(line.iter().map(|&(gx, gy)| (x0 + f32::from(gx) * k, y - f32::from(gy) * k)).collect());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_case_is_drawn_as_capitals_and_unknown_characters_as_a_question_mark() {
        assert_eq!(strokes("living", 0.0, 0.0, 12.0), strokes("LIVING", 0.0, 0.0, 12.0));
        assert_eq!(strokes("€", 0.0, 0.0, 12.0), strokes("?", 0.0, 0.0, 12.0));
        assert!(strokes(" ", 0.0, 0.0, 12.0).is_empty());
    }

    #[test]
    fn accented_letters_are_their_base_letter_plus_an_accent() {
        let plain = strokes("A", 0.0, 0.0, 12.0);
        let acute = strokes("á", 0.0, 0.0, 12.0);
        assert_eq!(acute.len(), plain.len() + 1);
        assert_eq!(acute[..plain.len()], plain[..]);
    }

    #[test]
    fn text_is_scaled_to_its_height_from_the_baseline() {
        let s = strokes("I", 10.0, 100.0, 24.0);
        let ys: Vec<f32> = s.iter().flatten().map(|p| p.1).collect();
        assert_eq!(ys.iter().copied().fold(f32::MAX, f32::min), 76.0);
        assert_eq!(ys.iter().copied().fold(f32::MIN, f32::max), 100.0);
        assert_eq!(width("II", 12.0), 19.0);
    }

    /// The x every point of `text`'s strokes lies between, at 10 px from 0.
    fn reach(f: Lettering, text: &str) -> (f32, f32) {
        let xs: Vec<f32> = f.strokes(text, 0.0, 0.0, 10.0).into_iter().flatten().map(|p| p.0).collect();
        (xs.iter().copied().fold(f32::MAX, f32::min), xs.iter().copied().fold(f32::MIN, f32::max))
    }

    #[test]
    fn the_block_letters_stay_the_default_and_draw_as_they_did() {
        assert_eq!(Lettering::default(), Lettering::Block);
        assert_eq!(Lettering::Block.strokes("Sala 01", 3.0, 40.0, 12.0), strokes("Sala 01", 3.0, 40.0, 12.0));
        assert_eq!(Lettering::Block.width("Sala 01", 12.0), width("Sala 01", 12.0));
        for f in [Lettering::Block, Lettering::Iso] {
            assert_eq!(Lettering::from_name(f.name()), Some(f));
        }
        assert_eq!(Lettering::from_name("ISO"), Some(Lettering::Iso));
        assert_eq!(Lettering::from_name("comic"), None);
    }

    #[test]
    fn iso_lettering_keeps_lower_case_and_draws_the_marks_the_block_letters_lack() {
        let iso = Lettering::Iso;
        assert_ne!(iso.strokes("sala", 0.0, 0.0, 10.0), iso.strokes("SALA", 0.0, 0.0, 10.0));
        let unknown = iso.strokes("€", 0.0, 0.0, 10.0);
        assert_eq!(unknown, iso.strokes("?", 0.0, 0.0, 10.0));
        for c in "%;<>º³!*[]".chars() {
            assert_eq!(Lettering::Block.strokes(&c.to_string(), 0.0, 0.0, 12.0), strokes("?", 0.0, 0.0, 12.0), "{c}");
        }
        let printable = (' '..='~').filter(|c| !matches!(c, '?' | '@'));
        let marks = "%;<>º³!*[]ª¹²°±×÷·´Øø⌀–—";
        let accented = "ÁÀÂÃÄÉÈÊËÍÌÎÏÓÒÔÕÖÚÙÛÜÝÑÇáàâãäéèêëíìîïóòôõöúùûüýñç";
        for c in printable.chain(marks.chars()).chain(accented.chars()) {
            assert_ne!(iso.strokes(&c.to_string(), 0.0, 0.0, 10.0), unknown, "{c} has a shape of its own");
        }
        // An accented letter is its letter's strokes and the mark's; í loses i's dot.
        let a = iso.strokes("a", 0.0, 0.0, 10.0);
        let acute = iso.strokes("á", 0.0, 0.0, 10.0);
        assert_eq!((acute.len(), &acute[..a.len()]), (a.len() + 1, &a[..]));
        assert_eq!(iso.strokes("í", 0.0, 0.0, 10.0).len(), iso.strokes("i", 0.0, 0.0, 10.0).len());
    }

    #[test]
    fn iso_lettering_is_as_wide_as_its_letters_and_measures_what_it_draws() {
        let iso = Lettering::Iso;
        assert!(iso.width("iii", 10.0) < iso.width("mmm", 10.0) / 2.0);
        assert_eq!(iso.width("", 10.0), 0.0);
        // Capitals 10 px tall stand on the baseline; lower case is 7 px, with descenders to 3 below it.
        let ys = |t: &str| -> (f32, f32) {
            let ys: Vec<f32> = iso.strokes(t, 0.0, 0.0, 10.0).into_iter().flatten().map(|p| p.1).collect();
            (ys.iter().copied().fold(f32::MAX, f32::min), ys.iter().copied().fold(f32::MIN, f32::max))
        };
        assert_eq!(ys("HEL"), (-10.0, 0.0));
        assert_eq!(ys("xvz"), (-7.0, 0.0));
        assert_eq!(ys("p"), (-7.0, 3.0));
        // What it draws starts at x and ends at its width, within half a stroke: tables and title blocks lay out by it.
        for text in ["Planta de cobertura 1:100", "ÁREA 53,90 m²", "Ø 100 mm; 2% <máx.>", "H", "lll", "[3]"] {
            let (lo, hi) = reach(iso, text);
            let w = iso.width(text, 10.0);
            assert!(lo > -0.6 && lo < 0.6, "{text}: starts at {lo}");
            assert!((hi - w).abs() < 0.6, "{text}: ends at {hi}, {w} wide");
        }
        assert!((iso.width("Sala", 25.0) - 2.5 * iso.width("Sala", 10.0)).abs() < 1e-3);
    }
}
