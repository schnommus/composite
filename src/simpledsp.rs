use std::f32::consts::PI;

/// Window functions
pub enum WindowType {
    Rectangular,
    Hamming,
    BlackmanHarris
}

/// Generate a window function
/// Only odd orders are accepted
pub fn generate_window(window_type: WindowType, window_len: u32) -> Vec<f32> {
    assert!(window_len % 2 == 1);

    let samples = (1..window_len).map(|e| e as f32);
    let m = window_len as f32;

    let window_function = |n: f32| {
        match window_type {
            WindowType::Rectangular => 1.0,
            WindowType::Hamming =>
                0.54 - 0.46 * f32::cos((2.0*n*PI)/m),
            WindowType::BlackmanHarris =>
                0.35875 - 0.48829 * f32::cos((2.0*n*PI)/m)
                        + 0.14128 * f32::cos((4.0*n*PI)/m)
                        - 0.01168 * f32::cos((6.0*n*PI)/m)
        }
    };

    samples.map(window_function)
           .collect()
}

/// Types of filter to generate.
/// Cutoffs are omega/pi - i.e 1 is nyquist frequency
pub enum FilterType {
    LowPass(f32),
    HighPass(f32),
    BandPass(f32, f32),
    BandStop(f32, f32)
}

/// Design FIR filter coefficients of the given order
/// Only odd orders are accepted
pub fn fir_design(filter_type: FilterType, window_type: WindowType, filter_len: u32) -> Vec<f32> {
    assert!(filter_len % 2 == 1);

    let samples = (1..filter_len).map(|e| e as f32);
    let m = filter_len as f32;
    let sinc = |x: f32| f32::sin(x)/x;

    let filter_fn = |n: f32| {
        let lowpass = |c: f32| c * sinc(c * PI * (n - m / 2.0));
        match filter_type {
            FilterType::LowPass(c) =>
                lowpass(c),
            FilterType::HighPass(c) =>
                lowpass(1.0) - lowpass(c),
            FilterType::BandPass(cl, ch) =>
                lowpass(ch) - lowpass(cl),
            FilterType::BandStop(cl, ch) =>
                lowpass(1.0) - lowpass(ch) + lowpass(cl)
        }
    };

    samples.map(filter_fn)
           .zip(generate_window(window_type, filter_len))
           .map(|(a, b)| a * b)
           .collect()
}

/// Simple, O(n^2), real -> abs mag dft
pub fn dft(x: &[f32], n_samples: usize) -> Vec<f32> {

    let mut y_re = vec![0.0; n_samples];
    let mut y_im = vec![0.0; n_samples];

    let m = x.len();

    for k in 0..n_samples {
        for n in 0..(m-1) {
            y_re[k] += x[n] * f32::cos(2.0 * PI * k as f32 * n as f32 / n_samples as f32);
            y_im[k] -= x[n] * f32::sin(2.0 * PI * k as f32 * n as f32 / n_samples as f32);
        }
    }

    y_re.iter()
        .zip(y_im)
        .map(|(a, b)| f32::sqrt(a*a + b*b))
        .collect()
}

/// Simple O(n*m) convolution
pub fn convolve(signal: &[f32], kernel: &[f32]) -> Vec<f32> {
    let result_len = signal.len() + kernel.len() - 1;
    let mut y = vec![0.0; result_len];
    for n in 0..result_len {
        let kmin = if n >= kernel.len() - 1 {n - (kernel.len() - 1)} else {0};
        let kmax = if n < signal.len() - 1 {n} else {signal.len() - 1};
        for k in kmin..kmax {
            y[n] += signal[k] * kernel[n - k];
        }
    }
    y
}

/// Applies convolution, but corrects for time delay and culls to signal length
pub fn filter(signal: &[f32], kernel: &[f32]) -> Vec<f32> {
    let mut ret = convolve(signal, kernel).split_off(kernel.len()/2);
    ret.resize(signal.len(), 0.0);
    ret
}
