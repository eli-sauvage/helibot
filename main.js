const discord = require("discord.js");
// require("./bot/graphique/chart")
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
client.on("ready", async ()=>{
    console.log("bot pret");
    new (require("./bot/points"))(client)
    new (require("./bot/messageScores"))(client)
    let query = await require("./bot/sqlQueries").init()
    query.fillNULL()
    console.log(client.guilds.cache.find(e=>e.id == '532956456492728320').members.cache.find(e=>e.nickname == "Elic de 3").id)
    console.log(client.guilds.cache.find(e=>e.id == '532956456492728320').members.cache.find(e=>e.nickname == "Elic de 3").user.id)


    
    /////
    // dev(query)
    // setInterval(()=>dev(query), 1000)
})
async function dev(q){
    console.log(await q.getPoints())
}
client.login(require("./token").discord)