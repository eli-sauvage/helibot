use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::all::{ChannelId, GuildId, MessageId, RoleId};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};

use crate::config::Config;
use crate::db::Db;
use crate::domain::tiers::Threshold;

/// A configured tier resolved to a real role in one guild.
#[derive(Debug, Clone)]
pub struct GuildRole {
    pub role_id: RoleId,
    pub name: String,
    pub threshold_seconds: u64,
}

impl Threshold for GuildRole {
    fn threshold_seconds(&self) -> u64 {
        self.threshold_seconds
    }
}

/// The leaderboard message this process maintains in a guild.
#[derive(Debug, Clone)]
pub struct PostedBoard {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub fields: Vec<(String, String, bool)>,
}

/// Everything the handlers need, held behind one `Arc`.
///
/// The previous version kept this in serenity's `TypeMap`, where every read was an
/// `unwrap()` on a key that might not have been inserted yet — and during startup, some
/// of them genuinely were not.
pub struct AppState {
    pub db: Db,
    pub config: Config,
    boards: Mutex<HashMap<GuildId, PostedBoard>>,
    channels: Mutex<HashMap<GuildId, ChannelId>>,
    roles: RwLock<HashMap<GuildId, Vec<GuildRole>>>,
    members: RwLock<HashMap<GuildId, HashSet<u64>>>,
    sweeps: Mutex<HashMap<(GuildId, &'static str), Arc<Semaphore>>>,
}

impl AppState {
    pub fn new(db: Db, config: Config) -> Self {
        Self {
            db,
            config,
            boards: Mutex::default(),
            channels: Mutex::default(),
            roles: RwLock::default(),
            members: RwLock::default(),
            sweeps: Mutex::default(),
        }
    }

    pub async fn board(&self, guild_id: GuildId) -> Option<PostedBoard> {
        self.boards.lock().await.get(&guild_id).cloned()
    }

    pub async fn set_board(&self, guild_id: GuildId, board: PostedBoard) {
        self.boards.lock().await.insert(guild_id, board);
    }

    pub async fn forget_board(&self, guild_id: GuildId) {
        self.boards.lock().await.remove(&guild_id);
    }

    pub async fn cached_channel(&self, guild_id: GuildId) -> Option<ChannelId> {
        self.channels.lock().await.get(&guild_id).copied()
    }

    pub async fn cache_channel(&self, guild_id: GuildId, channel_id: ChannelId) {
        self.channels.lock().await.insert(guild_id, channel_id);
    }

    pub async fn cached_roles(&self, guild_id: GuildId) -> Option<Vec<GuildRole>> {
        self.roles.read().await.get(&guild_id).cloned()
    }

    pub async fn cache_roles(&self, guild_id: GuildId, roles: Vec<GuildRole>) {
        self.roles.write().await.insert(guild_id, roles);
    }

    /// Ids of members currently in the guild, if a member list has been fetched.
    /// `None` means "unknown", which callers treat as "do not filter".
    pub async fn cached_members(&self, guild_id: GuildId) -> Option<HashSet<u64>> {
        self.members.read().await.get(&guild_id).cloned()
    }

    pub async fn cache_members(&self, guild_id: GuildId, members: HashSet<u64>) {
        self.members.write().await.insert(guild_id, members);
    }

    /// Guards a long per-guild sweep. `None` means one is already running and this call
    /// should simply be skipped.
    pub async fn try_sweep(
        &self,
        kind: &'static str,
        guild_id: GuildId,
    ) -> Option<OwnedSemaphorePermit> {
        self.semaphore(kind, guild_id)
            .await
            .try_acquire_owned()
            .ok()
    }

    /// Serialises work that must not be skipped, only queued — voice reconciliation,
    /// where two events arriving together would otherwise race on the same sessions.
    pub async fn sweep(&self, kind: &'static str, guild_id: GuildId) -> OwnedSemaphorePermit {
        self.semaphore(kind, guild_id)
            .await
            .acquire_owned()
            .await
            .expect("sweep semaphores are never closed")
    }

    async fn semaphore(&self, kind: &'static str, guild_id: GuildId) -> Arc<Semaphore> {
        self.sweeps
            .lock()
            .await
            .entry((guild_id, kind))
            .or_insert_with(|| Arc::new(Semaphore::new(1)))
            .clone()
    }
}
