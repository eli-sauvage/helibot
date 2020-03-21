const makeEmbed = require("./makeEmbed");
const save = require("./save");
const fs = require("fs");
class Logs {
  constructor() {
    this.logChannel;
  }
  init(channel) {
    this.logChannel = channel;
  }
  start() {
    if (this.logChannel)
      this.logChannel.send("démarré " + new Date().toLocaleString());
  }
  saved() {
    if (this.logChannel)
      this.logChannel.send("scores enregistrés" + new Date().toLocaleString());
  }
  async historique(members) {
    if (!this.logChannel) return;
    //this.logChannel.send(makeEmbed(members));
    this.logChannel.send(new Date().toLocaleString());
    this.logChannel.send(JSON.stringify(save.json(members, true)));
  }
  erreur(err) {
    if (this.logChannel) this.logChannel.send(err);
  }
  role(memb, role) {
    if (this.logChannel)
      this.logChannel.send(
        "role de " + memb.user.user.username + " mis a jour vers " + role.name
      );
  }
}
module.exports = new Logs();
