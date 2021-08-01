import {Intents, Client} from 'discord.js'
import { MessageScores } from './bot/messageScores';
import { SqlQueries } from './bot/sqlQueries';
import {Points} from './bot/points'
import {LolMessage} from './bot/ext/lol/message' ;
import {Ws} from './bot/ext/lol/ws'
import {discord as discordToken} from './token'
console.log("starting")


var client = new Client({ws:{intents:Intents.ALL}});

var promises = [
    new SqlQueries().init(),
    new Promise((res, rej)=>{
        client.on("ready", ()=>res(null))
    })
]

// Promise.all<SqlQueries, void>(promises).then((params:any[])=>onReady(params[0]))

async function start(){
    let t = await Promise.all(promises)
    let query = (t as [SqlQueries, void])[0]
    
    console.log("successfully connected to discord.com and the database")
    let points = new Points(client, query)
    new MessageScores(client, points)

    let msg = new LolMessage(query.sql, client)
    // lolMessageInit(query.sql, client)
    Ws(query.sql, msg)
}
start()
client.login(discordToken)
// function mem(){
// // let memory = process.memoryUsage()
// //     for(let key in memory)console.log(`${key} ${Math.round(memory[key] / 1024 / 1024 * 100) / 100} MB`)
// }
// mem()
// setInterval(mem, 10*60*1000)