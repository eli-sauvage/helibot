use serenity::all::{Context, GuildId, UserId};

pub mod handler;
pub mod leaderboard;
pub mod roles;
pub mod usernames;
pub mod voice;

/// Best available name for a member, preferring the cache so the common path costs no
/// HTTP request.
pub async fn display_name(ctx: &Context, guild_id: GuildId, user_id: UserId) -> Option<String> {
    // Scoped so the cache guard is dropped before any await.
    let cached = ctx.cache.guild(guild_id).and_then(|guild| {
        guild
            .members
            .get(&user_id)
            .map(|member| member.display_name().to_owned())
    });

    match cached {
        Some(name) => Some(name),
        None => user_id.to_user(ctx).await.ok().map(|user| user.name),
    }
}
