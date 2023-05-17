use super::{Hz, Second, StereoIn, StereoOut};

pub fn rotating(
    rate: Hz,
    angle: f64,
    left_time: Second,
    right_time: Second,
    left_gain: f64,
    right_gain: f64,
) -> impl FnMut(StereoIn, StereoOut) {
    let (p, q, r, s) = {
        let (sin, cos) = angle.sin_cos();
        (cos, sin, -sin, cos)
    };

    let left_time = (left_time * rate) as usize;
    let right_time = (right_time * rate) as usize;
    let left_makeup = 1.0 / (1.0 + left_gain);
    let right_makeup = 1.0 / (1.0 + right_gain);

    let mut buffer_left = vec![0.0; left_time];
    let mut buffer_right = vec![0.0; right_time];

    let mut head_left = 0;
    let mut head_right = 0;

    move |(left_in, right_in), (left_out, right_out)| {
        let inputs = left_in.iter().copied().zip(right_in.iter().copied());
        let outputs = left_out.iter_mut().zip(right_out.iter_mut());

        for ((x1, x2), (y1, y2)) in inputs.zip(outputs) {
            *y1 = x1 + left_gain * buffer_left[head_left];
            *y2 = x2 + right_gain * buffer_right[head_right];

            *y1 *= left_makeup;
            *y2 *= right_makeup;

            buffer_left[head_left] = p * *y1 + q * *y2;
            buffer_right[head_right] = r * *y1 + s * *y2;

            head_left = (head_left + 1) % left_time;
            head_right = (head_right + 1) % right_time;
        }
    }
}
