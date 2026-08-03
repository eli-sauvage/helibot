use crate::config::RoleTier;

/// Anything that can be ranked by how much voice time it requires.
pub trait Threshold {
    fn threshold_seconds(&self) -> u64;
}

impl Threshold for RoleTier {
    fn threshold_seconds(&self) -> u64 {
        RoleTier::threshold_seconds(self)
    }
}

/// The single rank a member has earned: the highest tier whose threshold they reached.
///
/// Returns `None` when they are below every configured threshold, which only happens if
/// the ladder has no tier at 0.
pub fn tier_for<T: Threshold>(points_seconds: u64, tiers: &[T]) -> Option<&T> {
    tiers
        .iter()
        .filter(|tier| points_seconds >= tier.threshold_seconds())
        .max_by_key(|tier| tier.threshold_seconds())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ladder() -> Vec<RoleTier> {
        [
            (0, "Subalterne"),
            (500, "Cul-terreux"),
            (1500, "Strapontin"),
        ]
        .into_iter()
        .map(|(seuil, role_name)| RoleTier {
            seuil,
            role_name: role_name.to_owned(),
        })
        .collect()
    }

    #[test]
    fn thresholds_are_minutes_but_points_are_seconds() {
        let tiers = ladder();
        // 499 minutes of voice time is not yet 500.
        assert_eq!(tier_for(499 * 60, &tiers).unwrap().role_name, "Subalterne");
        assert_eq!(tier_for(500 * 60, &tiers).unwrap().role_name, "Cul-terreux");
        // 500 raw seconds must not be mistaken for 500 minutes.
        assert_eq!(tier_for(500, &tiers).unwrap().role_name, "Subalterne");
    }

    #[test]
    fn the_highest_reached_tier_wins() {
        let tiers = ladder();
        assert_eq!(
            tier_for(99_999 * 60, &tiers).unwrap().role_name,
            "Strapontin"
        );
    }

    #[test]
    fn no_tier_below_the_lowest_threshold() {
        let tiers = vec![RoleTier {
            seuil: 10,
            role_name: "Dixième".to_owned(),
        }];
        assert!(tier_for(0, &tiers).is_none());
    }

    #[test]
    fn empty_ladder_is_not_an_error() {
        assert!(tier_for::<RoleTier>(1_000_000, &[]).is_none());
    }
}
