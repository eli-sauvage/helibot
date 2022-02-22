import { SqlQueries } from '../sql/sqlQueries'
import { Client as discordClient, GuildMember, VoiceState, Guild } from 'discord.js'

type session = {
    memberID: string,
    start: number,
    name: string
}

export class Points {
    sql: SqlQueries
    client: discordClient
    guild: Guild
    openedSessions: session[]
    operations: { user: string, timeoutCode: NodeJS.Timeout }[]
    constructor(client: discordClient, sql: SqlQueries, guild: Guild) {
        this.sql = sql
        this.client = client
        this.guild = guild
        this.openedSessions = []
        this.operations = []
        this.init()
    }
    async init() {
        let idsInPoints = (await this.sql.getPoints()).map(e => e.User);
        let members = this.guild.members.cache.array()
        let pr: Promise<any>[] = []
        members.forEach((member: GuildMember) => {
            if (Points.testCo(member.voice)) {
                pr.push(this.openSession(member.id, member.user.username))
            }
            if (!idsInPoints.includes(member.id) && !member.user.bot) {
                console.log(`adding ${member.user.username} to the DB`)
                pr.push(this.sql.addToPoints(member.id, member.user.username))
            }
        })
        await Promise.all(pr)
    }
    async onVoiceUpdate(oldState:VoiceState, newState:VoiceState) {
        // console.log(this.operations)
        if (oldState.member == null) return
        if (newState.member == null) return
        if ((oldState.channelID == newState.channelID) &&
            (oldState.mute == newState.mute) &&
            (oldState.deaf == newState.deaf)) return //si pas changement de channel / de mute
        this.operations.forEach(op => {
            if (op.user == oldState.member!.id)
                clearTimeout(op.timeoutCode)
        })//on arrete toutes les oprérations en cours pour la personne (pour eviter le spam dans la db, comme ça il ne reste toujours qu'une opération par personne dans la file)
        this.operations = this.operations.filter(e => e.user != oldState.member!.id)
        this.operations.push({
            user: oldState.member.id, timeoutCode: setTimeout(() => {
                if (oldState.guild.id != this.guild.id) return//mauvaise guild
                if (Points.testCo(newState)) {
                    if (!this.openedSessions.find(e => e.memberID == newState.member!.id))
                        this.openSession(newState.member!.id, newState.member!.user.username)
                } else {
                    let session = this.openedSessions.find(e => e.memberID == oldState.member!.id)
                    if (session)
                        this.closeSession(session)
                }
            }, 1000)
        })
    }
    async openSession(id: string, name: string) {
        let currentPoints = await this.sql.getPoint(id).catch(e => { return 0 })
        this.openedSessions.push({
            memberID: id,
            start: Date.now(),
            name: name,
            // pointsAtStart: currentPoints
        })
        // console.log(`${name} a rejoint`)
    }
    async closeSession(session: session) {
        var duration = Math.round((Date.now() - session.start) / 1000)
        this.sql.newSession(session.memberID, session.name, duration)
        this.openedSessions = this.openedSessions.filter(e => e.memberID != session.memberID)
        // console.log(`${session.name} a quitté`)
    }
    static testCo(voiceState: VoiceState) {
        if (!voiceState.member) return
        if (voiceState.member.user.bot) return false
        if (!voiceState.channel) return false//si pas dans un channel
        if (voiceState.selfMute || voiceState.selfDeaf) return false // si mute/mute casque
        // if(voiceState.channel.members.size == 1 )return false //si tout seul
        if (voiceState.channel.id == (voiceState.guild.afkChannel && voiceState.guild.afkChannel.id)) return false //afk
        // if(voiceState.channel.members.filter(e=>e.user.bot).size == voiceState.channel.members.size - 1) return false //si tout seul avec un/des bots
        return true
    }
    async exportPoints() {
        let Points = await this.sql.getPoints();
        this.openedSessions.forEach(session => {
            let indexToChange = Points.findIndex(e => e.User == session.memberID)
            Points[indexToChange].Points += (Date.now() - session.start) / 1000
        })
        Points.forEach(e => e.Points /= 60)
        return Points
    }
}
