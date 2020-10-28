require("./bot/vocaux")
require("./bot/mouche")
const roles = require("./bot/roles"),
  testpl = require("./bot/testpl"), 
  save = require("./bot/save"),
  makeEmbed = require("./bot/makeEmbed"),
  logs = require("./bot/logs"),
  start = require("./bot/start"),
  discord = require("discord.js"),
  express = require("./bot/express"),
  poll = require("./bot/poll");
const shell = require("shelljs")

var scoreMessage, scoreChannel, logsChannel;
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
var guild;
var members = [];
 
var up = function(client, guild,fs,fetch) {
  members.forEach(memb => { 
    if (testpl(memb)) memb.score += 1 / 60;
  });
};

client.on("ready", async function() {
  console.log("trh")
  guild = client.guilds.cache.find(e => e.id == "532956456492728320");
  scoreChannel = guild.channels.cache
    .array()
    .find(e => e.name === "classement_points");
 members = start.load(guild,__dirname);
  express.init(members, poll, __dirname);
  logsChannel = guild.channels.cache.array().find(e => e.name === "bot_logs");
  logs.init(logsChannel, poll);
  logs.start();
  var pollChannel = guild.channels.cache.find(e => e.name === "sondages");//client.guilds.find(e => e.id == "544953131205918720").channels.find(e => e.name == "sondages"); ////
  poll.init(pollChannel, discord);
  guild.channels.cache
    .find(e => e.name == "classement_points")
    .messages
    .fetch({ limit: 100 })
    .then(e =>
      e.forEach(m => {
        m.delete();
      })
    );
  setInterval(() => up(client, guild,require("fs"),require("node-fetch")), 1000);
  roles.up(members);
  //logs.historique(members)
  setInterval(() => roles.up(members), 20000);
  setInterval(() => save.savefile(members), 60000);
  setInterval(() => {
     save.saveFileHistorique(members,__dirname);
  }, 30000);
  setInterval(() => { 
    logs.historique(members).catch();
  }, 43200000);
  setInterval(()=>sendScores().catch(), 3000)
  save.saveOrdi(members,__dirname)
  setInterval(()=>save.saveOrdi(members),24*60*60*1000) 
  setInterval(()=>{guild.channels.cache.find(e=>e.name=="bothistoriquev2").send(makeEmbed(members))},300000)
  setTimeout(() => {
    express.stopPortAndApp()
  }, 24*60*60*1000);
});

client.on("message", msg => {}); 
client.on("messageReactionAdd", (react, user) => {
  if (react.message.channel.name == "sondages") {
    poll.newReact(react, user);
  }
});
var sendScores = async function() {
  guild.channels.cache
  .find(e => e.name == "classement_points")
  .messages
  .fetch({ limit: 100 })
  .then(async e=>{
    if(e.array().length > 1){
      for(let i=0;i<e.array().length;i++){
        if(e.array()[i].id != scoreMessage.id)await e.array()[i].delete()
      }
    }
    if (!scoreMessage) scoreMessage = await scoreChannel.send(makeEmbed(members));
    else scoreMessage.edit(makeEmbed(members)).catch(()=>scoreChannel.send(makeEmbed(members)))

  })
};
client.login(require("./token"))


process.on("uncaughtException",(err)=>{
  console.error(err)
  express.stopPortAndApp()
})
