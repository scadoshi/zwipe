//! The palette gate for `assets/themes.css`, per `context/plans/palette.md`.
//!
//! Every theme block must carry `--palette-1` to `--palette-6` and `--color-money`. Each slot must
//! read on the block's `--bg-primary` at 3:1 or better. Pairs closer than 15 in
//! OKLab are printed, never failed: published colors stay as published. The four
//! accessibility themes are the exception: their six are built, not sourced, so
//! every pair must stay 8 apart under the deficiency they are built for.

// The close-pair report is the point of the test's output.
#![allow(clippy::print_stdout)]

const CSS: &str = include_str!("../assets/themes.css");

struct Block {
    name: String,
    vars: Vec<(String, String)>,
}

fn blocks() -> Vec<Block> {
    let mut out = Vec::new();
    let mut rest = CSS;
    while let Some(start) = rest.find(".theme-") {
        let after = &rest[start + ".theme-".len()..];
        let Some(brace) = after.find('{') else { break };
        let name = after[..brace].trim().to_string();
        let body_start = brace + 1;
        let Some(end) = after[body_start..].find('}') else {
            break;
        };
        let body = &after[body_start..body_start + end];
        let vars = body
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                let l = l.strip_prefix("--")?;
                let (k, v) = l.split_once(':')?;
                Some((
                    k.trim().to_string(),
                    v.trim().trim_end_matches(';').trim().to_string(),
                ))
            })
            .collect();
        out.push(Block { name, vars });
        rest = &after[body_start + end..];
    }
    out
}

impl Block {
    fn get(&self, key: &str) -> Option<&str> {
        self.vars
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

fn hex(s: &str) -> Option<[f64; 3]> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let ch = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
    Some([
        f64::from(ch(0)?) / 255.0,
        f64::from(ch(2)?) / 255.0,
        f64::from(ch(4)?) / 255.0,
    ])
}

fn lin(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab(rgb: [f64; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(lin);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
    ]
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let [x, y, z] = oklab(a);
    let [p, q, r] = oklab(b);
    100.0 * ((x - p).powi(2) + (y - q).powi(2) + (z - r).powi(2)).sqrt()
}

fn luminance(rgb: [f64; 3]) -> f64 {
    let [r, g, b] = rgb.map(lin);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// Machado, Oliveira and Fernandes 2009, severity 1.0, applied in linear RGB.
fn simulate(rgb: [f64; 3], kind: &str) -> [f64; 3] {
    let m: [[f64; 3]; 3] = match kind {
        "protanopia" => [
            [0.152_286, 1.052_583, -0.204_868],
            [0.114_503, 0.786_281, 0.099_216],
            [-0.003_882, -0.048_116, 1.051_998],
        ],
        "deuteranopia" => [
            [0.367_322, 0.860_646, -0.227_968],
            [0.280_085, 0.672_501, 0.047_413],
            [-0.011_820, 0.042_940, 0.968_881],
        ],
        "tritanopia" => [
            [1.255_528, -0.076_749, -0.178_779],
            [-0.078_411, 0.930_809, 0.147_602],
            [0.004_733, 0.691_367, 0.303_900],
        ],
        _ => return rgb,
    };
    let [r, g, b] = rgb.map(lin);
    let unlin = |c: f64| {
        let c = c.clamp(0.0, 1.0);
        if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };
    [
        unlin(m[0][0] * r + m[0][1] * g + m[0][2] * b),
        unlin(m[1][0] * r + m[1][1] * g + m[1][2] * b),
        unlin(m[2][0] * r + m[2][1] * g + m[2][2] * b),
    ]
}

#[test]
fn every_theme_has_six_legible_palette_slots() {
    let blocks = blocks();
    assert!(
        blocks.len() >= 60,
        "expected the theme blocks, found {}",
        blocks.len()
    );
    let mut failures = Vec::new();
    let mut close = Vec::new();
    for b in &blocks {
        let Some(bg) = b.get("bg-primary").and_then(hex) else {
            failures.push(format!("{}: no bg-primary", b.name));
            continue;
        };
        let mut six = Vec::new();
        for i in 1..=6 {
            match b.get(&format!("palette-{i}")).and_then(hex) {
                Some(c) => six.push(c),
                None => failures.push(format!("{}: palette-{i} missing or not a hex", b.name)),
            }
        }
        if six.len() != 6 {
            continue;
        }
        for (i, c) in six.iter().enumerate() {
            let cr = contrast(*c, bg);
            if cr < 3.0 {
                failures.push(format!(
                    "{}: palette-{} is {cr:.2}:1 on the background",
                    b.name,
                    i + 1
                ));
            }
        }
        match b.get("color-money").and_then(hex) {
            Some(c) => {
                let cr = contrast(c, bg);
                if cr < 3.0 {
                    failures.push(format!(
                        "{}: color-money is {cr:.2}:1 on the background",
                        b.name
                    ));
                }
            }
            None => failures.push(format!("{}: color-money missing or not a hex", b.name)),
        }
        let base = b
            .name
            .rsplit_once('-')
            .map_or(b.name.as_str(), |(base, _)| base);
        let kind = match base {
            "protanopia" | "deuteranopia" | "tritanopia" | "achromatopsia" => base,
            _ => "",
        };
        for i in 0..6 {
            for j in i + 1..6 {
                let (Some(a), Some(c)) = (six.get(i), six.get(j)) else {
                    continue;
                };
                let d = distance(*a, *c);
                if d < 15.0 {
                    close.push(format!(
                        "{}: palette-{} and palette-{} are {d:.0} apart",
                        b.name,
                        i + 1,
                        j + 1
                    ));
                }
                if !kind.is_empty() {
                    let ds = distance(simulate(*a, kind), simulate(*c, kind));
                    if ds < 8.0 {
                        failures.push(format!(
                            "{}: palette-{} and palette-{} are {ds:.0} apart under {kind}",
                            b.name,
                            i + 1,
                            j + 1
                        ));
                    }
                }
            }
        }
    }
    if !close.is_empty() {
        println!("palette pairs under 15 (reported, not enforced):");
        for c in &close {
            println!("  {c}");
        }
    }
    assert!(
        failures.is_empty(),
        "palette floors failed:\n  {}",
        failures.join("\n  ")
    );
}
