//! Sheets: paper with a title block, and views placed on it at a scale. A sheet is drawn once,
//! at [`DENSITY`] px per paper mm, as a display list in paper pixels: what the PNG, the PDF and the
//! desktop's sheet view show. Each view on it is drawn as its own view draws it, moved onto the
//! paper and made the viewport's to pick, and titled under it; a section's picture of what lies
//! beyond, and the 3D view's, go in as images. A schedule is a ruled table under its title, the
//! size its lettering makes it. Text notes go on the paper itself.

use crate::geom::P2;
use crate::graphics::Look;
use crate::language::Language;
use crate::model::{Element, Id, Model, Schedule, Sheet, Viewport, paper};
use crate::render::{self, Drawing, PAPER, Prim, Rgb, View};
use crate::{font, schedule, view3d};

/// Paper px per mm in a sheet's drawing: plan lettering 10 px tall comes out 2 mm on paper.
pub const DENSITY: f64 = 5.0;
/// Room around what a view shows, on paper, for grid bubbles and the like: mm.
const ROOM: f64 = 12.0;
/// The title box in the bottom right corner, inside the border: mm.
const TITLE: P2 = P2(180.0, 40.0);
const INK: Rgb = Rgb(25, 25, 25);
/// A schedule's lettering and its title's, px: 2.5 mm and 3.5 mm on paper.
const CELL_TEXT: f32 = 12.5;
const TITLE_TEXT: f32 = 17.5;
/// A schedule's rows and its title's, and the room either side of a cell's text, px.
const ROW: f32 = 30.0;
const TITLE_ROW: f32 = 45.0;
const PAD: f32 = 10.0;

/// Paper mm (from the bottom left corner, y up) to the sheet drawing's pixels.
fn page(paper: P2) -> View {
    View {
        scale: DENSITY,
        cx: paper.0 / 2.0,
        cy: paper.1 / 2.0,
        width: (paper.0 * DENSITY).round() as u32,
        height: (paper.1 * DENSITY).round() as u32,
    }
}

/// View `view` at 1 : `scale` on paper: the frame its drawing gets, `ROOM` mm around what it shows
/// (a 3D view's picture, as large as the model comes out); a schedule's, its table's.
fn framing(m: &Model, view: Id, scale: f64) -> Option<View> {
    let k = DENSITY / scale;
    let pts: Vec<P2> = match m.elements.get(&view)? {
        Element::Level(_) => render::plan_extent(m, view),
        Element::Section(_) | Element::Elevation(_) => render::section_extent(m, view, k, 1.0)?.to_vec(),
        Element::Schedule(sc) => {
            let (width, height) = table_size(&sc.name, &table_of(m, sc));
            return Some(View { scale: 1.0, cx: 0.0, cy: 0.0, width, height });
        }
        Element::View3D(v) => {
            let cam = view3d::camera_in(m, v.azimuth, v.elevation, k, ROOM * DENSITY, &Look::of(m, view));
            return Some(View { scale: k, cx: 0.0, cy: 0.0, width: cam.width, height: cam.height });
        }
        _ => return None,
    };
    let (lo, hi) = match pts.first() {
        Some(&first) => pts.iter().fold((first, first), |(lo, hi), p| (P2(lo.0.min(p.0), lo.1.min(p.1)), P2(hi.0.max(p.0), hi.1.max(p.1)))),
        None => (P2(-5000.0, -5000.0), P2(5000.0, 5000.0)),
    };
    let side = |d: f64| ((d * k + 2.0 * ROOM * DENSITY).ceil().clamp(1.0, f64::from(u32::MAX))) as u32;
    Some(View { scale: k, cx: (lo.0 + hi.0) / 2.0, cy: (lo.1 + hi.1) / 2.0, width: side(hi.0 - lo.0), height: side(hi.1 - lo.1) })
}

/// How much paper view `view` takes at 1 : `scale`, mm.
pub fn size(m: &Model, view: Id, scale: f64) -> Option<P2> {
    framing(m, view, scale).map(|v| P2(f64::from(v.width) / DENSITY, f64::from(v.height) / DENSITY))
}

/// Viewport `id`'s view frame, and where that frame's top left corner sits in its sheet's drawing,
/// px: a point `p` of the view is at `frame.px(p)` plus that corner.
pub fn frame(m: &Model, id: Id) -> Option<(View, (f32, f32))> {
    let Some(Element::Viewport(vp)) = m.elements.get(&id) else { return None };
    let view = framing(m, vp.view, vp.scale)?;
    let (x, y) = page(paper(&m.sheet(vp.sheet)?.size)?).px(vp.at);
    Some((view, (x - view.width as f32 / 2.0, y - view.height as f32 / 2.0)))
}

/// Where view `view` at 1 : `scale` goes on sheet `sheet` by default: right of the views on it,
/// else in the middle of the paper above the title box; when there's no room left on the right,
/// in the paper's top right corner.
pub fn spot(m: &Model, sheet: Id, view: Id, scale: f64) -> P2 {
    let paper = m.sheet(sheet).and_then(|s| paper(&s.size)).unwrap_or(P2(420.0, 297.0));
    let middle = P2((20.0 + paper.0 - 10.0) / 2.0, (10.0 + TITLE.1 + paper.1 - 10.0) / 2.0);
    let taken = size(m, view, scale).unwrap_or(P2(0.0, 0.0));
    let right = m
        .elements
        .values()
        .filter_map(|e| match e {
            Element::Viewport(v) if v.sheet == sheet => Some(v.at.0 + size(m, v.view, v.scale)?.0 / 2.0),
            _ => None,
        })
        .fold(None, |r: Option<f64>, x| Some(r.map_or(x, |r| r.max(x))));
    // Inside the border, 5 mm clear of it.
    let (east, north) = (paper.0 - 15.0, paper.1 - 15.0);
    match right {
        None => middle,
        Some(r) if r + 10.0 + taken.0 <= east => P2(r + 10.0 + taken.0 / 2.0, middle.1),
        Some(_) => P2(east - taken.0 / 2.0, north - taken.1 / 2.0),
    }
}

/// `text` with its left end at paper point `at` (mm), capitals `height` mm tall.
fn left(d: &mut Drawing, text: &str, at: P2, height: f64) {
    let h = (height * DENSITY) as f32;
    let (x, y) = d.view.px(at);
    d.text(text, (x + font::width(text, h) / 2.0, y), h, INK);
}

/// `text` with its right end at paper point `at` (mm), capitals `height` mm tall.
fn right(d: &mut Drawing, text: &str, at: P2, height: f64) {
    let h = (height * DENSITY) as f32;
    let (x, y) = d.view.px(at);
    d.text(text, (x - font::width(text, h) / 2.0, y), h, INK);
}

/// `text` broken into lines at most `width` mm long, capitals `height` mm tall; its own line
/// breaks kept.
fn wrap(text: &str, width: f64, height: f64) -> Vec<String> {
    let h = (height * DENSITY) as f32;
    let fits = |s: &str| f64::from(font::width(s, h)) / DENSITY <= width;
    let mut lines = Vec::new();
    for para in text.lines() {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let longer = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if fits(&longer) || line.is_empty() {
                line = longer;
            } else {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            }
        }
        lines.push(line);
    }
    lines
}

fn rect(a: P2, b: P2) -> Vec<P2> {
    vec![a, P2(b.0, a.1), b, P2(a.0, b.1)]
}

/// The title blocks a sheet may have besides BIMCraft's own.
pub const TITLE_BLOCKS: [&str; 1] = ["umuarama"];
/// The fields of Umuarama's title block, in the order its legend box shows them.
pub const FIELDS: [&str; 15] = [
    "tipo",
    "conteudo",
    "proprietario",
    "cpf",
    "zona",
    "endereco",
    "autor",
    "autor_registro",
    "responsavel",
    "responsavel_registro",
    "escala",
    "data",
    "desenhista",
    "areas",
    "declaracao",
];
/// How long a field may be: the table of areas runs to several lines.
pub const MAX_FIELD: usize = 1024;
/// Umuarama's legend box (LC 436/2017, art. 37 §2): 170 x 270 mm, its stamp at most 90 mm tall,
/// the sheet number in a 35 mm square, and 60 mm left for the city's approval.
const LEGEND: P2 = P2(170.0, 270.0);
const STAMP: f64 = 90.0;
const NUMBER_BOX: f64 = 35.0;
const CITY: f64 = 60.0;
/// What the code has the owner declare, unless `declaracao` says otherwise.
const DECLARATION: &str = "Declaro que a aprovação do projeto não implica no reconhecimento, por parte da Prefeitura, \
    do direito de propriedade ou de posse do terreno.";

/// Whether sheet `sh`, on `paper`, has a title block BIMCraft draws and fields it has.
pub fn check_title_block(sh: &Sheet, paper: P2) -> Result<(), String> {
    match sh.title_block.as_deref() {
        None if sh.fields.is_empty() => Ok(()),
        None => Err("its `fields` belong to a title block; give it one (`title_block`: umuarama)".into()),
        Some(t) if !TITLE_BLOCKS.contains(&t) => Err(format!("title block `{t}` isn't one there is ({})", TITLE_BLOCKS.join(", "))),
        Some(_) => {
            if let Some(k) = sh.fields.keys().find(|k| !FIELDS.contains(&k.as_str())) {
                return Err(format!("the title block has no field `{k}` (it has {})", FIELDS.join(", ")));
            }
            if let Some((k, _)) = sh.fields.iter().find(|(_, v)| v.chars().count() > MAX_FIELD) {
                return Err(format!("its field `{k}` is longer than {MAX_FIELD} characters"));
            }
            if paper.1 < LEGEND.1 + 20.0 {
                return Err(format!("Umuarama's legend box is {} mm tall: it needs A3 or larger, not {}", LEGEND.1, sh.size));
            }
            Ok(())
        }
    }
}

/// Umuarama's legend box in the bottom right corner (LC 436/2017, art. 37 §2): from the top,
/// the room for the city's approval, the owner's statement, the table of areas, and the stamp
/// (the kind of project and what the sheet shows, the owner, the zone and address, the
/// professionals, and the scale, date and drafter, beside the sheet number's box).
fn umuarama(d: &mut Drawing, sheet: &Sheet, paper: P2, scale: &str) {
    let field = |k: &str| sheet.fields.get(k).map_or("", String::as_str);
    let (x1, y0) = (paper.0 - 10.0, 10.0);
    let (x0, y1) = (x1 - LEGEND.0, y0 + LEGEND.1);
    d.line(&rect(P2(x0, y0), P2(x1, y1)), 2.5, INK, true);
    let hr = |d: &mut Drawing, y: f64, a: f64, b: f64, w: f32| d.line(&[P2(a, y), P2(b, y)], w, INK, false);
    let vr = |d: &mut Drawing, x: f64, a: f64, b: f64| d.line(&[P2(x, a), P2(x, b)], 1.0, INK, false);
    let label = |d: &mut Drawing, s: &str, x: f64, top: f64| left(d, s, P2(x + 2.0, top - 2.5), 1.6);

    // The stamp, from its foot: the scale, date and drafter in 7 mm strips; the professionals;
    // the zone and the address; the owner; the kind of project and what the sheet shows, beside
    // the sheet number.
    let ys = y0 + STAMP;
    hr(d, ys, x0, x1, 2.5);
    let strip = y0 + 7.0;
    hr(d, strip, x0, x1, 1.0);
    let third = LEGEND.0 / 3.0;
    let escala = if field("escala").is_empty() { scale.to_string() } else { format!("ESCALA {}", field("escala")) };
    left(d, &escala, P2(x0 + 2.0, y0 + 3.5), 2.0);
    vr(d, x0 + third, y0, strip);
    left(d, &format!("DATA: {}", field("data")), P2(x0 + third + 2.0, y0 + 3.5), 2.0);
    vr(d, x0 + 2.0 * third, y0, strip);
    left(d, &format!("DESENHO: {}", field("desenhista")), P2(x0 + 2.0 * third + 2.0, y0 + 3.5), 2.0);

    let pros = strip + 36.0;
    hr(d, pros, x0, x1, 1.0);
    let half = x0 + LEGEND.0 / 2.0;
    vr(d, half, strip, pros);
    for (x, title, name, reg) in [
        (x0, "AUTOR DO PROJETO", field("autor"), field("autor_registro")),
        (half, "RESPONSÁVEL TÉCNICO PELA EXECUÇÃO", field("responsavel"), field("responsavel_registro")),
    ] {
        label(d, title, x, pros);
        left(d, name, P2(x + 2.0, pros - 10.0), 2.5);
        left(d, reg, P2(x + 2.0, pros - 16.0), 2.0);
        d.line(&[P2(x + 8.0, strip + 8.0), P2(x + LEGEND.0 / 2.0 - 8.0, strip + 8.0)], 0.8, INK, false);
        left(d, "ASSINATURA", P2(x + 8.0, strip + 5.0), 1.6);
    }

    let place = pros + 12.0;
    hr(d, place, x0, x1 - NUMBER_BOX, 1.0);
    vr(d, x0 + 40.0, pros, place);
    label(d, "ZONA", x0, place);
    left(d, field("zona"), P2(x0 + 2.0, pros + 4.0), 2.5);
    label(d, "ENDEREÇO", x0 + 40.0, place);
    for (i, line) in wrap(field("endereco"), LEGEND.0 - 40.0 - NUMBER_BOX - 4.0, 2.0).iter().take(2).enumerate() {
        left(d, line, P2(x0 + 42.0, pros + 6.0 - 3.0 * i as f64), 2.0);
    }

    // The sheet number's square, in the stamp's top right corner.
    let (nx, ny) = (x1 - NUMBER_BOX, ys - NUMBER_BOX);
    d.line(&rect(P2(nx, ny), P2(x1, ys)), 1.5, INK, true);
    label(d, "FOLHA", nx, ys);
    let number = (7.0 * DENSITY) as f32;
    let at = d.view.px(P2(nx + NUMBER_BOX / 2.0, ny + NUMBER_BOX / 2.0 - 2.0));
    d.text(&sheet.number, at, number, INK);
    if pros + 12.0 < ny {
        vr(d, nx, place, ny);
    }

    let owner = place + (ys - place) * 0.47;
    hr(d, owner, x0, nx, 1.0);
    label(d, "PROPRIETÁRIO", x0, owner);
    left(d, field("proprietario"), P2(x0 + 2.0, owner - 7.5), 2.5);
    left(d, &format!("CPF: {}", field("cpf")), P2(x0 + 2.0, owner - 12.5), 2.0);
    d.line(&[P2(x0 + 75.0, place + 4.0), P2(nx - 4.0, place + 4.0)], 0.8, INK, false);
    left(d, "ASSINATURA", P2(x0 + 75.0, place + 1.8), 1.6);
    let kind = if field("tipo").is_empty() { sheet.name.as_str() } else { field("tipo") };
    left(d, kind, P2(x0 + 2.0, ys - 6.0), 5.0);
    for (i, line) in wrap(field("conteudo"), nx - x0 - 4.0, 2.5).iter().take(2).enumerate() {
        left(d, line, P2(x0 + 2.0, ys - 11.5 - 3.6 * i as f64), 2.5);
    }

    // Above the stamp: the table of areas, the owner's statement, and the city's room.
    let city = y1 - CITY;
    hr(d, city, x0, x1, 1.5);
    label(d, "ESPAÇO RESERVADO À PREFEITURA E DEMAIS ÓRGÃOS", x0, y1);
    let statement = if field("declaracao").is_empty() { DECLARATION } else { field("declaracao") };
    let lines = wrap(statement, LEGEND.0 - 4.0, 2.2);
    let said = city - 6.0 - 3.6 * lines.len() as f64;
    hr(d, said, x0, x1, 1.5);
    for (i, line) in lines.iter().enumerate() {
        left(d, line, P2(x0 + 2.0, city - 4.0 - 3.6 * i as f64), 2.2);
    }
    label(d, "QUADRO DE ÁREAS", x0, said);
    let mut y = said - 9.0;
    for row in field("areas").lines().filter(|r| !r.trim().is_empty()) {
        if y < ys + 4.0 {
            break;
        }
        match row.rsplit_once(':') {
            Some((what, value)) => {
                left(d, what.trim(), P2(x0 + 2.0, y), 2.5);
                right(d, value.trim(), P2(x1 - 2.0, y), 2.5);
            }
            None => left(d, row.trim(), P2(x0 + 2.0, y), 2.5),
        }
        hr(d, y - 3.0, x0 + 2.0, x1 - 2.0, 0.5);
        y -= 7.0;
    }
}

/// The border, 20 mm in on the left (for binding) and 10 mm on the other sides, and the title
/// box in the bottom right corner: the project, the sheet's name, its scale, and its number; or
/// the title block the sheet names.
fn title_block(d: &mut Drawing, project: &str, sheet: &Sheet, paper: P2, scale: &str) {
    d.line(&rect(P2(20.0, 10.0), P2(paper.0 - 10.0, paper.1 - 10.0)), 2.5, INK, true);
    if sheet.title_block.is_some() {
        umuarama(d, sheet, paper, scale);
        return;
    }
    let width = TITLE.0.min(paper.0 - 30.0);
    let (x0, x1, y0, y1) = (paper.0 - 10.0 - width, paper.0 - 10.0, 10.0, 10.0 + TITLE.1);
    d.line(&rect(P2(x0, y0), P2(x1, y1)), 2.5, INK, true);
    d.line(&[P2(x0, y0 + 26.0), P2(x1, y0 + 26.0)], 1.0, INK, false);
    d.line(&[P2(x1 - 50.0, y0), P2(x1 - 50.0, y0 + 26.0)], 1.0, INK, false);
    left(d, project, P2(x0 + 4.0, y0 + 33.0), 5.0);
    left(d, &sheet.name, P2(x0 + 4.0, y0 + 17.0), 3.5);
    left(d, scale, P2(x0 + 4.0, y0 + 7.0), 2.0);
    let number = (7.0 * DENSITY) as f32;
    let at = d.view.px(P2(x1 - 25.0, y0 + 13.0));
    d.text(&sheet.number, at, number, INK);
}

/// A schedule's table: its headings; its rows' cells as shown, its totals' last; each column's
/// width, px; and whether it has totals.
struct Table {
    headings: Vec<String>,
    rows: Vec<Vec<String>>,
    widths: Vec<f32>,
    totals: bool,
}

fn table_of(m: &Model, sc: &Schedule) -> Table {
    let t = schedule::table(m, sc).unwrap_or(schedule::Table { columns: Vec::new(), rows: Vec::new(), ids: Vec::new(), totals: None });
    let said = |c: &schedule::Column, v: &serde_json::Value| {
        let s = schedule::shown(m.units, c.kind, v);
        if v.is_boolean() { m.language.word(&s).to_string() } else { s }
    };
    let mut rows: Vec<Vec<String>> = t.rows.iter().map(|r| r.iter().zip(&t.columns).map(|(v, c)| said(c, v)).collect()).collect();
    if let Some(totals) = &t.totals {
        let mut row: Vec<String> = totals.iter().zip(&t.columns).map(|(v, c)| said(c, v)).collect();
        if let Some(first) = row.first_mut().filter(|f| f.is_empty()) {
            *first = m.language.word("Total").to_string();
        }
        rows.push(row);
    }
    let headings: Vec<String> = t
        .columns
        .iter()
        .map(|c| schedule::heading(m.units, &schedule::Column { heading: m.language.word(&c.heading).to_string(), kind: c.kind }))
        .collect();
    let widths = (0..headings.len())
        .map(|i| {
            let widest = headings.get(i).map_or(0.0, |c| font::width(c, CELL_TEXT));
            rows.iter().filter_map(|r| r.get(i)).map(|c| font::width(c, CELL_TEXT)).fold(widest, f32::max) + 2.0 * PAD
        })
        .collect();
    Table { headings, rows, widths, totals: t.totals.is_some() }
}

fn table_size(name: &str, t: &Table) -> (u32, u32) {
    let width = t.widths.iter().sum::<f32>().max(2.0 * PAD + font::width(name, TITLE_TEXT));
    (width.ceil() as u32, (TITLE_ROW + ROW * (t.rows.len() + 1) as f32).ceil() as u32)
}

/// Schedule `sc` as a ruled table from its top left corner at `(x, y)` px: its title, then its
/// headings and its rows, a column each, its totals under a heavier rule.
fn table(d: &mut Drawing, m: &Model, sc: &Schedule, (x, y): (f32, f32)) {
    let t = table_of(m, sc);
    let (w, h) = table_size(&sc.name, &t);
    // On whole pixels, so its rules come out crisp.
    let (x, y, w, h) = (x.round(), y.round(), w as f32, h as f32);
    let rule = |d: &mut Drawing, a: (f32, f32), b: (f32, f32), width: f32| {
        d.push(Prim::Line { pts: vec![a, b], width, color: INK, dash: None, closed: false });
    };
    d.push(Prim::Line { pts: vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)], width: 2.0, color: INK, dash: None, closed: true });
    d.text(&sc.name, (x + w / 2.0, y + TITLE_ROW / 2.0), TITLE_TEXT, INK);
    let top = y + TITLE_ROW;
    rule(d, (x, top), (x + w, top), 1.0);
    let last = t.rows.len();
    for (i, cells) in std::iter::once(&t.headings).chain(&t.rows).enumerate() {
        let row = top + ROW * i as f32;
        if i > 0 {
            rule(d, (x, row), (x + w, row), if i == 1 || (t.totals && i == last) { 1.5 } else { 0.5 });
        }
        let mut left = x;
        for (text, width) in cells.iter().zip(&t.widths) {
            d.text(text, (left + PAD + font::width(text, CELL_TEXT) / 2.0, row + ROW / 2.0), CELL_TEXT, INK);
            left += width;
        }
    }
    let mut left = x;
    for width in t.widths.iter().take(t.widths.len().saturating_sub(1)) {
        left += width;
        rule(d, (left, top), (left, y + h), 0.5);
    }
}

/// The scale a sheet's title box gives, in `language`: its views', when they share one.
fn scale_note(views: &[&Viewport], language: Language) -> String {
    match views.split_first() {
        None => String::new(),
        Some((first, rest)) if rest.iter().all(|v| v.scale == first.scale) => format!("{} 1 : {}", language.word("SCALE"), first.scale),
        Some(_) => language.word("SCALE AS INDICATED").into(),
    }
}

/// Sheet `id` on its paper: its title block, then each view on it, at its scale and titled.
pub fn draw(m: &Model, id: Id) -> Option<Drawing> {
    let sheet = m.sheet(id)?;
    let paper = paper(&sheet.size)?;
    let views: Vec<(Id, &Viewport)> = m
        .elements
        .iter()
        .filter_map(|(&vid, e)| match e {
            Element::Viewport(v) if v.sheet == id => Some((vid, v)),
            _ => None,
        })
        .collect();
    let mut d = Drawing::new(page(paper));
    d.own(Some(id));
    // Schedules keep their own size: no scale of theirs.
    let scaled: Vec<&Viewport> =
        views.iter().filter(|(_, v)| !matches!(m.elements.get(&v.view), Some(Element::Schedule(_)))).map(|(_, v)| *v).collect();
    title_block(&mut d, &m.name, sheet, paper, &scale_note(&scaled, m.language));
    let mut scene = None;
    for (vid, vp) in views {
        let Some((view, (ox, oy))) = frame(m, vid) else { continue };
        d.own(Some(vid));
        // Paper under it, its own: a click anywhere over a view picks it.
        let (w, h) = (view.width as f32, view.height as f32);
        d.push(Prim::Fill { rings: vec![vec![(ox, oy), (ox + w, oy), (ox + w, oy + h), (ox, oy + h)]], color: PAPER });
        let inner = match m.elements.get(&vp.view) {
            Some(Element::Level(_)) => render::draw_at(m, vp.view, view, 1.0),
            Some(Element::Section(_) | Element::Elevation(_)) => {
                let scene = scene.get_or_insert_with(|| view3d::Scene::new(m));
                if let Some(img) = render::section_image(m, scene, vp.view, view, &[]) {
                    d.push(Prim::Image { at: (ox, oy), width: img.width, height: img.height, rgba: img.rgba });
                }
                render::section_at(m, vp.view, view, 1.0)
            }
            // Its title is its first row; it has no view title under it.
            Some(Element::Schedule(sc)) => {
                table(&mut d, m, sc, (ox, oy));
                continue;
            }
            Some(Element::View3D(v)) => {
                let cam = view3d::camera_in(m, v.azimuth, v.elevation, view.scale, ROOM * DENSITY, &Look::of(m, vp.view));
                let img = scene.get_or_insert_with(|| view3d::Scene::new(m)).render_in(&cam, &[], &Look::of(m, vp.view));
                d.push(Prim::Image { at: (ox, oy), width: img.width, height: img.height, rgba: img.rgba });
                Drawing::new(view)
            }
            _ => continue,
        };
        // Its own title, owned by no element, gives way to the viewport's.
        for (prim, owner) in inner.prims.into_iter().zip(inner.owners) {
            if owner.is_some() {
                d.push(prim.moved(ox, oy));
            }
        }
        // The title, under it: the view's name over a line, its scale under that.
        let name = m.view_name(vp.view).unwrap_or_default();
        let corner = d.view.mm((ox, oy + view.height as f32));
        let x = corner.0 + ROOM;
        left(&mut d, name, P2(x, corner.1 - 5.0), 3.0);
        let line = f64::from(font::width(name, (3.0 * DENSITY) as f32)) / DENSITY + 10.0;
        d.line(&[P2(x, corner.1 - 8.0), P2(x + line, corner.1 - 8.0)], 1.5, INK, false);
        left(&mut d, &format!("1 : {}", vp.scale), P2(x, corner.1 - 11.0), 2.0);
    }
    for (&nid, e) in &m.elements {
        if let Element::Text(n) = e
            && n.view == id
        {
            d.own(Some(nid));
            render::note(&mut d, n);
        }
    }
    d.own(None);
    Some(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Session;
    use crate::geom::P2;
    use crate::render::{POCHE, png};

    fn pixel(png: &[u8], (x, y): (f32, f32)) -> (u8, u8, u8) {
        let pm = tiny_skia::Pixmap::decode_png(png).unwrap();
        let c = pm.pixel(x as u32, y as u32).unwrap();
        (c.red(), c.green(), c.blue())
    }

    #[test]
    fn sheets_draw_a_title_block_and_their_views_at_scale() {
        let mut s = Session::new();
        s.script(
            r#"wall {"points": [[0,0],[8000,0],[8000,6000],[0,6000]], "closed": true}
               sheet {"name": "Plans"}
               place {"sheet": "A101", "view": 1}"#,
        )
        .unwrap();
        // Walls 3 to 6, the sheet 7, its viewport 8.
        let d = draw(&s.model, 7).unwrap();
        let png = png(&d).unwrap();
        let dark = |at: (f32, f32)| {
            let (r, g, b) = pixel(&png, at);
            r < 120 && g < 120 && b < 120
        };
        assert!(dark(d.view.px(P2(20.0, 150.0))), "the border, 20 mm in from the left");
        assert!(dark(d.view.px(P2(415.0 - 5.0, 10.0))) || dark(d.view.px(P2(410.0, 10.0))), "the border's bottom right");
        // The plan at 1 : 100, where the viewport puts it: a wall in poché.
        let (v, (ox, oy)) = frame(&s.model, 8).unwrap();
        let at = |p: P2| {
            let (x, y) = v.px(p);
            (x + ox, y + oy)
        };
        assert_eq!(pixel(&png, at(P2(4000.0, 0.0))), (POCHE.0, POCHE.1, POCHE.2));
        assert!((v.scale - DENSITY / 100.0).abs() < 1e-12, "1 : 100");
        // What a viewport draws is the viewport's to pick; the title block is the sheet's.
        assert_eq!(d.pick(at(P2(4000.0, 0.0)), 2.0), Some(8));
        assert_eq!(d.pick(d.view.px(P2(20.0, 150.0)), 2.0), Some(7));
    }

    #[test]
    fn umuaramas_title_block_is_the_legend_box_its_building_code_asks_for() {
        let mut s = Session::new();
        s.script(
            r#"sheet {"name": "Projeto", "size": "A2", "title_block": "umuarama", "fields": {"tipo": "Projeto arquitetônico", "areas": "Lote: 140,00 m²\nA construir: 53,90 m²"}}"#,
        )
        .unwrap();
        let d = draw(&s.model, 3).unwrap();
        let png = png(&d).unwrap();
        let dark = |p: P2| {
            let (r, g, b) = pixel(&png, d.view.px(p));
            r < 120 && g < 120 && b < 120
        };
        // A2 is 594 x 420 mm: the legend box is 170 mm wide and 270 mm tall in the bottom right
        // corner, inside the border, its stamp at most 90 mm tall at its foot.
        let (x0, x1) = (594.0 - 10.0 - 170.0, 594.0 - 10.0);
        assert!(dark(P2(x0, 200.0)), "the legend box's left side");
        assert!(dark(P2(x0 + 85.0, 280.0)), "its top, 270 mm up from the border");
        assert!(dark(P2(x0 + 60.0, 100.0)), "the stamp's top, 90 mm up");
        assert!(dark(P2(x1 - 35.0, 80.0)), "the sheet number's 35 mm box");
        assert!(!dark(P2(x0 + 85.0, 290.0)), "nothing above the legend box");
        assert!(!dark(P2(x1 - 175.0, 30.0)), "and none of BIMCraft's own title box");
    }

    #[test]
    fn a_title_block_says_what_it_takes() {
        let mut s = Session::new();
        let err = s.script(r#"sheet {"title_block": "curitiba"}"#).unwrap_err();
        assert!(err.contains("umuarama"), "{err}");
        let err = s.script(r#"sheet {"title_block": "umuarama", "fields": {"proprietária": "x"}}"#).unwrap_err();
        assert!(err.contains("proprietario"), "the fields there are: {err}");
        let err = s.script(r#"sheet {"title_block": "umuarama", "size": "A4"}"#).unwrap_err();
        assert!(err.contains("A3"), "{err}");
        let err = s.script(r#"sheet {"title_block": "umuarama", "fields": {"tipo": 3}}"#).unwrap_err();
        assert!(err.contains("fields"), "{err}");
        // A sheet's fields change as any of its fields do.
        s.script(r#"sheet {"title_block": "umuarama", "fields": {"tipo": "Projeto"}}"#).unwrap();
        s.script(r#"modify {"id": 3, "fields": {"data": "Outubro de 2026"}}"#).unwrap();
        let sh = s.model.sheet(3).unwrap();
        assert_eq!(sh.fields.get("tipo").map(String::as_str), Some("Projeto"));
        assert_eq!(sh.fields.get("data").map(String::as_str), Some("Outubro de 2026"));
    }

    #[test]
    fn a_schedule_on_a_sheet_shows_its_own_fields_and_its_totals_in_the_projects_language() {
        let mut s = Session::new();
        s.script(
            r#"wall {"points": [[0,0],[8000,0],[8000,6000],[0,6000]], "closed": true}
               room {"point": [4000, 3000], "name": "Living"}
               sheet {"name": "Areas"}
               schedule {"name": "Room Areas", "category": "rooms", "fields": ["Name", {"name": "Area", "total": true}, {"name": "Big", "formula": "Area > 40", "kind": "yesno"}]}
               place {"sheet": "A101", "view": "Room Areas", "at": [100, 220]}"#,
        )
        .unwrap();
        let sc = s.model.elements.values().find_map(|e| if let Element::Schedule(sc) = e { Some(sc.clone()) } else { None }).unwrap();
        let t = table_of(&s.model, &sc);
        assert_eq!(t.headings, ["Name", "Area (m²)", "Big"]);
        assert_eq!(t.rows, [vec!["Living", "48.00", "Yes"], vec!["Total", "48.00", ""]]);
        assert!(t.totals);
        // A row more than its rows and headings: its totals.
        let vp = s.model.elements.iter().find_map(|(id, e)| matches!(e, Element::Viewport(_)).then_some(*id)).unwrap();
        let (frame, _) = frame(&s.model, vp).unwrap();
        assert_eq!(frame.height as f32, TITLE_ROW + 3.0 * ROW);
        s.execute("language", &serde_json::json!({"code": "pt"})).unwrap();
        let t = table_of(&s.model, &sc);
        assert_eq!((t.headings[1].as_str(), t.rows[0][2].as_str(), t.rows[1][0].as_str()), ("Área (m²)", "Sim", "Total"));
    }

    #[test]
    fn schedules_3d_views_and_notes_go_on_sheets() {
        let mut s = Session::new();
        s.script(
            r#"wall {"points": [[0,0],[8000,0],[8000,6000],[0,6000]], "closed": true}
               room {"point": [4000, 3000], "name": "Living"}
               sheet {"name": "Schedules"}
               place {"sheet": "A101", "view": "Room Schedule", "at": [100, 220]}
               place {"sheet": "A101", "view": "{3D}", "at": [300, 150], "scale": 200}
               text {"view": "A101", "text": "Not for construction", "at": [30, 40]}"#,
        )
        .unwrap();
        // Walls 3 to 6, the room 7, the sheet 8; the schedule 9 and its viewport 10; the 3D view 11
        // and its viewport 12; the note 13.
        let d = draw(&s.model, 8).unwrap();
        let png = png(&d).unwrap();
        let dark = |at: (f32, f32)| {
            let (r, g, b) = pixel(&png, at);
            r < 120 && g < 120 && b < 120
        };
        // The schedule's rules: its outline's top left corner, and the line under its headings.
        let (table, (ox, oy)) = frame(&s.model, 10).unwrap();
        assert!(table.width > 200 && table.height > 100, "{table:?}");
        assert!(dark((ox + 0.5, oy + 0.5)), "its outline");
        assert_eq!(d.pick((ox + 4.0, oy + 4.0), 2.0), Some(10));
        // The 3D view's picture: shaded faces, not paper, somewhere in its frame.
        let (picture, (x3, y3)) = frame(&s.model, 12).unwrap();
        let pm = tiny_skia::Pixmap::decode_png(&png).unwrap();
        let shaded = (0..picture.width)
            .flat_map(|x| (0..picture.height).map(move |y| (x, y)))
            .filter_map(|(x, y)| pm.pixel(x3 as u32 + x, y3 as u32 + y))
            .filter(|c| (c.red(), c.green(), c.blue()) != (255, 255, 255))
            .count();
        assert!(shaded > 100, "{shaded}");
        // The note, on the paper.
        let at = d.view.px(P2(30.0, 40.0));
        assert_eq!(d.pick((at.0 + 1.0, at.1 + 6.0), 2.0), Some(13));
    }

    #[test]
    fn a_3d_view_on_a_sheet_is_drawn_in_its_visual_style() {
        let mut s = Session::new();
        s.script(
            r#"wall {"points": [[0,0],[8000,0],[8000,6000],[0,6000]], "closed": true}
               sheet {"name": "Views"}
               place {"sheet": "A101", "view": "{3D}", "at": [300, 150], "scale": 200}"#,
        )
        .unwrap();
        // Walls 3 to 6, the sheet 7, the 3D view 8 and its viewport 9.
        let inked = |s: &Session| {
            let d = draw(&s.model, 7).unwrap();
            let (picture, (x, y)) = frame(&s.model, 9).unwrap();
            let pm = tiny_skia::Pixmap::decode_png(&png(&d).unwrap()).unwrap();
            (0..picture.width)
                .flat_map(|px| (0..picture.height).map(move |py| (px, py)))
                .filter_map(|(px, py)| pm.pixel(x as u32 + px, y as u32 + py))
                .filter(|c| (c.red(), c.green(), c.blue()) != (255, 255, 255))
                .count()
        };
        let shaded = inked(&s);
        s.execute("visual_style", &serde_json::json!({"view": 8, "style": "wireframe"})).unwrap();
        let wire = inked(&s);
        assert!(wire > 50 && wire < shaded / 4, "{wire} of {shaded}");
    }
}
