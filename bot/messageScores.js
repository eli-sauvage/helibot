const discord = require("discord.js")
let sqlQueries =  require("./sqlQueries");
const c = (e)=>{console.error(e);debugger;}
class MessageScores{
    constructor(client, points){
        this.points = points
        this.init(client)
    }
    async init(client){
        // sqlQueries = await sqlQueries.init()
        this.guild = client.guilds.cache.find(e=>e.id == require("./const").guildId)
        this.channel = this.guild.channels.cache.find(e=>e.id == require("./const").scoreChannelId);
        //deleting old messages
        this.channel.messages.fetch({ limit: 100 }).then(messages=>messages.array().forEach(m=>m.delete().catch(c))).catch(c)
        this.setupInterval()
    }

    async setupInterval(){
        var message;
        setInterval(async ()=>{
            var content = await this.makeEmbed()
            if(message)
                message.edit(content).catch(()=>{message = null; return})
            else
                message = await this.channel.send(content)
        }, 3000)
    }
    async makeEmbed(){
        let scores = await this.points.exportPoints()
        scores.forEach(e=>e.Points = e.Points/60)
        scores.sort((a, b) => b.Points - a.Points)
        scores.forEach(e => {
            let a = e.User;
            let user = this.guild.members.cache.find(e=>e.id==a)
            if(!user)return
            let isCo = this.points.constructor.testCo(user.voice)
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
        embed.addField("**Top #1:**"+ scores[0].name + ":", "__**" + Math.round(scores[0].Points) + "**__", true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #2**" + scores[1].name + ":","**" + Math.round(scores[1].Points) + "**",true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #3**" + scores[2].name + ":","**" + Math.round(scores[2].Points) + "**",true)
        for(let i=3;i<15;i++){
            if(scores[i])
                embed.addField("**#"+(i+1)+"**" + scores[i].name + ":","**" + Math.round(scores[i].Points) + "**",true)
        }
        // embed.setFooter(roles.toString())
        return embed;
    }
}
module.exports = MessageScores