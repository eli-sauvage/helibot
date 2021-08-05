import { sql } from "../sql/sqlconnector"
import { Client, Message, TextChannel, MessageEmbed, Guild } from 'discord.js'
import { stats, computeGlobalScore } from "./lolStats"
import fetch from "node-fetch"
import { discord as discordToken } from "../../token"

export class LolMessage {
    sqlConn: sql
    channel: TextChannel
    client: Client
    messageID: string | void
    constructor(sqlConn: sql, client: Client, lolChannel: TextChannel) {
        this.sqlConn = sqlConn
        this.client = client
        this.channel = lolChannel
        this.messageID = void
            this.setupInterval()
    }
    async setupInterval() {
        await this.clearMessages(true)
        setInterval(() => this.main(false), 60 * 15 * 1000)
    }
    async clearMessages(onlyBot: boolean) {
        let messages
        if (!onlyBot) {
            while ((messages = await this.channel.messages.fetch({ limit: 100 })).size != 0) {
                let pr = []
                for (let message of messages.array()) {
                    pr.push(message.delete().catch(console.log))
                }
                await Promise.all(pr)
            }
        } else {
            while ((messages = (await this.channel.messages.fetch({ limit: 100 })).filter(m => m.author.bot)).size != 0) {
                let pr = []
                for (let message of messages.array()) {
                    pr.push(message.delete().catch(console.log))
                }
                await Promise.all(pr)
            }
        }
        this.main(true).catch(console.log)
    }
    async main(force: boolean) {
        let stats: stats | undefined = await computeGlobalScore(this.sqlConn)
        if (stats == undefined) return false
        let lastSent: string = await this.sqlConn.getSingleVal("SELECT data FROM storedData WHERE name = 'lastLolMessage'") as string
        if (!stats.moyTier || !stats.moyRank || stats.moyleaguePoints == undefined) {
            return false
        } else if (force || (lastSent != stats.moyTier.toString() + stats.moyRank.toString() + stats.moyleaguePoints)) {
            let fields = []
            for (let player of stats.players.sort((a, b) => b.tot - a.tot).slice(0, 24)) {
                fields.push({
                    name: player.name + ":",
                    value:`${player.tier} ${player.rank}, ${player.lp}LP`,
                    inline:true
                })
            }
            let msg = JSON.stringify({
                embeds: [{
                    title: `__**Le Molec est ${stats.moyTier} ${stats.moyRank}, ${stats.moyleaguePoints}LP**__`,
                    color: 0,
                    description: "Elo des joueurs :",
                    fields: fields,
                    author: {
                        name: "Helibot",
                        icon_url: "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
                    },
                }],
                components: [
                    {
                        type: 1,
                        components: [
                            {
                                type: 2,
                                style: 1,
                                label: "refresh",
                                custom_id: "refresh_lol_score"
                            },
                            {
                                type: 2,
                                style: 1,
                                label: "clean channel",
                                custom_id: "clear_lol_channel"
                            }
                        ]
                    }
                ]
            })
            let msgFetch, edit:boolean
            if (this.messageID) msgFetch = await this.channel.messages.fetch(this.messageID).catch(() => { return false })
            else msgFetch = true
            if (this.messageID && msgFetch)
                edit = true
            else
                edit = false

            let res = await fetch(`https://discord.com/api/v8/channels/${this.channel.id}/messages${edit?'/' + this.messageID:''}`, {
                method: edit?"PATCH":"POST",
                headers: {
                    "Authorization": `Bot ${discordToken}`,
                    'Content-Type': "application/json"
                },
                body:msg
            }).catch(console.error)
            if(res){
                let body = await res.json()
                if(body)this.messageID = body.id
            }
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