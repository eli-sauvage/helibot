const discord = require("discord.js");
// require("./bot/graphique/chart")
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});

async function onReady(query){
    console.log("successfully connected to discord.com and to the database")
    query.fillNULL()
    new (require("./bot/points"))(client)
    new (require("./bot/messageScores"))(client)
}
var promises = [
    require("./bot/sqlQueries").init(),
    new Promise((res, rej)=>{
        client.on("ready", res)
    })
]
Promise.all(promises).then(params=>onReady(params[0]))

// client.on("ready", async ()=>{
    // console.log("bot pret");
    // new (require("./bot/points"))(client)
    // new (require("./bot/messageScores"))(client)
    // let query = await require("./bot/sqlQueries").init()
    // query.fillNULL()
    // let sql = new require("./bot/sqlconnector")("helibot")
    
    // console.log(client.guilds.cache.find(e=>e.id == '532956456492728320').members.cache.find(e=>e.nickname == "Elic de 3").id)
    // console.log(client.guilds.cache.find(e=>e.id == '532956456492728320').members.cache.find(e=>e.nickname == "Elic de 3").user.id)


    
    /////
    // dev(query)
    // dev()
    // setInterval(()=>dev(query), 1000)
// })
async function dev(q){

}
client.login(require("./token").discord)