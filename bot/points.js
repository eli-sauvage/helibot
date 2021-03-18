class Points{
    constructor(client){
        this.init(client)
    }
    async init(client){
        this.client = client
        this.sql = await require("./sqlQueries").init()
        let currentPoints = await this.sql.getPoints()
        let members = this.client.guilds.cache.find(e=>e.id == require("./const").guildId).members.cache.array()
        for(var i=0;i<members.length;i++){
            if(!currentPoints.map(e=>e.User).includes(members[i].id) && !members[i].user.bot){
                console.log("adding "+members[i].user.username);
                await this.sql.addMember(members[i].id, (members[i].nickname || members[i].user.username))
            }
            if(Points.testCo(members[i].voice))this.sql.onMemberConnect(members[i].id, (members[i].nickname || members[i].user.username))//ouvre toutes les co en cours
        }
        this.client.on("voiceStateUpdate", async (oldState, newState)=>{
            if(oldState.guild.id != require("./const").guildId )return//mauvaise guild
            if((oldState.channel.id == newState.channel.id) &&
                (oldState.mute == newState.mute)&&
                (oldState.deaf == newState.deaf))return //si pas changement de channel / de mute
            if(Points.testCo(newState)){
                console.log(`${oldState.member.user.username} a rejoint`)
                this.sql.onMemberConnect(newState.member.id, (newState.member.nickname || newState.member.user.username))
            }else{
                console.log(`${oldState.member.user.username} a quitté`)
                this.sql.onMemberDisconnect(oldState.member.id)
            }
        })
    }
    static testCo(voiceState){
        if(voiceState.member.user.bot) return false
        if(!voiceState.channel) return false//si pas dans un channel
        if(voiceState.selfMute || voiceState.selfDeaf) return false // si mute/mute casque
        // if(voiceState.channel.members.size == 1 )return false //si tout seul
        if(voiceState.channel.id == voiceState.guild.afkChannel.id) return false //afk
        // if(voiceState.channel.members.filter(e=>e.user.bot).size == voiceState.channel.members.size - 1) return false //si tout seul avec un/des bots
        return true
    }
}
module.exports = Points;