const discord = require("discord.js");
// require("./bot/graphique/chart")
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
client.on("ready", async ()=>{
    console.log("bot pret");
    new (require("./bot/points"))(client)
    new (require("./bot/messageScores"))(client)
    let query = await require("./bot/sqlQueries").init()
    query.fillNULL()



    
    /////
    // dev(query)
    // setInterval(()=>dev(query), 1000)
})
async function dev(q){
    console.log(await q.getPoints())
}
client.login(require("./token").discord)