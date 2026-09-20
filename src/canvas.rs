//! A tiny 3D point renderer for the terminal.
//!
//! Braille cells carry a 2×4 dot matrix, so one character cell becomes
//! eight sub-pixels: an 80×20 pane is a 160×80 plotting grid. Points are
//! rotated, projected with a simple perspective divide, and painted into
//! that grid. Color is per CELL (a braille glyph is one character, so it
//! has one color), and the nearest point in a cell wins — which doubles
//! as a crude depth cue. Where the terminal shows images the same canvas
//! is real pixels, one colour and depth per pixel, painted through glow.

/// Braille dot bit for a sub-pixel position within a cell.
/// Rows 0-2 use bits 0,1,2 / 3,4,5; row 3 uses bits 6,7.
const DOTS: [[u8; 2]; 4] = [
    [0x01, 0x08],
    [0x02, 0x10],
    [0x04, 0x20],
    [0x40, 0x80],
];

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    /// Sub-pixels per cell across and down: 2 by 4 for braille, 1 by 1
    /// when every cell is a real pixel.
    sub: (usize, usize),
    /// Pixels per braille sub-pixel, what a radius given in sub-pixels
    /// is scaled by. 1 for braille.
    dot: f64,
    bits: Vec<u8>,
    color: Vec<Option<(u8, u8, u8)>>,
    /// Depth of whatever set each cell's color, for nearest-wins.
    depth: Vec<f32>,
}

impl Canvas {
    /// A braille canvas of `cols` × `rows` cells.
    pub fn new(cols: usize, rows: usize) -> Self {
        Self::make(cols, rows, (2, 4), 1.0)
    }

    /// A canvas of real pixels for `cols` × `rows` cells of `cell` pixels.
    pub fn pixels(cols: usize, rows: usize, cell: (u16, u16)) -> Self {
        let (cw, ch) = (cell.0.max(1) as usize, cell.1.max(1) as usize);
        Self::make(cols * cw, rows * ch, (1, 1), (cw as f64 / 2.0 + ch as f64 / 4.0) / 2.0)
    }

    fn make(w: usize, h: usize, sub: (usize, usize), dot: f64) -> Self {
        Self {
            w,
            h,
            sub,
            dot,
            bits: vec![0; w * h],
            color: vec![None; w * h],
            depth: vec![f32::INFINITY; w * h],
        }
    }

    /// The plotting grid, in sub-pixels.
    pub fn pw(&self) -> usize {
        self.w * self.sub.0
    }
    pub fn ph(&self) -> usize {
        self.h * self.sub.1
    }

    /// A radius given in braille sub-pixels, in this canvas's own units.
    /// A point of radius 0 stays one sub-pixel wide either way.
    fn radius(&self, r: i32) -> i32 {
        if self.sub == (1, 1) {
            ((r.max(0) as f64).max(0.5) * self.dot).round() as i32
        } else {
            r.max(0)
        }
    }

    /// Plot a sub-pixel. `x` in 0..pw, `y` in 0..ph.
    pub fn set(&mut self, x: i32, y: i32, rgb: (u8, u8, u8), z: f64) {
        if x < 0 || y < 0 {
            return;
        }
        let (px, py) = (x as usize, y as usize);
        let (cx, cy) = (px / self.sub.0, py / self.sub.1);
        if cx >= self.w || cy >= self.h {
            return;
        }
        let i = cy * self.w + cx;
        self.bits[i] |= if self.sub == (1, 1) { 1 } else { DOTS[py % 4][px % 2] };
        if (z as f32) < self.depth[i] {
            self.depth[i] = z as f32;
            self.color[i] = Some(rgb);
        }
    }

    /// Draw a filled disc, `r` in braille sub-pixels, for particles with size.
    pub fn disc(&mut self, x: i32, y: i32, r: i32, rgb: (u8, u8, u8), z: f64) {
        let r = self.radius(r);
        self.fill(x, y, r, rgb, z);
    }

    /// A filled disc of radius `r` in this canvas's own units.
    fn fill(&mut self, x: i32, y: i32, r: i32, rgb: (u8, u8, u8), z: f64) {
        if r == 0 {
            return self.set(x, y, rgb, z);
        }
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    self.set(x + dx, y + dy, rgb, z);
                }
            }
        }
    }

    /// Straight line in sub-pixel space (Bresenham), for gluon strings
    /// and orbit traces.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: (u8, u8, u8), z: f64) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        // As thick as a braille dot, whatever the canvas.
        let thick = (self.dot / 2.0).floor() as i32;
        loop {
            self.fill(x, y, thick, rgb, z);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Copy a pixel canvas onto glow's canvas, cell for pixel.
    pub fn paint(&self, out: &mut glow::Canvas) {
        for y in 0..self.h.min(out.h) {
            for x in 0..self.w.min(out.w) {
                let i = y * self.w + x;
                if self.bits[i] != 0 {
                    if let Some(c) = self.color[i] {
                        out.put(x, y, c);
                    }
                }
            }
        }
    }

    /// One rendered braille row, as (cell glyph, color) pairs.
    pub fn row(&self, y: usize) -> Vec<(char, Option<(u8, u8, u8)>)> {
        (0..self.w)
            .map(|x| {
                let i = y * self.w + x;
                let b = self.bits[i];
                let ch = if b == 0 { ' ' } else { char::from_u32(0x2800 + b as u32).unwrap_or(' ') };
                (ch, self.color[i])
            })
            .collect()
    }
}

/// A point in the scene: position, color, and how fat to draw it.
#[derive(Clone, Copy)]
pub struct P3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rgb: (u8, u8, u8),
    pub r: i32,
}

impl P3 {
    pub fn new(x: f64, y: f64, z: f64, rgb: (u8, u8, u8), r: i32) -> Self {
        Self { x, y, z, rgb, r }
    }
}

/// Rotate by yaw (around the vertical axis) then pitch (around the
/// horizontal), and project. Returns sub-pixel coordinates plus depth.
pub struct View3 {
    pub yaw: f64,
    pub pitch: f64,
    /// Distance of the eye from the origin, in scene units.
    pub eye: f64,
    /// Scene units to sub-pixels at the origin plane.
    pub scale: f64,
}

impl View3 {
    pub fn project(&self, p: &P3, canvas: &Canvas) -> Option<(i32, i32, f64)> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        // Yaw about Y, then pitch about X.
        let x1 = p.x * cy + p.z * sy;
        let z1 = -p.x * sy + p.z * cy;
        let y2 = p.y * cp - z1 * sp;
        let z2 = p.y * sp + z1 * cp;
        let denom = self.eye + z2;
        if denom <= 0.05 {
            return None; // behind the eye
        }
        let f = self.eye / denom;
        let sx = canvas.pw() as f64 / 2.0 + x1 * f * self.scale;
        let syp = canvas.ph() as f64 / 2.0 - y2 * f * self.scale;
        Some((sx.round() as i32, syp.round() as i32, z2))
    }

    /// Project and draw a whole cloud, painting far points first so the
    /// near ones win the color of any cell they share.
    pub fn draw(&self, canvas: &mut Canvas, points: &[P3]) {
        let mut order: Vec<&P3> = points.iter().collect();
        order.sort_by(|a, b| {
            let da = self.depth_of(a);
            let db = self.depth_of(b);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
        for p in order {
            if let Some((x, y, z)) = self.project(p, canvas) {
                canvas.disc(x, y, p.r, p.rgb, z);
            }
        }
    }

    fn depth_of(&self, p: &P3) -> f64 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let z1 = -p.x * sy + p.z * cy;
        p.y * sp + z1 * cp
    }

    /// Project a scene-space segment into canvas space and draw it.
    pub fn draw_line(&self, canvas: &mut Canvas, a: &P3, b: &P3, rgb: (u8, u8, u8)) {
        if let (Some((x0, y0, z0)), Some((x1, y1, z1))) =
            (self.project(a, canvas), self.project(b, canvas))
        {
            // Nudged forward: a line that matters should not lose its
            // cell to a background dot that happens to be a hair nearer.
            canvas.line(x0, y0, x1, y1, rgb, z0.min(z1) - 0.5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pixel_canvas_scales_dots_and_paints_them() {
        let mut cv = Canvas::pixels(4, 2, (10, 20));
        assert_eq!((cv.pw(), cv.ph()), (40, 40));
        cv.disc(20, 20, 0, (200, 100, 50), 1.0);
        let mut out = glow::Canvas::with_cell(4, 2, (10, 20));
        cv.paint(&mut out);
        let lit = out.rgba.chunks(4).filter(|p| p[0] == 200).count();
        assert!(lit > 1 && lit < 60, "a point is a small dot of {lit} pixels, not one");
        assert_eq!(&out.rgba[(20 * 40 + 20) * 4..][..3], &[200, 100, 50]);
        // Braille stays one sub-pixel per point of radius 0.
        let mut br = Canvas::new(4, 2);
        br.disc(3, 5, 0, (1, 1, 1), 1.0);
        assert_eq!(br.row(1)[1].0, '\u{2810}');
    }
}
