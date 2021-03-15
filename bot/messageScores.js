const discord = require("discord.js")
let sqlQueries =  require("./sqlQueries");
const c = (e)=>console.error(e)
class MessageScores{
    constructor(client){this.init(client)}
    async init(client){
        sqlQueries = await sqlQueries.init()
        this.guild = client.guilds.cache.find(e => e.id == "532956456492728320");//544953131205918720
        this.channel = this.guild.channels.cache.find(e=>e.id == "672858838810230799");//800719816536096786
        //deleting old messages
        this.channel.messages.fetch({ limit: 100 }).then(messages=>messages.array().forEach(m=>m.delete().catch(c)))
        this.setupInterval()
    }

    async setupInterval(){
        var message;
        setInterval(async ()=>{
            var content = await this.makeEmbed()
            if(message)
                message.edit(content)
            else
                message = await this.channel.send(content)
        }, 3000)
    }
    async makeEmbed(){
        let members = await sqlQueries.getPoints()
        members.sort((a, b) => b.Point - a.Point)
        members.forEach(e => {
            let a = e.User;
            let user = this.guild.members.cache.find(e=>e.id==a)//???
            let isCo = require("./points").testCo(user.voice)
            if(isCo) e.name = "__" + (user.nickname || user.user.username).substring(0,18) + "__";
            else e.name = user.nickname || user.user.username
            e.name = e.name.slice(0,22)
        });
        let embed = new discord.MessageEmbed()
          .setAuthor(
            "Helibot",
            "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
          )
          .setTitle("http://77.151.84.172:2832/")//https://murmuring-dawn-90139.herokuapp.com (pour tous les scores)")
          .setURL("http://77.151.84.172:2832/")
          .setColor(0)
          .setDescription("scores des 15 premiers :")
          .addField("\u200b","\u200b",true);
        embed.addField("**Top #1:**"+ members[0].name + ":", "__**" + Math.round(members[0].Point) + "**__", true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #2**" + members[1].name + ":","**" + Math.round(members[1].Point) + "**",true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #3**" + members[2].name + ":","**" + Math.round(members[2].Point) + "**",true)
        for(let i=3;i<15;i++){
            if(members[i])
                embed.addField("**#"+(i+1)+"**" + members[i].name + ":","**" + Math.round(members[i].Point) + "**",true)
        }
        // embed.setFooter(roles.toString())
        return embed;
    }
}
module.exports = MessageScores