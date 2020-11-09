let discord = require("discord.js")
let client = new discord.Client()
client.on("guildMemberRemove",(memb)=>{
    memb.guild.channels.cache.find(e=>e.name=="général").send(memb.nickname + " a pris la mouche (a quitté " + memb.guild.name + ")")
})
client.login(require("./../token"))
