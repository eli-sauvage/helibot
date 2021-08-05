import { TextChannel , MessageEmbed} from "discord.js";

export class Poll{
    channel:TextChannel
    constructor(channel:TextChannel){
        this.channel = channel
    }
    newpoll(question:string, responses:string[], mention:boolean){
        var msg = new MessageEmbed()
            .setColor("#0099ff")
            .setTitle("Sondage")
            //.setURL("site en cours")//https://murmuring-dawn-90139.herokuapp.com/poll")
            .setAuthor("helibot", "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg")
            .setDescription(question)
            .setTimestamp()
            .setFooter("http://helibot.biz.uz")
        for (let i = 0; i < responses.slice(0,24).length; i++)
            msg.addField(responses[i], reacts[i], true);
        if (mention)
            this.channel.send("@everyone, " + question).then(e => e.delete())
        this.channel.send(msg).then(async function (m) {
            let pr = []
            for (var i = 0; i < responses.slice(0, 24).length; i++)
                pr.push(m.react(reacts[i]))
            Promise.all(pr)
        }).catch();
    }
}
const reacts = [
    "1️⃣",
    "2️⃣",
    "3️⃣",
    "4️⃣",
    "5️⃣",
    "6️⃣",
    "7️⃣",
    "8️⃣",
    "9️⃣",
    "🔟",
    "🇦",
    "🇧",
    "🇨",
    "🇩",
    "🇪",
    "🇫",
    "🇬",
    "🇭",
    "🇮",
    "🇯",
    "🇰",
    "🇱",
    "🇲",
    "🇳",
    "🇴"
];