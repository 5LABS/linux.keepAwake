use ksni::Icon;

const SIZE: i32 = 32;
const SS: i32 = 4; // supersampling factor for anti-aliasing

/// Test whether a normalized point (u, v in 0..1) belongs to the coffee-cup
/// drawing: a tapered cup body, a handle on the right, and two steam wisps.
/// Coordinates are normalized so the cup fills almost the whole icon frame.
fn inside_norm(u: f32, v: f32) -> bool {
    // Cup body: trapezoid, slightly narrower at the bottom.
    let body = if (0.34..=0.97).contains(&v) {
        let t = (v - 0.34) / (0.97 - 0.34);
        let left = 0.05 + t * (0.16 - 0.05);
        let right = 0.80 + t * (0.66 - 0.80);
        u >= left && u <= right
    } else {
        false
    };

    // Handle: a partial ring on the right side of the cup.
    let handle = {
        let dx = u - 0.80;
        let dy = v - 0.62;
        let d = (dx * dx + dy * dy).sqrt();
        (0.15..=0.255).contains(&d) && u >= 0.78 && (0.42..=0.84).contains(&v)
    };

    // Steam: two short wavy wisps rising above the cup.
    let steam = if (0.02..=0.30).contains(&v) {
        let wob = (v * 11.0).sin() * 0.045;
        (u - (0.32 + wob)).abs() < 0.05 || (u - (0.56 - wob)).abs() < 0.05
    } else {
        false
    };

    body || handle || steam
}

/// Render the tray icon procedurally (no asset files, no image crate).
/// Active = green coffee cup; inactive = grey coffee cup.
/// Data is ARGB32 in network byte order, as required by the SNI spec.
pub fn render(active: bool) -> Vec<Icon> {
    let (red, green, blue) = if active {
        (0x4C, 0xC2, 0x5E) // green
    } else {
        (0x9E, 0x9E, 0x9E) // grey
    };

    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            // Coverage via supersampling -> smooth edges.
            let mut hits = 0;
            for sy in 0..SS {
                for sx in 0..SS {
                    let u = (x as f32 + (sx as f32 + 0.5) / SS as f32) / SIZE as f32;
                    let v = (y as f32 + (sy as f32 + 0.5) / SS as f32) / SIZE as f32;
                    if inside_norm(u, v) {
                        hits += 1;
                    }
                }
            }
            if hits == 0 {
                continue;
            }
            let alpha = (hits * 255 / (SS * SS)) as u8;
            let idx = ((y * SIZE + x) * 4) as usize;
            data[idx] = alpha;
            data[idx + 1] = red;
            data[idx + 2] = green;
            data[idx + 3] = blue;
        }
    }

    vec![Icon {
        width: SIZE,
        height: SIZE,
        data,
    }]
}
