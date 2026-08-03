use std::collections::{BTreeMap, BTreeSet};

/// A member seen in a voice channel, reduced to the facts that decide eligibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceMember {
    pub user_id: u64,
    pub channel_id: u64,
    pub is_bot: bool,
    pub muted: bool,
    pub deafened: bool,
}

impl VoiceMember {
    pub fn is_eligible(&self) -> bool {
        !self.is_bot && !self.muted && !self.deafened
    }
}

/// Talking to yourself does not pay: a channel needs this many eligible members before
/// anyone in it earns.
pub const MIN_MEMBERS_PER_CHANNEL: usize = 2;

/// Users who should have an open session right now, across every voice channel of a
/// guild.
///
/// This is deliberately computed for the whole guild at once. The previous version
/// reconciled a single channel against the guild's full session list, so activity in one
/// channel terminated the sessions of everyone in every other channel.
pub fn earning_users(members: &[VoiceMember]) -> BTreeSet<u64> {
    let mut by_channel: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for member in members.iter().filter(|member| member.is_eligible()) {
        by_channel
            .entry(member.channel_id)
            .or_default()
            .push(member.user_id);
    }

    by_channel
        .into_values()
        .filter(|users| users.len() >= MIN_MEMBERS_PER_CHANNEL)
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(user_id: u64, channel_id: u64) -> VoiceMember {
        VoiceMember {
            user_id,
            channel_id,
            is_bot: false,
            muted: false,
            deafened: false,
        }
    }

    #[test]
    fn alone_in_a_channel_earns_nothing() {
        let members = vec![member(1, 100)];
        assert!(earning_users(&members).is_empty());
    }

    #[test]
    fn two_in_a_channel_both_earn() {
        let members = vec![member(1, 100), member(2, 100)];
        assert_eq!(earning_users(&members), BTreeSet::from([1, 2]));
    }

    #[test]
    fn channels_are_counted_separately() {
        // Two people in 100, one alone in 200: the loner earns nothing, and crucially
        // his presence does not disturb the pair.
        let members = vec![member(1, 100), member(2, 100), member(3, 200)];
        assert_eq!(earning_users(&members), BTreeSet::from([1, 2]));
    }

    #[test]
    fn bots_muted_and_deafened_do_not_count_towards_the_minimum() {
        let bot = VoiceMember {
            is_bot: true,
            ..member(2, 100)
        };
        let muted = VoiceMember {
            muted: true,
            ..member(3, 100)
        };
        let deafened = VoiceMember {
            deafened: true,
            ..member(4, 100)
        };
        let members = vec![member(1, 100), bot, muted, deafened];
        assert!(
            earning_users(&members).is_empty(),
            "one eligible human plus three ineligible members is still one human"
        );
    }

    #[test]
    fn a_muted_member_does_not_earn_among_active_ones() {
        let muted = VoiceMember {
            muted: true,
            ..member(3, 100)
        };
        let members = vec![member(1, 100), member(2, 100), muted];
        assert_eq!(earning_users(&members), BTreeSet::from([1, 2]));
    }
}
