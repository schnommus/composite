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

/// Dumb O(n*m) filter
pub fn filter(signal: &[f32], kernel: &[f32], result: &mut [f32]) {
    assert!(result.len() == signal.len());
    for n in kernel.len()..signal.len() {
        for k in 0..kernel.len() {
            result[n] += signal[n-k] * kernel[k];
        }
    }
}
