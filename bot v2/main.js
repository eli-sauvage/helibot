let SqlCo = require("../bot/sql/sqlconnector")
const discord = require("discord.js");
const { sql } = require("../token");
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
let sqlCo;
let guild;
client.on("ready", async ()=>{
    sqlCo = await new SqlCo("helibot");
    console.log("bot pret");
    guild = client.guilds.cache.find(e => e.id == "532956456492728320");
})
client.on("voiceStateUpdate", async (oldState, newState)=>{
    if(newState.channel && !newState.mute){
        console.log(`${oldState.member.nickname} a rejoint`)
        sqlCo.query(`INSERT INTO Session (User, ConnectionTime, Point) VALUES (${oldState.member.id}, now(), (SELECT MAX(Point) WHERE User = ${oldState.member.id}));`)
    }else{
        let sessionID = await sqlCo.getSingleVal(`SELECT max(id) FROM Session Where User = ${oldState.member.id}`)
        console.log(`${oldState.member.nickname} a quitté`)
        sqlCo.query(`UPDATE Session SET DisconnectionTime = now(), Point = (Select now() - (SELECT max(ConnectionTime) FROM Session WHERE User = ${oldState.member.id}) + (SELECT MAX(Point) FROM Session Where User = ${oldState.member.id} ) WHERE id = ${sessionID}) WHERE  id = ${sessionID};`)
    }
})
client.login(require("../token").discord)