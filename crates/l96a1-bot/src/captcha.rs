//! Captcha em imagem (Fase 3): 5 chars, fonte bitmap 5x7 própria + ruído.
//! Sem arquivos de fonte: os glifos vivem nesta tabela.

use image::{Rgb, RgbImage};
use rand::RngExt;

/// Charset sem ambíguos (sem 0/O, 1/I/L).
pub const CHARSET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Glifos 5x7 ('#' = tinta). Ordem = CHARSET.
const GLYPHS: [&str; 31] = [
    ".###.#...##...#######...##...##...#", // A
    "####.#...##...#####.#...##...#####.", // B
    ".###.#....#....#....#....#.....###.", // C
    "####.#...##...##...##...##...#####.", // D
    "######....#....####.#....#....#####", // E
    "######....#....####.#....#....#....", // F
    ".###.#....#....#.####...##...#.###.", // G
    "#...##...##...#######...##...##...#", // H
    "..###...#....#....#....#.#..#..##..", // J
    "#...##..#.#.#..##...#.#..#..#.#...#", // K
    "#...###.###.#.##.#.##...##...##...#", // M
    "#...###..##.#.##.#.##..###..###...#", // N
    "####.#...##...#####.#....#....#....", // P
    ".###.#...##...##...##.#.##..#..##.#", // Q
    "####.#...##...#####.#.#..#..#.#...#", // R
    ".#####....#.....###.....#....#####.", // S
    "#####..#....#....#....#....#....#..", // T
    "#...##...##...##...##...##...#.###.", // U
    "#...##...##...##...##...#.#.#...#..", // V
    "#...##...##...##.#.##.#.###.###...#", // W
    "#...##...#.#.#...#...#.#.#...##...#", // X
    "#...##...#.#.#...#....#....#....#..", // Y
    "#####....#...#...#...#...#....#####", // Z
    ".###.#...#....#...#...#...#...#####", // 2
    "#####...#...#.....#.....##...#.###.", // 3
    "...#...##..#.#.#..#.#####...#....#.", // 4
    "######....####.....#....##...#.###.", // 5
    "..##..#...#....####.#...##...#.###.", // 6
    "#####....#...#...#...#....#....#...", // 7
    ".###.#...##...#.###.#...##...#.###.", // 8
    ".###.#...##...#.####....#...#..##..", // 9
];

fn glyph(ch: u8) -> &'static str {
    CHARSET
        .iter()
        .position(|&c| c == ch)
        .map(|i| GLYPHS[i])
        .unwrap_or(GLYPHS[0])
}

/// Gera (solução, png_bytes). `len` chars (default 5).
pub fn generate(len: usize) -> (String, Vec<u8>) {
    let mut rng = rand::rng();
    let len = len.clamp(4, 8);
    let idx: Vec<usize> = (0..len).map(|_| rng.random_range(0..CHARSET.len())).collect();
    let solution: String = idx.iter().map(|&i| CHARSET[i] as char).collect();

    let scale: u32 = 6;
    let cw = 5 * scale;
    let chh = 7 * scale;
    let pad: u32 = 12;
    let gap: u32 = 8;
    let w = pad * 2 + len as u32 * cw + (len as u32 - 1) * gap;
    let h = pad * 2 + chh;
    let mut img = RgbImage::new(w, h);
    // fundo quase-branco com leve gradiente
    for (x, y, px) in img.enumerate_pixels_mut() {
        let v = 235 + ((x * 7 + y * 13) % 20) as u8;
        *px = Rgb([v, v, v]);
    }
    // texto escuro com deslocamento vertical aleatório por char
    for (i, &gi) in idx.iter().enumerate() {
        let g = glyph(CHARSET[gi]);
        let ox = pad + i as u32 * (cw + gap);
        let oy = pad + rng.random_range(0..=6);
        let ink = Rgb([
            rng.random_range(10..70),
            rng.random_range(10..70),
            rng.random_range(10..70),
        ]);
        for (r, row) in g.as_bytes().chunks_exact(5).enumerate() {
            for (c, &b) in row.iter().enumerate() {
                if b != b'#' {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = ox + c as u32 * scale + dx;
                        let y = oy + r as u32 * scale + dy;
                        if x < w && y < h {
                            img.put_pixel(x, y, ink);
                        }
                    }
                }
            }
        }
    }
    // ruído: linhas + pontos
    for _ in 0..3 {
        let y0 = rng.random_range(0..h);
        let slope = rng.random_range(-4..=4);
        for x in 0..w {
            let y = (y0 as i32 + slope * x as i32 / w as i32 * 10).clamp(0, h as i32 - 1) as u32;
            img.put_pixel(x, y, Rgb([120, 120, 120]));
        }
    }
    for _ in 0..(w * h / 40) {
        img.put_pixel(rng.random_range(0..w), rng.random_range(0..h), Rgb([90, 90, 90]));
    }
    let mut buf = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder;
    enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .expect("png encode");
    (solution, buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glyphs_are_5x7() {
        assert_eq!(CHARSET.len(), GLYPHS.len());
        for (i, g) in GLYPHS.iter().enumerate() {
            assert_eq!(g.len(), 35, "glifo {} ({})", i, CHARSET[i] as char);
        }
    }

    #[test]
    fn captcha_sample_to_temp() {
        let dir = std::env::temp_dir().join("l96a1_captcha");
        let _ = std::fs::create_dir_all(&dir);
        for i in 0..3 {
            let (sol, png) = generate(5);
            std::fs::write(dir.join(format!("sample{i}_{sol}.png")), &png).unwrap();
        }
    }

    #[test]
    fn captcha_generates_png() {
        let (sol, png) = generate(5);
        assert_eq!(sol.len(), 5);
        assert!(png.starts_with(&[137, 80, 78, 71])); // assinatura PNG
        assert!(sol.bytes().all(|b| CHARSET.contains(&b)));
    }
}
