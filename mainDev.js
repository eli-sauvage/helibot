const discord = require("discord.js");
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});

var promises = [
    new (require("./bot/sqlQueries"))().init(),
    new Promise((res, rej)=>{
        client.on("ready", res)
    }),
]

Promise.all(promises).then(params=>onReady(params[0]))


async function onReady(query){
    console.log("successfully connected to discord.com and the database")
    let points = new (require("./bot/points"))(client, query)
    new (require("./bot/messageScores"))(client, points)

    
    let messageLol = require("./bot/ext/lol/message")
    messageLol.init(query.sql, client)
    require("./bot/ext/lol/ws")(query.sql, messageLol)
}

client.login(require("./token").discord)