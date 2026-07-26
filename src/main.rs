//! particles — the Standard Model, and a zoom from an atom down to the
//! quarks, for the Fe2O3 suite.
//!
//! Two views. TABLE is the familiar chart: three generations of quarks
//! and leptons, the force carriers, the Higgs, each with its numbers and
//! its full Wikipedia article from a local cache. ZOOM descends through
//! atom → nucleus → nucleon → quark as a rotatable 3D point cloud drawn
//! in braille, with the scale and the physics named at every step.
//!
//! Rotation is driven by keys, never by a timer: idle costs nothing.

mod canvas;
mod data;
mod fetch;
mod models;

use canvas::{Canvas, View3};
use crust::style;
use crust::{Crust, Cursor, Input, Pane};
use data::{Kind, Particle, PARTICLES};
use models::LEVELS;
use std::io::Write;

const RUST_RGB: (u8, u8, u8) = (247, 76, 0);
const HEAD_RGB: (u8, u8, u8) = (247, 140, 60);
const ERR_RGB: (u8, u8, u8) = (255, 120, 120);
const ASK_RGB: (u8, u8, u8) = (120, 200, 255);
const BAR_BG: (u8, u8, u8) = (38, 38, 38);

const GRID_X: u16 = 10; // first column of the chart
const GRID_Y: u16 = 4; // first row of the chart
const CELL_W: u16 = 12;
const CELL_H: u16 = 3;
const SIDE_X: u16 = GRID_X + 6 * CELL_W + 3;
const SIDE_MIN: u16 = SIDE_X + 40;
const DETAIL_Y: u16 = GRID_Y + 4 * CELL_H + 2;
/// In the zoom view the model gets the room the chart would have used.
const ZOOM_BOTTOM: u16 = 12;

#[derive(PartialEq, Clone, Copy)]
enum View {
    Table,
    Zoom,
}

#[derive(PartialEq, Clone, Copy)]
enum Pane2 {
    Article,
    Help,
    Chat,
}

struct App {
    sel: usize,
    view: View,
    pane: Pane2,
    level: usize,
    yaw: f64,
    pitch: f64,
    cache: data::Cache,
    chat: Vec<(String, String)>,
}

fn main() {
    let mut force_fetch = false;
    let mut start: Option<String> = None;
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "--fetch" => force_fetch = true,
            "-h" | "--help" => {
                println!("particles — Standard Model explorer (Fe2O3 suite)");
                println!();
                println!("Usage: particles [PARTICLE] [--fetch]");
                println!();
                println!("  PARTICLE    start on a particle (symbol or name)");
                println!("  --fetch     re-fetch the Wikipedia articles");
                println!("  -v          print version");
                println!();
                println!("Articles cache at ~/.particles/particles.json; the UI works offline.");
                return;
            }
            "-v" | "--version" => {
                println!("particles {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            other => start = Some(other.to_string()),
        }
    }

    let cache = if force_fetch { None } else { data::load() };
    let cache = match cache {
        Some(c) => c,
        None => {
            println!("particles: fetching articles (one-time) …");
            match fetch::fetch_all() {
                Ok(c) => {
                    if let Err(e) = data::save(&c) {
                        eprintln!("particles: could not save cache: {e}");
                    }
                    c
                }
                Err(e) => {
                    eprintln!("particles: fetch failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    };

    let mut sel = 0usize;
    if let Some(q) = start {
        match data::find(&q) {
            Some(i) => sel = i,
            None => {
                eprintln!("particles: no particle matches '{q}'");
                std::process::exit(1);
            }
        }
    }

    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        let text = detail_text(&PARTICLES[sel], &cache, false);
        if std::io::stdout().is_terminal() {
            println!("{text}");
        } else {
            println!("{}", crust::strip_ansi(&text));
        }
        return;
    }

    let mut app = App {
        sel,
        view: View::Table,
        pane: Pane2::Article,
        level: 0,
        yaw: 0.6,
        pitch: 0.35,
        cache,
        chat: Vec::new(),
    };

    Crust::init();
    Crust::set_app_identity("Particles");
    let (mut cols, mut rows) = Crust::terminal_size();
    let mut detail = Pane::new(1, DETAIL_Y, cols, rows.saturating_sub(DETAIL_Y).max(1), 253, 0);
    let mut status = Pane::new(1, rows, cols, 1, 250, 236);
    status.scroll = false;

    draw_all(&app, &mut detail, &mut status, cols, rows);

    loop {
        let key = match Input::getchr(None) {
            Some(k) => k,
            None => continue,
        };
        match key.as_str() {
            "q" => break,
            "ESC" => {
                if app.pane == Pane2::Article {
                    break;
                }
                app.pane = Pane2::Article;
                set_detail(&app, &mut detail, cols);
            }
            "TAB" => {
                app.view = if app.view == View::Table { View::Zoom } else { View::Table };
                draw_all(&app, &mut detail, &mut status, cols, rows);
            }
            // ── zoom view: rotate and descend ───────────────────────
            "LEFT" | "h" if app.view == View::Zoom => {
                app.yaw -= 0.22;
                draw_zoom(&app, cols, rows);
            }
            "RIGHT" | "l" if app.view == View::Zoom => {
                app.yaw += 0.22;
                draw_zoom(&app, cols, rows);
            }
            "UP" | "k" if app.view == View::Zoom => {
                app.pitch = (app.pitch + 0.18).clamp(-1.4, 1.4);
                draw_zoom(&app, cols, rows);
            }
            "DOWN" | "j" if app.view == View::Zoom => {
                app.pitch = (app.pitch - 0.18).clamp(-1.4, 1.4);
                draw_zoom(&app, cols, rows);
            }
            "+" | "=" if app.view == View::Zoom => {
                if app.level + 1 < LEVELS.len() {
                    app.level += 1;
                    draw_all(&app, &mut detail, &mut status, cols, rows);
                }
            }
            "-" | "_" if app.view == View::Zoom => {
                if app.level > 0 {
                    app.level -= 1;
                    draw_all(&app, &mut detail, &mut status, cols, rows);
                }
            }
            // ── table view: walk the chart ──────────────────────────
            "LEFT" | "h" => step(&mut app, (-1, 0), &mut detail, cols),
            "RIGHT" | "l" => step(&mut app, (1, 0), &mut detail, cols),
            "UP" | "k" => step(&mut app, (0, -1), &mut detail, cols),
            "DOWN" | "j" => step(&mut app, (0, 1), &mut detail, cols),
            "<" | "p" => {
                let t = app.sel.saturating_sub(1);
                select(&mut app, t, &mut detail, cols);
            }
            ">" | "n" => {
                let t = (app.sel + 1).min(PARTICLES.len() - 1);
                select(&mut app, t, &mut detail, cols);
            }
            "J" | "S-DOWN" => detail.linedown(),
            "K" | "S-UP" => detail.lineup(),
            " " | "PgDOWN" => detail.pagedown(),
            "PgUP" => detail.pageup(),
            "g" | "HOME" => detail.top(),
            "G" | "END" => detail.bottom(),
            "/" => {
                let q = status.ask_or_cancel("Find particle: ", "");
                print!("{}", Cursor::hide_seq());
                std::io::stdout().flush().ok();
                match q.as_deref().map(data::find) {
                    Some(Some(i)) => {
                        app.view = View::Table;
                        select(&mut app, i, &mut detail, cols);
                        draw_all(&app, &mut detail, &mut status, cols, rows);
                    }
                    Some(None) => status.say(&style::rgb("no match", Some(ERR_RGB), None, "")),
                    None => status.say(&help_line(&app)),
                }
            }
            "w" => {
                if let Some(url) = app.cache.sources.get(PARTICLES[app.sel].name) {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(url)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                }
            }
            "c" => {
                let prompt = if app.chat.is_empty() {
                    format!("Ask Claude about the {}: ", PARTICLES[app.sel].name)
                } else {
                    "Follow-up: ".to_string()
                };
                let q = status.ask_or_cancel(&prompt, "");
                print!("{}", Cursor::hide_seq());
                std::io::stdout().flush().ok();
                match q {
                    Some(q) if !q.trim().is_empty() => {
                        status.say(&style::rgb(" asking claude…", Some(ASK_RGB), None, ""));
                        match ask_claude(&app, q.trim()) {
                            Ok(a) if !a.is_empty() => {
                                app.chat.push((q.trim().to_string(), a));
                                app.pane = Pane2::Chat;
                                set_detail(&app, &mut detail, cols);
                                status.say(&help_line(&app));
                            }
                            Ok(_) => status.say(&style::rgb("claude returned nothing", Some(ERR_RGB), None, "")),
                            Err(e) => status.say(&style::rgb(&format!("claude: {e}"), Some(ERR_RGB), None, "")),
                        }
                    }
                    _ => status.say(&help_line(&app)),
                }
            }
            "C" => {
                app.pane = if app.pane == Pane2::Chat { Pane2::Article } else { Pane2::Chat };
                set_detail(&app, &mut detail, cols);
            }
            "?" => {
                app.pane = if app.pane == Pane2::Help { Pane2::Article } else { Pane2::Help };
                set_detail(&app, &mut detail, cols);
            }
            "u" => {
                Crust::cleanup();
                println!("particles: re-fetching articles …");
                let msg = match fetch::fetch_all() {
                    Ok(c) => {
                        let _ = data::save(&c);
                        app.cache = c;
                        "articles updated".to_string()
                    }
                    Err(e) => format!("fetch failed: {e}"),
                };
                Crust::init();
                Crust::set_app_identity("Particles");
                draw_all(&app, &mut detail, &mut status, cols, rows);
                status.say(&msg);
            }
            "RESIZE" => {
                let (c, r) = Crust::terminal_size();
                cols = c;
                rows = r;
                status.y = rows;
                status.w = cols;
                draw_all(&app, &mut detail, &mut status, cols, rows);
            }
            _ => {}
        }
    }

    Crust::cleanup();
}

// ─────────────────────────── selection ───────────────────────────────

fn step(app: &mut App, dir: (i32, i32), detail: &mut Pane, cols: u16) {
    let (cx, cy) = PARTICLES[app.sel].pos;
    let (mut x, mut y) = (cx as i32, cy as i32);
    for _ in 0..8 {
        x += dir.0;
        y += dir.1;
        if x < 0 || y < 0 || x > 5 || y > 3 {
            return;
        }
        if let Some(i) = PARTICLES.iter().position(|p| p.pos == (x as u16, y as u16)) {
            select(app, i, detail, cols);
            return;
        }
    }
}

fn select(app: &mut App, new: usize, detail: &mut Pane, cols: u16) {
    if new == app.sel && app.pane == Pane2::Article {
        return;
    }
    if new != app.sel {
        app.chat.clear();
    }
    app.sel = new;
    app.pane = Pane2::Article;
    draw_header(app, cols);
    draw_table(app, cols);
    draw_side(app, cols);
    set_detail(app, detail, cols);
}

// ───────────────────────────── colors ────────────────────────────────

fn kind_rgb(k: Kind) -> (u8, u8, u8) {
    match k {
        Kind::Quark => (255, 140, 90),
        Kind::Lepton => (120, 200, 255),
        Kind::Gauge => (255, 200, 90),
        Kind::Scalar => (200, 140, 255),
        Kind::Composite => (150, 220, 150),
    }
}

// ─────────────────────────── rendering ───────────────────────────────

fn move_to(row: u16, col: u16) -> String {
    Cursor::at(col, row)
}

fn draw_header(app: &App, cols: u16) {
    let p = &PARTICLES[app.sel];
    let (r, g, b) = kind_rgb(p.kind);
    let view = match app.view {
        View::Table => "table",
        View::Zoom => "zoom",
    };
    let info = format!(
        " {}  {}  {}  {}",
        style::rgb("particles", Some(RUST_RGB), None, "b"),
        style::bold(p.name),
        style::rgb(p.kind.label(), Some((r, g, b)), None, ""),
        style::dim(&format!("[{view}]  Tab switches"))
    );
    let pad = (cols as usize).saturating_sub(crust::display_width(&info));
    let armed = style::rgb("", None, Some(BAR_BG), "");
    let armed = armed.trim_end_matches(style::RESET);
    let line = info.replace(style::RESET, &format!("{}{}", style::RESET, armed));
    print!(
        "{}{}",
        move_to(1, 1),
        style::rgb(&format!("{line}{}", " ".repeat(pad)), None, Some(BAR_BG), "")
    );
    std::io::stdout().flush().ok();
}

/// The Standard Model chart: generations across, kinds down.
fn draw_table(app: &App, cols: u16) {
    if app.view != View::Table || cols < GRID_X + 3 * CELL_W {
        return;
    }
    let last = ((cols - GRID_X) / CELL_W).min(6);
    let mut s = String::new();
    // Column captions: three generations, then the bosons and what they bind.
    let caps = ["I", "II", "III", "bosons", "Higgs", "made of"];
    for (i, cap) in caps.iter().take(last as usize).enumerate() {
        s.push_str(&move_to(GRID_Y - 1, GRID_X + i as u16 * CELL_W));
        s.push_str(&style::dim(&format!("{cap:<11}")));
    }
    // Row captions down the left edge.
    for (i, cap) in ["quarks", "", "leptons", ""].iter().enumerate() {
        if cap.is_empty() {
            continue;
        }
        s.push_str(&move_to(GRID_Y + i as u16 * CELL_H, 1));
        s.push_str(&style::rgb(cap, Some((110, 110, 120)), None, ""));
    }
    for (i, p) in PARTICLES.iter().enumerate() {
        let (cx, cy) = p.pos;
        if cx >= last {
            continue;
        }
        let x = GRID_X + cx * CELL_W;
        let y = GRID_Y + cy * CELL_H;
        let rgb = kind_rgb(p.kind);
        let sel = i == app.sel;
        let mark = if sel { "▸" } else { " " };
        let attrs = if sel { "b" } else { "" };
        s.push_str(&move_to(y, x));
        s.push_str(&style::rgb(&format!("{mark}{:<11}", p.symbol), Some(rgb), None, attrs));
        s.push_str(&move_to(y + 1, x));
        let name = format!(" {:<11}", trunc(p.name, 11));
        s.push_str(&if sel { style::rgb(&name, Some(rgb), None, "") } else { style::dim(&name) });
        s.push_str(&move_to(y + 2, x));
        s.push_str(&style::rgb(&format!(" {:<11}", short_mass(p.mass)), Some((100, 100, 110)), None, ""));
    }
    print!("{s}");
    std::io::stdout().flush().ok();
}

/// "2.16 MeV/c²" → "2.16M", "< 0.8 eV/c² (upper limit)" → "<0.8e".
fn short_mass(m: &str) -> String {
    if m == "0" {
        return "massless".into();
    }
    let up = m.starts_with('<');
    let t = m.trim_start_matches("< ");
    let mut it = t.split_whitespace();
    let num = it.next().unwrap_or("");
    let unit = it.next().unwrap_or("");
    let u = match unit.chars().next() {
        Some('G') => "GeV",
        Some('M') => "MeV",
        Some('k') => "keV",
        _ => "eV",
    };
    format!("{}{num} {u}", if up { "<" } else { "" })
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}

/// Rows the model gets in the zoom view.
fn zoom_h(rows: u16) -> u16 {
    rows.saturating_sub(GRID_Y + ZOOM_BOTTOM).max(6)
}

/// The zoom view: one scene, rotated by the arrow keys.
fn draw_zoom(app: &App, cols: u16, rows: u16) {
    if app.view != View::Zoom {
        return;
    }
    let level = LEVELS[app.level];
    let w = cols.saturating_sub(4).max(20) as usize;
    let h = zoom_h(rows) as usize;
    let mut cv = Canvas::new(w, h);
    // A braille sub-pixel is half a cell wide and a quarter tall, and a
    // cell is about twice as tall as it is wide — so sub-pixels are
    // square and the scale is simply the smaller half-dimension.
    let view = View3 {
        yaw: app.yaw,
        pitch: app.pitch,
        eye: 5.0,
        scale: ((w * 2).min(h * 4) as f64 / 2.0) * 0.92,
    };
    let (pts, lines) = models::scene(level);
    view.draw(&mut cv, &pts);
    // Flux tubes and field lines go on top: they are the point of the
    // picture, and a sea of dots would otherwise bury them.
    for (a, b, rgb) in &lines {
        view.draw_line(&mut cv, a, b, *rgb);
    }

    let mut s = String::new();
    for y in 0..h {
        s.push_str(&move_to(GRID_Y + y as u16, 3));
        let mut line = String::new();
        for (ch, rgb) in cv.row(y) {
            match rgb {
                Some(c) if ch != ' ' => line.push_str(&style::rgb(&ch.to_string(), Some(c), None, "")),
                _ => line.push(ch),
            }
        }
        s.push_str(&line);
        s.push_str(&" ".repeat(cols.saturating_sub(3 + w as u16) as usize));
    }
    // Caption: what this is, and how big.
    s.push_str(&move_to(GRID_Y + h as u16, 3));
    s.push_str(&style::rgb(level.title(), Some(HEAD_RGB), None, "b"));
    s.push_str("   ");
    s.push_str(&style::dim(level.scale()));
    s.push_str(&move_to(GRID_Y + h as u16 + 1, 3));
    s.push_str(&style::dim(&format!(
        "level {}/{} · +/- to descend and climb · arrows rotate",
        app.level + 1,
        LEVELS.len()
    )));
    print!("{s}");
    std::io::stdout().flush().ok();
}

fn help_line(app: &App) -> String {
    match app.view {
        View::Table => style::dim(
            "←↓↑→ move · Tab zoom view · / find · c claude · w wiki · ? help · q quit",
        ),
        View::Zoom => style::dim(
            "arrows rotate · +/- zoom level · Tab table view · c claude · ? help · q quit",
        ),
    }
}

/// The article pane starts below whichever view is showing.
fn fit_panes(app: &App, detail: &mut Pane, cols: u16, rows: u16) {
    let top = match app.view {
        View::Table => DETAIL_Y,
        View::Zoom => GRID_Y + zoom_h(rows) + 3,
    };
    detail.y = top;
    detail.w = cols;
    detail.h = rows.saturating_sub(top).max(1);
}

fn draw_all(app: &App, detail: &mut Pane, status: &mut Pane, cols: u16, rows: u16) {
    Crust::clear_screen();
    fit_panes(app, detail, cols, rows);
    draw_header(app, cols);
    match app.view {
        View::Table => {
            draw_table(app, cols);
            draw_side(app, cols);
        }
        View::Zoom => draw_zoom(app, cols, rows),
    }
    status.invalidate();
    status.say(&help_line(app));
    detail.invalidate();
    set_detail(app, detail, cols);
}

fn prop_rows(p: &Particle) -> Vec<(String, String)> {
    let mut v = Vec::new();
    if p.generation > 0 {
        v.push(("generation".into(), format!("{}", p.generation)));
    }
    v.push(("mass".into(), p.mass.to_string()));
    v.push(("charge".into(), format!("{} e", p.charge)));
    v.push(("spin".into(), p.spin.to_string()));
    v.push((
        "statistics".into(),
        if p.kind.is_fermion() { "fermion".into() } else { "boson".to_string() },
    ));
    v.push(("color charge".into(), p.color_charge.to_string()));
    v.push(("feels".into(), p.forces.list()));
    v.push(("antiparticle".into(), p.antiparticle.to_string()));
    v.push(("discovered".into(), p.discovered.to_string()));
    v
}

const LBL_W: usize = 13;
const GUTTER: usize = 2;

fn fit(v: &str, w: usize) -> String {
    let n = v.chars().count();
    if n <= w {
        format!("{v}{}", " ".repeat(w - n))
    } else {
        let mut t: String = v.chars().take(w.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

fn draw_side(app: &App, cols: u16) {
    if cols < SIDE_MIN || app.view != View::Table {
        return;
    }
    let avail = (cols - SIDE_X + 1) as usize;
    let p = &PARTICLES[app.sel];
    let cell = avail.saturating_sub(2).min(56);
    let mut lines = vec![
        format!(
            "{}  {}",
            style::rgb(p.name, Some(kind_rgb(p.kind)), None, "b"),
            style::dim(p.symbol)
        ),
        String::new(),
    ];
    for chunk in wrap(p.blurb, avail.saturating_sub(1)) {
        lines.push(style::rgb(&chunk, Some((170, 170, 180)), None, ""));
    }
    lines.push(String::new());
    let bar = style::dim("│");
    let rule = "─".repeat(cell);
    lines.push(style::dim(&format!("┌{rule}┐")));
    for (k, v) in prop_rows(p) {
        lines.push(format!(
            "{bar} {}{}{} {bar}",
            style::dim(&format!("{k:<LBL_W$}")),
            " ".repeat(GUTTER),
            fit(&v, cell.saturating_sub(LBL_W + GUTTER + 2))
        ));
    }
    lines.push(style::dim(&format!("└{rule}┘")));

    let blank = " ".repeat(avail);
    let mut s = String::new();
    let rows = DETAIL_Y - GRID_Y + 1;
    for r in 0..rows {
        s.push_str(&move_to(GRID_Y - 1 + r, SIDE_X));
        s.push_str(&blank);
    }
    for (i, l) in lines.iter().take(rows as usize).enumerate() {
        s.push_str(&move_to(GRID_Y - 1 + i as u16, SIDE_X));
        s.push_str(&crust::truncate_ansi(l, avail));
    }
    print!("{s}");
    std::io::stdout().flush().ok();
}

fn wrap(s: &str, w: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let need = if line.is_empty() { word.chars().count() } else { line.chars().count() + 1 + word.chars().count() };
        if need > w && !line.is_empty() {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn set_detail(app: &App, detail: &mut Pane, cols: u16) {
    let side = cols >= SIDE_MIN && app.view == View::Table;
    let text = match app.pane {
        Pane2::Help => help_text(),
        Pane2::Chat => chat_text(app),
        Pane2::Article => match app.view {
            View::Zoom => zoom_text(app),
            View::Table => detail_text(&PARTICLES[app.sel], &app.cache, side),
        },
    };
    detail.set_text(&text);
    detail.ix = 0;
    detail.refresh();
}

/// What the current zoom level is showing, in words.
fn zoom_text(app: &App) -> String {
    let level = LEVELS[app.level];
    let mut s = format!("\n{}\n", style::rgb(level.title(), Some(HEAD_RGB), None, "b"));
    s.push_str(&format!("{}\n\n", style::dim(level.scale())));
    for l in wrap(level.note(), 96) {
        s.push_str(&l);
        s.push('\n');
    }
    s.push('\n');
    let ladder: Vec<String> = LEVELS
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let name = l.title().split(" — ").next().unwrap_or("");
            if i == app.level {
                style::rgb(name, Some(HEAD_RGB), None, "b")
            } else {
                style::dim(name)
            }
        })
        .collect();
    s.push_str(&ladder.join(&style::dim("  →  ")));
    s.push('\n');
    s
}

fn detail_text(p: &Particle, cache: &data::Cache, side: bool) -> String {
    let mut out = String::from("\n");
    if !side {
        out.push_str(&format!(
            "{}  {}\n\n",
            style::rgb(p.name, Some(kind_rgb(p.kind)), None, "b"),
            style::dim(p.kind.label())
        ));
        for (k, v) in prop_rows(p) {
            out.push_str(&format!("{}{}\n", style::dim(&format!("{k:<14}")), v));
        }
        out.push('\n');
    }
    match cache.articles.get(p.name) {
        Some(a) => {
            out.push_str(&format!("{}\n", style::rgb("Wikipedia article", Some(HEAD_RGB), None, "b")));
            out.push_str(&style_article(a));
        }
        None => out.push_str(&format!("{}\n", style::dim("No article cached (run with --fetch)."))),
    }
    out
}

const TAIL_SECTIONS: [&str; 9] = [
    "see also", "references", "notes", "citations", "sources",
    "further reading", "external links", "bibliography", "explanatory notes",
];

fn style_article(a: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in a.lines() {
        let t = line.trim();
        if t.len() > 4 && t.starts_with("==") && t.ends_with("==") {
            let level = t.chars().take_while(|c| *c == '=').count();
            let title = t.trim_matches(|c: char| c == '=' || c == ' ');
            if TAIL_SECTIONS.contains(&title.to_lowercase().as_str()) {
                break;
            }
            out.push(match level {
                2 => style::rgb(title, Some(HEAD_RGB), None, "b"),
                3 => format!("  {}", style::rgb(title, Some((250, 200, 130)), None, "b")),
                _ => format!("    {}", style::rgb(title, Some((200, 170, 140)), None, "b")),
            });
        } else if let Some(pos) = line.find("{\\displaystyle").or_else(|| line.find("{\\textstyle")) {
            while matches!(out.last(), Some(l) if l.is_empty() || l.starts_with(' ')) {
                out.pop();
            }
            let rest = &line[pos..];
            let inner = rest.find(' ').map(|i| rest[i + 1..].trim_end()).unwrap_or("");
            let inner = inner.strip_suffix('}').unwrap_or(inner).trim();
            if !inner.is_empty() {
                out.push(format!("    {}", style::rgb(inner, Some((150, 200, 255)), None, "")));
            }
        } else {
            if !line.trim().is_empty()
                && matches!(out.last(), Some(l) if !l.trim().is_empty() && !l.contains('\u{1b}'))
            {
                out.push(String::new());
            }
            out.push(line.to_string());
        }
    }
    let mut s = String::with_capacity(a.len() + 2048);
    let mut blank = false;
    for l in out {
        let empty = l.trim().is_empty();
        if empty && blank {
            continue;
        }
        blank = empty;
        s.push_str(&l);
        s.push('\n');
    }
    s
}

fn help_text() -> String {
    format!(
        "{}\n\n\
         \x20 Tab                 switch between the TABLE and the ZOOM view\n\n\
         \x20 in the table:\n\
         \x20 ← ↑ ↓ → / h j k l   move around the chart\n\
         \x20 < > or p n          previous / next particle\n\n\
         \x20 in the zoom:\n\
         \x20 ← ↑ ↓ → / h j k l   rotate the model\n\
         \x20 + -                 descend / climb: atom → nucleus → nucleon → quark\n\n\
         \x20 J K / Shift-↓ ↑     scroll the article\n\
         \x20 Space, PgDn/PgUp    page the article\n\
         \x20 g G                 top / bottom\n\
         \x20 /                   find a particle\n\
         \x20 c                   ask Claude about this particle\n\
         \x20 C                   toggle the Claude conversation\n\
         \x20 w                   open the Wikipedia page in a browser\n\
         \x20 u                   re-fetch the articles\n\
         \x20 ?                   this help\n\
         \x20 ESC                 back to the article (quits from the article view)\n\
         \x20 q                   quit\n\n\
         Masses and charges are the PDG values; the neutrino entries are experimental\n\
         upper limits, not measurements. The zoom models are schematic: they get the\n\
         counts, the charges and the ordering right, but an atom's nucleus drawn to\n\
         scale would be invisible, and quarks have no size to draw at all.",
        style::rgb("particles — keys", Some(RUST_RGB), None, "b")
    )
}

// ─────────────────────────── claude chat ─────────────────────────────

fn claude_run(prompt: &str, input: &str) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let mut child = Command::new("claude")
        .args(["-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => "claude not on PATH".to_string(),
            _ => format!("spawn: {e}"),
        })?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("stdin: {e}"))?;
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().map_err(|e| format!("wait: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(err.lines().next().unwrap_or("(no message)").chars().take(80).collect());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn ask_claude(app: &App, question: &str) -> Result<String, String> {
    let p = &PARTICLES[app.sel];
    let mut ctx = format!(
        "Particle: {} ({}), a {}. Mass {}, charge {} e, spin {}, color charge {}.\n\
         Feels: {}. Discovered: {}. Antiparticle: {}.\n",
        p.name, p.symbol, p.kind.label(), p.mass, p.charge, p.spin, p.color_charge,
        p.forces.list(), p.discovered, p.antiparticle
    );
    if app.view == View::Zoom {
        let l = LEVELS[app.level];
        ctx.push_str(&format!(
            "\nThe user is looking at the zoom view, level: {} ({}).\n",
            l.title(),
            l.scale()
        ));
    }
    if let Some(a) = app.cache.articles.get(p.name) {
        ctx.push_str("\nWikipedia article (may be truncated):\n");
        let art: String = a.chars().take(12000).collect();
        ctx.push_str(&art);
    }
    if !app.chat.is_empty() {
        ctx.push_str("\n\nEarlier in this conversation:\n");
        for (q, a) in &app.chat {
            ctx.push_str(&format!("User: {q}\nYou: {a}\n\n"));
        }
    }
    ctx.push_str(&format!("\n\nUser's question: {question}\n"));
    let prompt = format!(
        "You are a particle physics tutor answering inside a terminal Standard Model \
         app. The user is looking at the {}. Answer from the reference material and your \
         own knowledge. Plain text only, no markdown headings. Keep it tight: a few short \
         paragraphs at most. Do not use any tools; just answer.",
        p.name
    );
    claude_run(&prompt, &ctx)
}

fn chat_text(app: &App) -> String {
    let p = &PARTICLES[app.sel];
    let mut out = format!(
        "\n{}\n\n",
        style::rgb(&format!("Claude — {}", p.name), Some(HEAD_RGB), None, "b")
    );
    if app.chat.is_empty() {
        out.push_str(&format!("{}\n", style::dim("Press c to ask a question about this particle.")));
        return out;
    }
    for (q, a) in &app.chat {
        out.push_str(&format!("{}\n\n{a}\n\n", style::rgb(&format!("? {q}"), Some(ASK_RGB), None, "b")));
    }
    out.push_str(&format!("{}\n", style::dim("c: ask a follow-up · ESC: back")));
    out
}
