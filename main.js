const discord = require("discord.js");
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});

var promises = [
    require("./bot/sqlQueries").init(),
    new Promise((res, rej)=>{
        client.on("ready", res)
    })
]

Promise.all(promises).then(params=>onReady(params[0]))

async function onReady(query){
    console.log("successfully connected to discord.com and to the database")
    query.fillNULL()
    new (require("./bot/points"))(client)
    new (require("./bot/messageScores"))(client)
}

client.login(require("./token").discord)