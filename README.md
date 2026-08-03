# helibot

A Discord bot that rewards time spent together in voice channels: one minute in a voice
channel with someone else earns one point, and points move members up a ladder of roles.

## What it does

- **Voice time.** A member earns while they are in a voice channel with **at least one
  other eligible member**. Bots, and anyone muted or deafened (by themselves or by the
  server), are ignored — and do not count towards the two needed to earn.
- **Leaderboard.** One self-maintaining message per guild, in the configured channel,
  refreshed on a timer. Shows the top 15 with connected members underlined. Two buttons
  sit under it: **refresh**, which re-renders the board and re-runs the role and username
  sweeps, and **afficher tous les scores**, which replies privately with every score as a
  `scores.txt` attachment. Both answer Discord inside its three-second interaction
  deadline and do the work afterwards.
- **Role ladder.** Every member holds exactly one role: the highest tier their score has
  reached. Missing roles are created when the bot joins a guild.
- **Usernames.** Stored names are kept in step with nicknames, and accounts that no
  longer exist are tagged rather than shown as `deleted_user_…`.

Points are stored in **seconds** and displayed in **minutes**. Role thresholds in
`helibot.toml` are in **minutes**.

## Configuration

Two sources, merged in that order:

- `helibot.toml` — the points channel, the refresh intervals and the role ladder.
- Environment variables prefixed with `HELIBOT_`, optionally via a `.env` file. The
  prefix is required: `HELIBOT_DATABASE_URL`, `HELIBOT_DISCORD_TOKEN`. Copy
  `.env.template` to `.env` to start.

The bot reads both files from its working directory, which is `/` in the container;
compose mounts them there.

`RUST_LOG` controls logging (`info` by default, `info,helibot=debug` to follow the
reconciliation in detail).

### Discord permissions

The bot connects with the non-privileged `GUILDS`, `GUILD_VOICE_STATES` and
`GUILD_MESSAGES` intents only — the last one delivers the deletion events that tell it
the board is gone, and does not include message content, which is the privileged part.
Adding a privileged intent that is not enabled in the developer portal makes the gateway
refuse the connection outright, so member lists are fetched over HTTP instead. It needs
**Manage Roles** for the ladder, and **Manage Messages** in the points channel so it can
clear it.

The points channel is treated as dedicated to the board: whenever a new board is posted,
everything already in the channel is deleted first.

## Running it

```sh
cp .env.template .env      # then fill in HELIBOT_DISCORD_TOKEN
docker compose up --build
```

Migrations run automatically at startup.

## Development

```sh
export DATABASE_URL="mysql://helibot:helibot@localhost:3306/helibot"
cargo sqlx migrate run
cargo test          # unit tests need nothing; tests/db.rs needs the database above
cargo clippy --all-targets
```

Queries are checked at compile time against the schema. After changing one, regenerate
the offline metadata the Docker build relies on:

```sh
cargo sqlx prepare   # requires cargo install sqlx-cli --no-default-features --features mysql,rustls
```

Reaching the production database through the deployment host:

```sh
gcloud compute ssh --zone "us-central1-c" "elicolh@instance-1" --project "test-micro-1" -- -NL 3307:localhost:3307
```

## Layout

| Path | What lives there |
| --- | --- |
| `src/domain/` | The rules — who earns, which rank, how the board reads. Pure and unit-tested. |
| `src/db/` | Every query. All time arithmetic happens here, in SQL. |
| `src/discord/` | The only code that talks to the gateway. |
| `src/tasks.rs` | The background loops, and the supervisor that restarts them. |
| `src/state.rs` | Shared state, held behind one `Arc`. |

### Schema

`Points` holds the running total per member per guild, `ActiveSessions` the open
sessions, `SessionHistory` the closed ones. `Points.score_historique` carries scores
imported from the pre-Rust bot and is not currently displayed anywhere.

Voice time is credited **incrementally**: every `credit_interval_secs`, open sessions
have their elapsed seconds added to `Points` and their `last_credited_at` watermark
advanced. A crash therefore costs at most one interval, and a clean shutdown costs
nothing. Sessions left behind by a previous run are archived at their watermark on the
next startup.

In `SessionHistory`, `session_seconds` is what that session was worth and
`total_points_after` is the member's total once it closed.
