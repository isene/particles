//! The zoom ladder: four scenes, each a factor of ten thousand or so
//! smaller than the one before. Carbon-12 is the subject throughout, so
//! the counts stay honest as you descend: 6 protons and 6 neutrons in
//! the nucleus, three valence quarks in each nucleon.

use crate::canvas::P3;

pub const PROTON_RGB: (u8, u8, u8) = (255, 110, 90);
pub const NEUTRON_RGB: (u8, u8, u8) = (120, 170, 255);
pub const ELECTRON_RGB: (u8, u8, u8) = (120, 220, 255);
pub const GLUON_RGB: (u8, u8, u8) = (255, 200, 90);
pub const QUARK_R: (u8, u8, u8) = (255, 90, 90);
pub const QUARK_G: (u8, u8, u8) = (110, 230, 130);
pub const QUARK_B: (u8, u8, u8) = (110, 160, 255);
pub const FIELD_RGB: (u8, u8, u8) = (110, 110, 125);
pub const ANTIQUARK_RGB: (u8, u8, u8) = (230, 120, 255);

#[derive(Clone, Copy, PartialEq)]
pub enum Level {
    Atom,
    Nucleus,
    Nucleon,
    Quark,
}

pub const LEVELS: [Level; 4] = [Level::Atom, Level::Nucleus, Level::Nucleon, Level::Quark];

impl Level {
    pub fn title(&self) -> &'static str {
        match self {
            Level::Atom => "atom — carbon-12",
            Level::Nucleus => "nucleus — 6 protons, 6 neutrons",
            Level::Nucleon => "nucleon — proton (uud)",
            Level::Quark => "quark — a point, and the string that traps it",
        }
    }
    /// Physical size of what fills the frame.
    pub fn scale(&self) -> &'static str {
        match self {
            Level::Atom => "≈ 1.4 × 10⁻¹⁰ m across",
            Level::Nucleus => "≈ 5 × 10⁻¹⁵ m across",
            Level::Nucleon => "≈ 1.7 × 10⁻¹⁵ m across",
            Level::Quark => "< 10⁻¹⁸ m — a point, as far as anyone can measure",
        }
    }
    /// What the picture is trying to teach.
    pub fn note(&self) -> &'static str {
        match self {
            Level::Atom =>
                "Almost all of it is empty. The nucleus at the centre is drawn far too \
                 large: at this scale it would be a speck a hundred thousand times smaller \
                 than the electron cloud. Electrons are not little planets on tracks — the \
                 shells are where they are LIKELY to be found.",
            Level::Nucleus =>
                "Twelve nucleons packed by the strong force, which at this range beats the \
                 electrical repulsion trying to blow the six protons apart. The residual \
                 strong force between nucleons is a leftover of the far stronger force \
                 binding quarks inside each one.",
            Level::Nucleon =>
                "Three valence quarks (two up, one down) exchanging gluons. The quark masses \
                 add up to about 1% of the proton's mass; the rest is the energy of the \
                 gluon field itself. Mass, here, is mostly binding energy.",
            Level::Quark =>
                "Pull two quarks apart and the gluon string between them stores energy until \
                 it snaps into a new quark-antiquark pair. You never get a free quark — that \
                 is confinement. No experiment has resolved any structure inside one.",
        }
    }
}

/// Deterministic pseudo-random in 0..1 — no RNG dependency, and the same
/// arrangement every run so the picture is stable while you rotate it.
fn rnd(seed: u64, i: u64) -> f64 {
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(i.wrapping_mul(1442695040888963407));
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    (x % 100_000) as f64 / 100_000.0
}

/// Points on a sphere of radius `r`, spread by the golden angle so they
/// look evenly scattered rather than clustered at the poles.
fn shell(n: usize, r: f64, rgb: (u8, u8, u8), dot: i32, seed: u64) -> Vec<P3> {
    let ga = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let rad = (1.0 - y * y).max(0.0).sqrt();
            let th = ga * i as f64 + rnd(seed, i as u64) * 0.3;
            P3::new(rad * th.cos() * r, y * r, rad * th.sin() * r, rgb, dot)
        })
        .collect()
}

/// The atom: a dense nucleus and two electron clouds. Carbon's six
/// electrons sit 2 in the inner shell and 4 in the outer one — drawn as
/// probability clouds, not tracks, because that is what they are.
pub fn atom() -> (Vec<P3>, Vec<(P3, P3, (u8, u8, u8))>) {
    let mut pts = Vec::new();
    // Nucleus, deliberately oversized so it is visible at all.
    pts.extend(shell(6, 0.13, PROTON_RGB, 1, 7));
    pts.extend(shell(6, 0.09, NEUTRON_RGB, 1, 11));
    for (n, cloud, r, seed) in [(2usize, 240usize, 0.46f64, 21u64), (4, 520, 0.98, 33)] {
        // The cloud: where the electrons are likely to be, thickened by
        // a radial jitter so it reads as a shell rather than a wire ball.
        for (i, p) in shell(cloud, r, (55, 70, 95), 0, seed).into_iter().enumerate() {
            let j = 1.0 + (rnd(seed + 1, i as u64) - 0.5) * 0.30;
            pts.push(P3::new(p.x * j, p.y * j, p.z * j, (55, 70, 95), 0));
        }
        // The electrons themselves, somewhere in that cloud.
        pts.extend(shell(n, r, ELECTRON_RGB, 2, seed + 2));
    }
    (pts, Vec::new())
}

/// The nucleus: twelve nucleons packed into a rough sphere.
pub fn nucleus() -> (Vec<P3>, Vec<(P3, P3, (u8, u8, u8))>) {
    let mut pts = Vec::new();
    let places = shell(12, 0.62, PROTON_RGB, 0, 3);
    for (i, p) in places.iter().enumerate() {
        let rgb = if i % 2 == 0 { PROTON_RGB } else { NEUTRON_RGB };
        // Each nucleon as a small ball of its own.
        for q in shell(26, 0.26, rgb, 0, 100 + i as u64) {
            pts.push(P3::new(p.x + q.x, p.y + q.y, p.z + q.z, rgb, 0));
        }
        pts.push(P3::new(p.x, p.y, p.z, rgb, 2));
    }
    (pts, Vec::new())
}

/// A proton: three valence quarks with gluon strings between them, in a
/// haze of sea quarks and gluons.
pub fn nucleon() -> (Vec<P3>, Vec<(P3, P3, (u8, u8, u8))>) {
    let mut pts = Vec::new();
    let quarks = [
        P3::new(0.0, 0.55, 0.0, QUARK_R, 4),
        P3::new(-0.5, -0.32, 0.18, QUARK_G, 4),
        P3::new(0.5, -0.32, -0.18, QUARK_B, 4),
    ];
    // The sea: virtual pairs and gluons filling the bag, kept dim so the
    // three valence quarks and the flux tubes stay readable.
    for i in 0..200 {
        let r = 0.95 * rnd(5, i).cbrt();
        let th = rnd(6, i) * std::f64::consts::TAU;
        let ph = (2.0 * rnd(7, i) - 1.0).acos();
        pts.push(P3::new(
            r * ph.sin() * th.cos(),
            r * ph.sin() * th.sin(),
            r * ph.cos(),
            if i % 3 == 0 { (130, 100, 45) } else { (60, 55, 65) },
            0,
        ));
    }
    pts.extend(quarks.iter().copied());
    // Gluon strings: the flux tubes that make confinement.
    let lines = vec![
        (quarks[0], quarks[1], GLUON_RGB),
        (quarks[1], quarks[2], GLUON_RGB),
        (quarks[2], quarks[0], GLUON_RGB),
    ];
    (pts, lines)
}

/// One quark, and what happens when you try to pull it out: the flux
/// tube between it and its partner stretches, stores energy, and snaps
/// into a fresh quark-antiquark pair. Two mesons, never a free quark.
pub fn quark() -> (Vec<P3>, Vec<(P3, P3, (u8, u8, u8))>) {
    let mut pts = Vec::new();
    // The two quarks being pulled apart.
    pts.push(P3::new(-1.30, 0.0, 0.0, QUARK_R, 4));
    pts.push(P3::new(1.30, 0.0, 0.0, QUARK_B, 4));
    // The pair created out of the string's own energy, at the snap.
    pts.push(P3::new(-0.14, 0.0, 0.0, ANTIQUARK_RGB, 3));
    pts.push(P3::new(0.14, 0.0, 0.0, QUARK_G, 3));
    // The flux tube: a narrow bundle of gluon field, not a thin line.
    for (a, b) in [(-1.30, -0.14), (0.14, 1.30)] {
        for i in 0..150u64 {
            let t = i as f64 / 150.0;
            let x = a + (b - a) * t;
            let ang = rnd(31, i) * std::f64::consts::TAU;
            let rad = 0.07 * rnd(32, i).sqrt();
            pts.push(P3::new(x, rad * ang.cos(), rad * ang.sin(), GLUON_RGB, 0));
        }
    }
    // The gap where it broke, marked by the field collapsing inward.
    let mut lines = Vec::new();
    for i in 0..8u64 {
        let th = i as f64 / 8.0 * std::f64::consts::TAU;
        lines.push((
            P3::new(-0.14, 0.20 * th.cos(), 0.20 * th.sin(), FIELD_RGB, 0),
            P3::new(0.14, 0.20 * th.cos(), 0.20 * th.sin(), FIELD_RGB, 0),
            FIELD_RGB,
        ));
    }
    (pts, lines)
}

pub fn scene(level: Level) -> (Vec<P3>, Vec<(P3, P3, (u8, u8, u8))>) {
    match level {
        Level::Atom => atom(),
        Level::Nucleus => nucleus(),
        Level::Nucleon => nucleon(),
        Level::Quark => quark(),
    }
}
