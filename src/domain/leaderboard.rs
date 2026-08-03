/// One line of the board, already sorted by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub username: String,
    pub points_seconds: u64,
    pub connected: bool,
}

/// How many members the embed shows.
pub const TOP_N: usize = 15;

/// Discord refuses embeds with more than 25 fields.
pub const MAX_FIELDS: usize = 25;

/// Embed fields as `(name, value, inline)`.
///
/// The previous version intended to show the top 15 but called `Vec::shrink_to(15)`,
/// which only touches capacity, so the cut never happened and the board actually ran to
/// 22 entries — the point at which the 25-field cap bit. Here the cut is real.
pub fn fields(rows: &[Row]) -> Vec<(String, String, bool)> {
    let mut fields: Vec<(String, String)> = rows
        .iter()
        .take(TOP_N)
        .enumerate()
        .map(|(index, row)| {
            let rank = index + 1;
            let name = if row.connected {
                format!("#{rank} __{}__", row.username)
            } else {
                format!("#{rank} {}", row.username)
            };
            (name, minutes(row.points_seconds))
        })
        .collect();

    // Blank spacers ahead of the first three entries, reproducing the podium layout of
    // the original board. Inserted back to front so the earlier indices stay valid.
    for index in [2, 1, 0] {
        if fields.len() > index {
            fields.insert(index, (String::new(), String::new()));
        }
    }

    fields
        .into_iter()
        .take(MAX_FIELDS)
        .map(|(name, value)| (name, value, true))
        .collect()
}

/// Points are stored in seconds and displayed in minutes.
fn minutes(points_seconds: u64) -> String {
    (points_seconds / 60).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(count: usize) -> Vec<Row> {
        (0..count)
            .map(|index| Row {
                username: format!("user{index}"),
                points_seconds: (count - index) as u64 * 60,
                connected: false,
            })
            .collect()
    }

    #[test]
    fn shows_at_most_fifteen_members() {
        let fields = fields(&rows(40));
        let named = fields
            .iter()
            .filter(|(name, _, _)| !name.is_empty())
            .count();
        assert_eq!(named, TOP_N);
    }

    #[test]
    fn never_exceeds_the_discord_field_cap() {
        assert!(fields(&rows(40)).len() <= MAX_FIELDS);
    }

    #[test]
    fn seconds_are_rendered_as_minutes() {
        let row = Row {
            username: "a".to_owned(),
            points_seconds: 3_599,
            connected: false,
        };
        let fields = fields(&[row]);
        let value = &fields
            .iter()
            .find(|(name, _, _)| !name.is_empty())
            .unwrap()
            .1;
        assert_eq!(value, "59");
    }

    #[test]
    fn connected_members_are_underlined_and_ranked() {
        let row = Row {
            username: "someone".to_owned(),
            points_seconds: 60,
            connected: true,
        };
        let fields = fields(&[row]);
        let name = &fields
            .iter()
            .find(|(name, _, _)| !name.is_empty())
            .unwrap()
            .0;
        assert_eq!(name, "#1 __someone__");
    }

    #[test]
    fn an_empty_board_produces_no_fields() {
        assert!(fields(&[]).is_empty());
    }
}
