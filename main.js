const discord = require("discord.js");
require("./bot/graphique/chart")
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
client.on("ready", async ()=>{
    console.log("bot pret");
    new (require("./bot/points"))(client)
    new (require("./bot/messageScores"))(client)




    
    /////
    let query = require("./bot/sqlQueries")
    setInterval(()=>dev(query), 10000)
})
async function dev(q){
    console.log(await q.getPoints())
}
client.login(require("./token").discord)