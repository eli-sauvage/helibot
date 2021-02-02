class Points{
    constructor(client){
        this.init(client)
    }
    async init(client){
        this.client = client
        this.sql = await require("./sqlQueries").init()
        this.client.on("voiceStateUpdate", async (oldState, newState)=>{
            if(oldState.guild.name != "Gros Molec")return
            if(this.testCo(newState)){
                console.log(`${oldState.member.nickname} a rejoint`)
                this.sql.onMemberConnect(newState.member.id)
            }else{
                console.log(`${oldState.member.nickname} a quitté`)
                this.sql.onMemberDisconnect(oldState.member.id)
            }
        })
    }
    testCo(voiceState){
        if(!voiceState.channel) return false//si pas dans un channel
        if(voiceState.selfMute || voiceState.selfDeaf) return false // si mute/mute casque
        if(voiceState.channel.members.size == 1 )return false //si tout seul
        if(voiceState.channel.members.filter(e=>e.user.bot).size == voiceState.channel.members.size - 1) return false //si tout seul avec un/des bots
        return true
    }
}
module.exports = Points;