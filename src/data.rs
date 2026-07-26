//! The Standard Model table, plus the composite particles the zoom view
//! walks through. Values are the PDG ones (cross-checked against
//! Wikidata); every article is fetched once and cached at
//! ~/.particles/particles.json.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Quark,
    Lepton,
    /// Force carrier (spin 1).
    Gauge,
    /// The Higgs (spin 0).
    Scalar,
    /// Made of quarks: proton, neutron.
    Composite,
}

impl Kind {
    pub fn label(&self) -> &'static str {
        match self {
            Kind::Quark => "quark",
            Kind::Lepton => "lepton",
            Kind::Gauge => "gauge boson",
            Kind::Scalar => "scalar boson",
            Kind::Composite => "composite",
        }
    }
    /// Fermions carry half-integer spin and obey the exclusion principle;
    /// bosons carry integer spin and do not.
    pub fn is_fermion(&self) -> bool {
        matches!(self, Kind::Quark | Kind::Lepton)
    }
}

/// Which of the four interactions a particle takes part in.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Forces {
    pub strong: bool,
    pub em: bool,
    pub weak: bool,
    /// Everything with energy gravitates; kept explicit so the table is
    /// honest rather than silent about it.
    pub gravity: bool,
}

impl Forces {
    pub const fn new(strong: bool, em: bool, weak: bool) -> Self {
        Self { strong, em, weak, gravity: true }
    }
    pub fn list(&self) -> String {
        let mut v = Vec::new();
        if self.strong { v.push("strong"); }
        if self.em { v.push("electromagnetic"); }
        if self.weak { v.push("weak"); }
        if self.gravity { v.push("gravity"); }
        v.join(", ")
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Particle {
    pub symbol: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    /// 1, 2, 3 for the fermion generations; 0 for bosons and composites.
    pub generation: u8,
    /// Mass as text: the units differ by 12 orders of magnitude across
    /// the table, and the neutrinos have only upper limits.
    pub mass: &'static str,
    /// In units of the elementary charge.
    pub charge: &'static str,
    pub spin: &'static str,
    /// Color charge: quarks and gluons carry it, nothing else does.
    pub color_charge: &'static str,
    pub forces: Forces,
    pub discovered: &'static str,
    pub antiparticle: &'static str,
    /// One line on what the particle is for, in plain words.
    pub blurb: &'static str,
    pub wiki: &'static str,
    /// Grid position in the Standard Model chart: (column, row).
    pub pos: (u16, u16),
}

/// The seventeen fundamental particles, plus the two composites the
/// zoom view needs. Masses are PDG values; neutrino entries are the
/// experimental upper limits, not measurements.
pub const PARTICLES: &[Particle] = &[
    // ── quarks: generation columns 0..2, rows 0..1 ───────────────────
    Particle {
        symbol: "u", name: "up quark", kind: Kind::Quark, generation: 1,
        mass: "2.16 MeV/c²", charge: "+2/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1968 (SLAC, deep inelastic scattering)",
        antiparticle: "anti-up (ū)",
        blurb: "Lightest quark. Two of them plus a down quark make a proton.",
        wiki: "Up quark", pos: (0, 0),
    },
    Particle {
        symbol: "d", name: "down quark", kind: Kind::Quark, generation: 1,
        mass: "4.70 MeV/c²", charge: "-1/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1968 (SLAC, deep inelastic scattering)",
        antiparticle: "anti-down (d̄)",
        blurb: "Partner of the up quark. Two downs and an up make a neutron.",
        wiki: "Down quark", pos: (0, 1),
    },
    Particle {
        symbol: "c", name: "charm quark", kind: Kind::Quark, generation: 2,
        mass: "1.273 GeV/c²", charge: "+2/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1974 (the November Revolution, J/ψ)",
        antiparticle: "anti-charm (c̄)",
        blurb: "Heavier copy of the up quark; its discovery confirmed the quark model.",
        wiki: "Charm quark", pos: (1, 0),
    },
    Particle {
        symbol: "s", name: "strange quark", kind: Kind::Quark, generation: 2,
        mass: "93.5 MeV/c²", charge: "-1/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1947 (kaons in cosmic rays)",
        antiparticle: "anti-strange (s̄)",
        blurb: "Named for the strangely long lifetime of the particles containing it.",
        wiki: "Strange quark", pos: (1, 1),
    },
    Particle {
        symbol: "t", name: "top quark", kind: Kind::Quark, generation: 3,
        mass: "172.57 GeV/c²", charge: "+2/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1995 (Fermilab, CDF and DØ)",
        antiparticle: "anti-top (t̄)",
        blurb: "Heaviest known particle, as massive as a gold atom. Decays before it can bind.",
        wiki: "Top quark", pos: (2, 0),
    },
    Particle {
        symbol: "b", name: "bottom quark", kind: Kind::Quark, generation: 3,
        mass: "4.183 GeV/c²", charge: "-1/3", spin: "1/2", color_charge: "r / g / b",
        forces: Forces::new(true, true, true), discovered: "1977 (Fermilab, upsilon)",
        antiparticle: "anti-bottom (b̄)",
        blurb: "Long-lived enough to leave a measurable track; the workhorse of flavour physics.",
        wiki: "Bottom quark", pos: (2, 1),
    },
    // ── leptons: rows 2..3 ──────────────────────────────────────────
    Particle {
        symbol: "e", name: "electron", kind: Kind::Lepton, generation: 1,
        mass: "0.511 MeV/c²", charge: "-1", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, true, true), discovered: "1897 (J. J. Thomson)",
        antiparticle: "positron (e⁺)",
        blurb: "Carries electricity, holds atoms together, does all of chemistry.",
        wiki: "Electron", pos: (0, 2),
    },
    Particle {
        symbol: "νe", name: "electron neutrino", kind: Kind::Lepton, generation: 1,
        mass: "< 0.8 eV/c² (upper limit)", charge: "0", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, false, true), discovered: "1956 (Cowan and Reines)",
        antiparticle: "electron antineutrino (ν̄e)",
        blurb: "Almost massless, almost never interacts. Trillions pass through you each second.",
        wiki: "Electron neutrino", pos: (0, 3),
    },
    Particle {
        symbol: "μ", name: "muon", kind: Kind::Lepton, generation: 2,
        mass: "105.66 MeV/c²", charge: "-1", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, true, true), discovered: "1936 (cosmic rays)",
        antiparticle: "antimuon (μ⁺)",
        blurb: "A heavy electron that lives 2.2 microseconds. \"Who ordered that?\"",
        wiki: "Muon", pos: (1, 2),
    },
    Particle {
        symbol: "νμ", name: "muon neutrino", kind: Kind::Lepton, generation: 2,
        mass: "< 0.19 MeV/c² (upper limit)", charge: "0", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, false, true), discovered: "1962 (Lederman, Schwartz, Steinberger)",
        antiparticle: "muon antineutrino (ν̄μ)",
        blurb: "Proved neutrinos come in distinct flavours.",
        wiki: "Muon neutrino", pos: (1, 3),
    },
    Particle {
        symbol: "τ", name: "tau", kind: Kind::Lepton, generation: 3,
        mass: "1776.86 MeV/c²", charge: "-1", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, true, true), discovered: "1975 (SLAC, Martin Perl)",
        antiparticle: "antitau (τ⁺)",
        blurb: "Heavy enough to decay into hadrons, unlike its lighter cousins.",
        wiki: "Tau (particle)", pos: (2, 2),
    },
    Particle {
        symbol: "ντ", name: "tau neutrino", kind: Kind::Lepton, generation: 3,
        mass: "< 18.2 MeV/c² (upper limit)", charge: "0", spin: "1/2", color_charge: "none",
        forces: Forces::new(false, false, true), discovered: "2000 (Fermilab, DONUT)",
        antiparticle: "tau antineutrino (ν̄τ)",
        blurb: "The last fermion of the Standard Model to be observed directly.",
        wiki: "Tau neutrino", pos: (2, 3),
    },
    // ── bosons: column 3 (gauge) and column 4 (scalar) ──────────────
    Particle {
        symbol: "γ", name: "photon", kind: Kind::Gauge, generation: 0,
        mass: "0", charge: "0", spin: "1", color_charge: "none",
        forces: Forces::new(false, false, false), discovered: "1905 (Einstein) / 1923 (Compton)",
        antiparticle: "its own",
        blurb: "Carries electromagnetism. Massless, so its reach is unlimited.",
        wiki: "Photon", pos: (3, 0),
    },
    Particle {
        symbol: "g", name: "gluon", kind: Kind::Gauge, generation: 0,
        mass: "0", charge: "0", spin: "1", color_charge: "8 combinations",
        forces: Forces::new(true, false, false), discovered: "1979 (DESY, three-jet events)",
        antiparticle: "its own (per color state)",
        blurb: "Carries the strong force. Carries color itself, so gluons pull on each other.",
        wiki: "Gluon", pos: (3, 1),
    },
    Particle {
        symbol: "W", name: "W boson", kind: Kind::Gauge, generation: 0,
        mass: "80.377 GeV/c²", charge: "±1", spin: "1", color_charge: "none",
        forces: Forces::new(false, true, true), discovered: "1983 (CERN, UA1 and UA2)",
        antiparticle: "W⁺ ↔ W⁻",
        blurb: "Carries the weak force and changes one flavour into another. Beta decay is its work.",
        wiki: "W boson", pos: (3, 2),
    },
    Particle {
        symbol: "Z", name: "Z boson", kind: Kind::Gauge, generation: 0,
        mass: "91.188 GeV/c²", charge: "0", spin: "1", color_charge: "none",
        forces: Forces::new(false, false, true), discovered: "1983 (CERN, UA1 and UA2)",
        antiparticle: "its own",
        blurb: "The neutral weak carrier. Heavy, so the weak force barely reaches past a nucleus.",
        wiki: "Z boson", pos: (3, 3),
    },
    Particle {
        symbol: "H", name: "Higgs boson", kind: Kind::Scalar, generation: 0,
        mass: "125.25 GeV/c²", charge: "0", spin: "0", color_charge: "none",
        forces: Forces::new(false, false, true), discovered: "2012 (CERN, ATLAS and CMS)",
        antiparticle: "its own",
        blurb: "A ripple in the field that gives the other particles their mass.",
        wiki: "Higgs boson", pos: (4, 0),
    },
    // ── composites, for the zoom view ───────────────────────────────
    Particle {
        symbol: "p", name: "proton", kind: Kind::Composite, generation: 0,
        mass: "938.272 MeV/c²", charge: "+1", spin: "1/2", color_charge: "neutral (white)",
        forces: Forces::new(true, true, true), discovered: "1919 (Rutherford)",
        antiparticle: "antiproton (p̄)",
        blurb: "Two up quarks and a down, bound by gluons. Its count is the atomic number.",
        wiki: "Proton", pos: (5, 0),
    },
    Particle {
        symbol: "n", name: "neutron", kind: Kind::Composite, generation: 0,
        mass: "939.565 MeV/c²", charge: "0", spin: "1/2", color_charge: "neutral (white)",
        forces: Forces::new(true, true, true), discovered: "1932 (Chadwick)",
        antiparticle: "antineutron (n̄)",
        blurb: "Two down quarks and an up. Free ones decay in about 15 minutes.",
        wiki: "Neutron", pos: (5, 1),
    },
];

/// Cached article text, keyed by particle name. The table itself is
/// compiled in; only the prose needs fetching.
#[derive(Serialize, Deserialize, Default)]
pub struct Cache {
    pub articles: std::collections::HashMap<String, String>,
    pub sources: std::collections::HashMap<String, String>,
}

pub fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".particles").join("particles.json")
}

pub fn load() -> Option<Cache> {
    let raw = std::fs::read_to_string(cache_path()).ok()?;
    let c: Cache = serde_json::from_str(&raw).ok()?;
    if c.articles.is_empty() { None } else { Some(c) }
}

pub fn save(c: &Cache) -> std::io::Result<()> {
    let path = cache_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string(c)?)?;
    std::fs::rename(tmp, path)
}

pub fn find(query: &str) -> Option<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }
    PARTICLES
        .iter()
        .position(|p| p.symbol.to_lowercase() == q)
        .or_else(|| PARTICLES.iter().position(|p| p.name.to_lowercase() == q))
        .or_else(|| PARTICLES.iter().position(|p| p.name.to_lowercase().starts_with(&q)))
        .or_else(|| PARTICLES.iter().position(|p| p.name.to_lowercase().contains(&q)))
}
