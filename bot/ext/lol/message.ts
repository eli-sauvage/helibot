import { sql } from "../../sqlconnector"
import {Client, Message, TextChannel} from 'discord.js'
import { guildId } from "../../const"
import { stats } from "./lolStats"

export class LolMessage{
    sqlConn:sql
    channel:TextChannel
    message:Message|undefined
    constructor(sqlConn:sql, client:Client){
        this.sqlConn = sqlConn
        let guild = client.guilds.cache.get(guildId)
        if(!guild){
            console.log("guild not found")
            process.exit()
        }
        this.channel =  guild.channels.cache.get("869264655124672652") as TextChannel
        setInterval(()=>this.main(false), 60*60*1000)
        this.main(false)
    }
    async main(force:boolean){
        let stats:stats =  await require("./lolStats").computeGlobalScore(this.sqlConn)
        let lastSent:string = await this.sqlConn.getSingleVal("SELECT data FROM storedData WHERE name = 'lastLolMessage'") as string
        if(!stats.moyTier || !stats.moyRank || stats.moyleaguePoints==undefined){
            return false
        }else if(force || (lastSent != stats.moyTier.toString() + stats.moyRank.toString() + stats.moyleaguePoints)){
            let embed = new (require("discord.js").MessageEmbed)()
            .setAuthor(
              "Helibot",
              "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
            )
            .setTitle(`__**Le Molec est ${stats.moyTier} ${stats.moyRank}, ${stats.moyleaguePoints}LP**__`)
            .setColor(0)
            .setDescription("Elo des joueurs :")
            for(let player of stats.players.sort((a,b)=>b.tot - a.tot).slice(0, 24)){
                    embed.addField(player.name + ":", `${player.tier} ${player.rank}, ${player.lp}LP`,true)
            }
            // if(message)
                // message.edit(embed)
            // else
            await this.channel.send(embed)
            this.sqlConn.query(`UPDATE storedData SET data = '${stats.moyTier + stats.moyRank + stats.moyleaguePoints}' WHERE name = "lastLolMessage"`)
            return true
        }
    }
}

// const runEveryQuartDHeure = (callbackFn) => {
//     const Hour = 15 * 60 * 1000;
//     const currentDate = new Date();
//     const firstCall =  Hour - ((currentDate.getMinutes()%15) * 60 + currentDate.getSeconds()) * 1000 + currentDate.getMilliseconds();
//     setTimeout(() => {
//       callbackFn();
//       setInterval(callbackFn, Hour);
//     }, firstCall);
//   };