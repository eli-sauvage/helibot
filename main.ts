import {Intents, Client} from 'discord.js'
import { MessageScores } from './bot/points/messageScores';
import { SqlQueries } from './bot/sql/sqlQueries';
import {Points} from './bot/points/points'
import {LolMessage} from './bot/lol/message' ;
import {Ws} from './bot/lol/ws'
import {discord as discordToken} from './token'
import { Consts } from './bot/consts';
import { Website } from './bot/website/webSite';
import { Poll } from './bot/website/poll';
import { Roles } from './bot/points/roles';
import { InitVocaux } from './bot/vocaux';

console.log("starting")


var client = new Client({ws:{intents:Intents.ALL}});


// Promise.all<SqlQueries, void>(promises).then((params:any[])=>onReady(params[0]))

async function start(){
    //starting sql and discord
    var promises = [
        new SqlQueries().init(),
        new Promise((res, rej) => {
            client.on("ready", () => res(null))
        })
    ]
    let t = await Promise.all(promises)
    let query = (t as [SqlQueries, void])[0]
    
    console.log("successfully connected to discord.com and the database")
    //wait 5 sec before loggin to ws (2nd client, 5sec between connections)
    await new Promise<void>(res=>setTimeout(res, 5000))
    // starting const
    let consts = new Consts(client)
    //starting points
    let points = new Points(client, query, consts.guild)
    //starting points message
    new MessageScores(client, points, consts.guild, consts.scoreChannel)
    //starting roles manager
    new Roles(consts.guild, points)
    //starting lol message
    //let msg = new LolMessage(query.sql, client, consts.lolChannel)
    //starting poll
    let poll = new Poll(consts.pollChannel)
    //starting website
    new Website(poll, points)
    //starting websocket
    //    Ws(query.sql, msg)
    //starting vocaux
    let voc = new InitVocaux(consts.guild, consts.vocCategory, consts.generalVoc)



    //setting up events
    client.on("voiceStateUpdate", (oldState, newState)=>{
        points.onVoiceUpdate(oldState, newState)
        voc.testChannels()
    })
    client.on("guildMemberRemove", (memb)=>{
        if(memb.user)consts.generalChannel.send(memb.user.username + " a pris la mouche (a quitté " + memb.guild.name + ")")
    })
}
start()
client.login(discordToken)
// function mem(){
// // let memory = process.memoryUsage()
// //     for(let key in memory)console.log(`${key} ${Math.round(memory[key] / 1024 / 1024 * 100) / 100} MB`)
// }
// mem()
// setInterval(mem, 10*60*1000)
