import {Guild, TextChannel, MessageEmbed, Client, Message} from 'discord.js'
import './sqlQueries'
import { Points } from './points';
import { guildId, scoreChannelId } from './const';
const c = (e:string)=>{console.error(e);debugger;}
export class MessageScores{
    points:Points
    guild:Guild | undefined
    channel:TextChannel | undefined
    constructor(client:Client, points:Points){
        this.points = points
        let guild = client.guilds.cache.find(e=>e.id == guildId)
        if(!guild){
            console.log("guild not found")
            process.exit()
        }
        this.guild = guild
        this.channel = this.guild.channels.cache.find(e => e.id == scoreChannelId) as unknown as TextChannel;
        this.init(client)
    }
    async init(client:Client){
        // sqlQueries = await sqlQueries.init()
        //deleting old messages
        this.channel!.messages.cache//then(messages=>messages.array().forEach(m=>m.delete().catch(c))).catch(c)
        this.setupInterval()
    }

    async setupInterval(){
        var message:Message | null;
        setInterval(async ()=>{
            var content = await this.makeEmbed()
            if(message)
                message.edit(content).catch(()=>{message = null; return})
            else
                message = await this.channel!.send(content)
        }, 3000)
    }
    async makeEmbed(){
        let scores = await this.points.exportPoints()
        scores.forEach(e=>e.Points = e.Points/60)
        scores.sort((a, b) => b.Points - a.Points)
        scores.forEach(e => {
            let a = e.User;
            let user = this.guild!.members.cache.find(e=>e.id==a)
            if(!user)return
            let isCo = Points.testCo(user.voice)
            if(isCo) e.Name = "__" + ((user.nickname || user.user.username) as string).slice(0,18) + "__";
            else e.Name = user.nickname || user.user.username
            e.Name = e.Name.slice(0,22)
        });
        let embed = new MessageEmbed()
          .setAuthor(
            "Helibot",
            "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
          )
          .setTitle("http://77.151.84.172:2832/")//https://murmuring-dawn-90139.herokuapp.com (pour tous les scores)")
          .setURL("http://77.151.84.172:2832/")
          .setColor(0)
          .setDescription("scores des 15 premiers :")
          .addField("\u200b","\u200b",true);
        embed.addField("**Top #1:**"+ scores[0].Name + ":", "__**" + Math.round(scores[0].Points) + "**__", true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #2**" + scores[1].Name + ":","**" + Math.round(scores[1].Points) + "**",true)
        embed.addField("\u200b","\u200b",true)
        embed.addField("**Top #3**" + scores[2].Name + ":","**" + Math.round(scores[2].Points) + "**",true)
        for(let i=3;i<15;i++){
            if(scores[i])
                embed.addField("**#"+(i+1)+"**" + scores[i].Name + ":","**" + Math.round(scores[i].Points) + "**",true)
        }
        // embed.setFooter(roles.toString())
        return embed;
    }
}
