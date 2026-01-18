use rand::Rng;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum PowerPolicy {
    /// Always runs inference (Subject to frequency). Risk taker.
    Aggressive,

    /// Only runs if battery is healthy (> 50%). Safe but low score.
    Conservative,

    /// Adapts to environment: Runs if Solar is present OR Battery is high. Sleeps at night if low.
    SmartAdaptive,
}

impl PowerPolicy {
    /// Decides whether to run inference based on current state
    pub fn should_infer(
        &self,
        battery_wh: f32,
        solar_output_w: f32,
        base_probability: f32,
    ) -> bool {
        let mut rng = rand::rng();

        // Base probabilistic check (Gene frequency)
        if !rng.random_bool(base_probability as f64) {
            return false;
        }

        match self {
            PowerPolicy::Aggressive => {
                // Ignores battery status (until empty)
                true
            }
            PowerPolicy::Conservative => {
                // Requires > 50% charge (assuming 40Wh max)
                battery_wh > 20.0
            }
            PowerPolicy::SmartAdaptive => {
                // If Solar is active (> 5W), run freely.
                // If Night/Cloudy, conserve unless battery is robust (> 30%).
                if solar_output_w > 5.0 {
                    true
                } else {
                    battery_wh > 12.0 // 30% of 40Wh
                }
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PowerPolicy::Aggressive => "Aggressive",
            PowerPolicy::Conservative => "Conservative",
            PowerPolicy::SmartAdaptive => "SmartAdaptive",
        }
    }
}
