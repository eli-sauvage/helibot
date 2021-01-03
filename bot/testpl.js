module.exports = function(memb) {
  if (memb.user.voiceChannel == undefined) return false;
  if (
    memb.user.voiceChannel.members.array().length >= 2 && //au moins 2 personnes ds channel
    !memb.user.deaf &&
    !memb.user.mute &&
    !(
      (memb.user.voiceChannel.members
        .array()
        .map(e => e.user.bot)
        .includes(true) &&
        memb.user.voiceChannel.members.array().length == 2) ||
      memb.user.voiceChannel.members.array().length == 1
    ) //pas tout seul avec le bot
  )
    return true;
};
