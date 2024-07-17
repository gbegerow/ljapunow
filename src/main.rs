// Wikipedia https://de.wikipedia.org/wiki/Ljapunow-Diagramm
// Dann werden für Werte ( a , b ) aus Intervallen, die – um interessante Figuren zu bekommen – meist im Bereich 0 ≤ a ≤ 4 und 0 ≤ b ≤ 4
// gewählt werden, jeweils die Iterationswerte der logistischen Gleichung und der Ljapunow-Exponent berechnet:
//     λ = lim N → ∞ 1 N ∑ n = 1 N log ⁡ | d x n + 1 d x n | = lim N → ∞ 1 N ∑ n = 1 N log ⁡ | r_n ( 1 − 2 x n ) |
// Ist der Wert von λ < 0, wählt man für den Punkt mit den Koordinaten ( a , b ) z. B. gelb als Farbe,
// ist er größer als Null (was zu exponentiellem Wachstum führt, Chaos), wählt man z. B. blau als Farbe.
// Entsprechend kann man die Farbwerte noch abstufen je nach der Größe von λ.
// Das Ergebnis ist das Ljapunow-Diagramm, das häufig fraktaler Natur ist.
// Ein Beispiel ist das Diagramm Zircon Zity, gebildet mit 3,4 ≤ a ≤ 4,0  und 2,5 ≤ b ≤ 3,4 und der Sequenz „BBBBBBAAAAAA“.

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use std::env;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;
const ITERATION_DEPTH: u32 = 300; // everything from 100+ seems to be fine
const WARMUP: u32 = 20;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (mut buffer, mut window) = init_window();

    let sequence_rule = args
        .get(1)
        .unwrap_or(&"BBBBBBAAAAAA".to_string())
        .chars()
        .collect::<Vec<_>>();
    let seq_len = sequence_rule.len();

    // todo: read ranges from args,
    let x_min = 3.4;
    let x_max = 4.0;
    let y_min = 2.5;
    let y_max = 3.4;

    let mut lambda_min = 5.0e5;
    let mut lambda_max = 0.0;

    // while window.is_open() && !window.is_key_down(Key::Escape) {
    for (i, pixel) in buffer.iter_mut().enumerate() {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        // map pixel to world coordinates
        let a = map((i % WIDTH) as f64, 0., WIDTH as f64, x_min, x_max);
        let b = map((i / HEIGHT) as f64, 0., HEIGHT as f64, y_min, y_max);

        // map sequence rules to actual values outside of inner loop
        let sequence = sequence_rule
            .iter()
            .map(|r| match r {
                'A' => a,
                'B' => b,
                _ => panic!("Invalid sequence"),
            })
            .collect::<Vec<_>>();
        let r = |n| sequence[n as usize % seq_len];

        let mut x_n = 0.5; // X_0 as start of iteration
        let mut lambda = 0.0;

        for n in 0..ITERATION_DEPTH {
            // ignore the first iterations or we always have -inf as first value as log(1-2*0.5) = log(0) = -inf
            if n > WARMUP || x_n != 0.5 {
                // sum for ljapunow exponent
                lambda += (r(n) * (1.0 - 2.0 * x_n)).abs().ln();
            }

            // iterate x to next value
            x_n = r(n) * x_n * (1.0 - x_n);

            // shortcut if we are already out of bounds
            if lambda > 1e12 || lambda < -1e12 {
                break;
            }
        }
        lambda /= (ITERATION_DEPTH - WARMUP) as f64;

        if lambda < lambda_min {
            lambda_min = lambda;
        }
        if lambda > lambda_max {
            lambda_max = lambda;
        }
        // println!("lambda {lambda} a {a} b {b}");

        // map to color
        *pixel = if lambda > 0.0 {
            0x00
        } else {
            color_ramp(lambda)
            //color_gradient(lambda)
            //0xFF
        };

        // how to update window while buffer is borrowed mutable?
    }

    println!("λ: ({lambda_min}..{lambda_max})");

    // We unwrap here as we want this code to exit if it fails
    window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

    // wait for window close
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }
}
// }

fn init_window() -> (Vec<u32>, Window) {
    let buffer = vec![0u32; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Ljapunow-Markus-Diagramm - press ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X1, // scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create the window");
    window.set_target_fps(60);
    window.set_background_color(0, 0, 20);

    (buffer, window)
}

// map / lerp between to ranges
fn map(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    start2 + (stop2 - start2) * ((val - start1) / (stop1 - start1))
}

// map to a byte range and shift in target range. 0 for values outside of range.
fn map_byte(val: f64, start1: f64, stop1: f64, start2: f64, stop2: f64, shift: u32) -> u32 {
    if val < start1 || val > stop1 {
        return 0;
    }

    (map(val, start1, stop1, start2, stop2)
        .round()
        .clamp(0.0, 255.0) as u32)
        << shift
}

const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;
const BLUE_SHIFT: u32 = 0;
// simple RGB ramp
#[allow(dead_code)]
fn color_ramp(lambda: f64) -> u32 {
    map_byte(lambda, -2.0, 0.5, 196.0, 255.0, RED_SHIFT)
        + map_byte(lambda, -0.5, 0.0, 0.0, 255.0, GREEN_SHIFT)
        + map_byte(lambda, -2.5, 0.5, 10.0, 55.0, BLUE_SHIFT)
}

// interpolate along a color gradient
#[allow(dead_code)]
fn color_gradient(lambda: f64) -> u32 {
    let gradient = [0x161c31, 0x613c62, 0xb75f74, 0xf29a6b, 0xfaec70];
    let ranges = [-2.5, -1.5, -0.8, -0.2, 0.0, 4.0];

    // find the range via simple search, no need for binary
    let mut pos = 1;
    while pos < ranges.len() && ranges[pos] < lambda {
        pos += 1;
    }

    // -0.3 -> pos 3 -> gradient[2]..gradient[3]
    let g1 = gradient[pos - 1] as f64;
    let g2 = gradient[pos] as f64;
    // todo: interpolate in hsl or lab space, rgb is not good for linear interpolation
    map(lambda, ranges[pos - 1], ranges[pos], g1, g2) as u32
}
