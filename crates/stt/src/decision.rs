use lira_core::settings::SttModel;

/// Pick default whisper model from a measured turbo latency (seconds per 10s of audio).
pub fn choose_default(turbo_secs_per_10s: Option<f64>, threshold_secs: f64) -> SttModel {
    match turbo_secs_per_10s {
        Some(t) if t <= threshold_secs => SttModel::LargeV3TurboQ5_0,
        _ => SttModel::SmallQ5_1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turbo_within_threshold() {
        assert_eq!(choose_default(Some(2.5), 3.0), SttModel::LargeV3TurboQ5_0);
    }

    #[test]
    fn turbo_above_threshold() {
        assert_eq!(choose_default(Some(3.5), 3.0), SttModel::SmallQ5_1);
    }

    #[test]
    fn no_measurement_defaults_small() {
        assert_eq!(choose_default(None, 3.0), SttModel::SmallQ5_1);
    }

    #[test]
    fn boundary_is_turbo() {
        assert_eq!(choose_default(Some(3.0), 3.0), SttModel::LargeV3TurboQ5_0);
    }
}
