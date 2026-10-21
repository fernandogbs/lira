/// Linear resample mono `f32` PCM from `src_rate` Hz to 16 kHz.
pub fn resample_to_16k_mono(input: &[f32], src_rate: u32) -> Vec<f32> {
    const DST_RATE: u32 = 16_000;
    if src_rate == 0 || input.is_empty() {
        return Vec::new();
    }
    if src_rate == DST_RATE {
        return input.to_vec();
    }

    let out_len = ((input.len() as u64) * DST_RATE as u64 / src_rate as u64) as usize;
    if out_len == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = (i as f64) * src_rate as f64 / DST_RATE as f64;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let s0 = input.get(idx).copied().unwrap_or(0.0);
        let s1 = input.get(idx + 1).copied().unwrap_or(s0);
        out.push(s0 + (s1 - s0) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_at_16k() {
        let input = vec![0.0, 1.0, 0.0, -1.0];
        assert_eq!(resample_to_16k_mono(&input, 16_000), input);
    }

    #[test]
    fn downsamples_48k_to_16k_length() {
        let input: Vec<f32> = (0..480).map(|i| (i as f32) * 0.01).collect();
        let out = resample_to_16k_mono(&input, 48_000);
        assert_eq!(out.len(), 160);
    }
}
