module.exports = function(memb, guild) {
  if (memb.user.voice.channel == undefined) return false;
  if (
    memb.user.voice.channel.members.array().length >= 2 && //au moins 2 personnes ds channel
    !memb.user.voice.deaf &&
    !memb.user.voice.mute &&
    guild.afkChannel.id != memb.user.voice.channelID &&
    !(
      (memb.user.voice.channel.members
        .array()
        .map(e => e.user.bot)
        .includes(true) &&
        memb.user.voice.channel.members.array().length == 2) ||
      memb.user.voice.channel.members.array().length == 1
    ) //pas tout seul avec le bot
  )
    return true;
};
