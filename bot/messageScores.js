const discord = require("discord.js")
const c = (e)=>console.error(e)
class MessageScores{
    constructor(client){
        this.guild = client.guilds.cache.find(e => e.id == "544953131205918720");//532956456492728320
        this.channel = this.guild.channels.cache.find(e=>e.id == "800719816536096786");
        this.deleteMessages()
        this.setupInterval()
    }
    deleteMessages(){
        this.channel.messages.fetch({ limit: 100 })
            .then(messages => messages.forEach(msg => msg.delete().catch(c)))
    }
    setupInterval(){
        let msg;
        let everyone = this.guild.roles.cache.find(e=>e.name == "@everyone")
        this.channel.overwritePermissions([{id:everyone, allow:['VIEW_CHANNEL']}]).catch(c)
        setInterval(async ()=>{
            if(msg)msg.delete().catch(c)
            await this.channel.overwritePermissions([{id:everyone, deny:['VIEW_CHANNEL']}]).catch(c)
            msg = await this.channel.send(new discord.MessageEmbed().attachFiles([__dirname + '/graphique/chart.png']).setImage('attachment://chart.png')).catch(c)
            await this.channel.overwritePermissions([{id:everyone, allow:['VIEW_CHANNEL']}]).catch(c)
        }, 10000)
    }
    makeEmbed(){

    }
}
module.exports = MessageScores