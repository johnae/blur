// This was taken more or less wholesale from https://github.com/fschutt/fastblur with a few deletions. Compared to the imagemagick/convert based blurring - this is A LOT faster since this is an approximation rather than a perfect gaussian blur, but you don't really notice that. Originally this algorithm is from here: http://blog.ivank.net/fastest-gaussian-blur.html

use std::cmp::min;

pub fn gaussian_blur(data: &mut Vec<[u8; 3]>, width: usize, height: usize, blur_radius: f32) {
    let boxes = create_box_gauss(blur_radius, 3);
    let mut backbuf = data.clone();

    for box_size in boxes.iter() {
        let radius = ((box_size - 1) / 2) as usize;
        box_blur(&mut backbuf, data, width, height, radius, radius);
    }
}

#[inline]
fn create_box_gauss(sigma: f32, n: usize) -> Vec<i32> {
    if sigma > 0.0 {
        let n_float = n as f32;

        let w_ideal = (12.0 * sigma * sigma / n_float).sqrt() + 1.0;
        let mut wl: i32 = w_ideal.floor() as i32;

        if wl % 2 == 0 {
            wl -= 1;
        };

        let wu = wl + 2;

        let wl_float = wl as f32;
        let m_ideal = (12.0 * sigma * sigma
            - n_float * wl_float * wl_float
            - 4.0 * n_float * wl_float
            - 3.0 * n_float)
            / (-4.0 * wl_float - 4.0);
        let m: usize = m_ideal.round() as usize;

        let mut sizes = Vec::<i32>::new();

        for i in 0..n {
            if i < m {
                sizes.push(wl);
            } else {
                sizes.push(wu);
            }
        }

        sizes
    } else {
        vec![1; n]
    }
}

#[inline]
fn box_blur(
    backbuf: &mut Vec<[u8; 3]>,
    frontbuf: &mut Vec<[u8; 3]>,
    width: usize,
    height: usize,
    blur_radius_horz: usize,
    blur_radius_vert: usize,
) {
    box_blur_horz(backbuf, frontbuf, width, height, blur_radius_horz);
    box_blur_vert(frontbuf, backbuf, width, height, blur_radius_vert);
}

#[inline]
fn box_blur_vert(
    backbuf: &[[u8; 3]],
    frontbuf: &mut [[u8; 3]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    if blur_radius == 0 {
        frontbuf.copy_from_slice(backbuf);
        return;
    }

    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for i in 0..width {
        let col_start = i;
        let col_end = i + width * (height - 1);
        let mut ti: usize = i;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius * width;

        let fv: [u8; 3] = backbuf[col_start];
        let lv: [u8; 3] = backbuf[col_end];

        let mut val_r: isize = (blur_radius as isize + 1) * isize::from(fv[0]);
        let mut val_g: isize = (blur_radius as isize + 1) * isize::from(fv[1]);
        let mut val_b: isize = (blur_radius as isize + 1) * isize::from(fv[2]);

        let get_top = |i: usize| {
            if i < col_start {
                fv
            } else {
                backbuf[i]
            }
        };

        let get_bottom = |i: usize| {
            if i > col_end {
                lv
            } else {
                backbuf[i]
            }
        };

        for j in 0..min(blur_radius, height) {
            let bb = backbuf[ti + j * width];
            val_r += isize::from(bb[0]);
            val_g += isize::from(bb[1]);
            val_b += isize::from(bb[2]);
        }
        if blur_radius > height {
            val_r += (blur_radius - height) as isize * isize::from(lv[0]);
            val_g += (blur_radius - height) as isize * isize::from(lv[1]);
            val_b += (blur_radius - height) as isize * isize::from(lv[2]);
        }

        for _ in 0..min(height, blur_radius + 1) {
            let bb = get_bottom(ri);
            ri += width;
            val_r += isize::from(bb[0]) - isize::from(fv[0]);
            val_g += isize::from(bb[1]) - isize::from(fv[1]);
            val_b += isize::from(bb[2]) - isize::from(fv[2]);

            frontbuf[ti] = [
                round(val_r as f32 * iarr) as u8,
                round(val_g as f32 * iarr) as u8,
                round(val_b as f32 * iarr) as u8,
            ];
            ti += width;
        }

        if height > blur_radius {
            for _ in (blur_radius + 1)..(height - blur_radius) {
                let bb1 = backbuf[ri];
                ri += width;
                let bb2 = backbuf[li];
                li += width;

                val_r += isize::from(bb1[0]) - isize::from(bb2[0]);
                val_g += isize::from(bb1[1]) - isize::from(bb2[1]);
                val_b += isize::from(bb1[2]) - isize::from(bb2[2]);

                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                ];
                ti += width;
            }

            for _ in 0..min(height - blur_radius - 1, blur_radius) {
                let bb = get_top(li);
                li += width;

                val_r += isize::from(lv[0]) - isize::from(bb[0]);
                val_g += isize::from(lv[1]) - isize::from(bb[1]);
                val_b += isize::from(lv[2]) - isize::from(bb[2]);

                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                ];
                ti += width;
            }
        }
    }
}

#[inline]
fn box_blur_horz(
    backbuf: &[[u8; 3]],
    frontbuf: &mut [[u8; 3]],
    width: usize,
    height: usize,
    blur_radius: usize,
) {
    if blur_radius == 0 {
        frontbuf.copy_from_slice(backbuf);
        return;
    }

    let iarr = 1.0 / (blur_radius + blur_radius + 1) as f32;

    for i in 0..height {
        let row_start: usize = i * width;
        let row_end: usize = (i + 1) * width - 1;
        let mut ti: usize = i * width;
        let mut li: usize = ti;
        let mut ri: usize = ti + blur_radius;

        let fv: [u8; 3] = backbuf[row_start];
        let lv: [u8; 3] = backbuf[row_end];

        let mut val_r: isize = (blur_radius as isize + 1) * isize::from(fv[0]);
        let mut val_g: isize = (blur_radius as isize + 1) * isize::from(fv[1]);
        let mut val_b: isize = (blur_radius as isize + 1) * isize::from(fv[2]);

        let get_left = |i: usize| {
            if i < row_start {
                fv
            } else {
                backbuf[i]
            }
        };

        let get_right = |i: usize| {
            if i > row_end {
                lv
            } else {
                backbuf[i]
            }
        };

        for j in 0..min(blur_radius, width) {
            let bb = backbuf[ti + j];
            val_r += isize::from(bb[0]);
            val_g += isize::from(bb[1]);
            val_b += isize::from(bb[2]);
        }
        if blur_radius > width {
            val_r += (blur_radius - height) as isize * isize::from(lv[0]);
            val_g += (blur_radius - height) as isize * isize::from(lv[1]);
            val_b += (blur_radius - height) as isize * isize::from(lv[2]);
        }

        for _ in 0..min(width, blur_radius + 1) {
            let bb = get_right(ri);
            ri += 1;
            val_r += isize::from(bb[0]) - isize::from(fv[0]);
            val_g += isize::from(bb[1]) - isize::from(fv[1]);
            val_b += isize::from(bb[2]) - isize::from(fv[2]);

            frontbuf[ti] = [
                round(val_r as f32 * iarr) as u8,
                round(val_g as f32 * iarr) as u8,
                round(val_b as f32 * iarr) as u8,
            ];
            ti += 1;
        }

        if width > blur_radius {
            for _ in (blur_radius + 1)..(width - blur_radius) {
                let bb1 = backbuf[ri];
                ri += 1;
                let bb2 = backbuf[li];
                li += 1;

                val_r += isize::from(bb1[0]) - isize::from(bb2[0]);
                val_g += isize::from(bb1[1]) - isize::from(bb2[1]);
                val_b += isize::from(bb1[2]) - isize::from(bb2[2]);

                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                ];
                ti += 1;
            }

            for _ in 0..min(width - blur_radius - 1, blur_radius) {
                let bb = get_left(li);
                li += 1;

                val_r += isize::from(lv[0]) - isize::from(bb[0]);
                val_g += isize::from(lv[1]) - isize::from(bb[1]);
                val_b += isize::from(lv[2]) - isize::from(bb[2]);

                frontbuf[ti] = [
                    round(val_r as f32 * iarr) as u8,
                    round(val_g as f32 * iarr) as u8,
                    round(val_b as f32 * iarr) as u8,
                ];
                ti += 1;
            }
        }
    }
}

#[inline]
/// Source: https://stackoverflow.com/a/42386149/585725
fn round(mut x: f32) -> f32 {
    x += 12582912.0;
    x -= 12582912.0;
    x
}
