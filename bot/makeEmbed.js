const discord = require("discord.js")
const testpl = require("./testpl")
module.exports = function (members) {
  members.sort((a, b) => b.score - a.score)
  members.forEach(e => {
    if(testpl(e)) e.name = "__" + (e.user.nickname || e.user.user.username) + "__";
    else e.name = e.user.nickname || e.user.user.username
    e.name = e.name.slice(0,22)
  });
  let embed = new discord.RichEmbed()
    .setAuthor(
      "Helibot",
      "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
    )
    .setTitle("site en cours")//https://murmuring-dawn-90139.herokuapp.com (pour tous les scores)")
    //.setURL("site en cours")//https://murmuring-dawn-90139.herokuapp.com")
    .setColor(0)
    .setDescription("scores des 15 premiers :")
    .addBlankField(true);
  embed.addField("**Top #1:**"+ members[0].name + ":", "__**" + Math.round(members[0].score) + "**__", true)
  embed.addBlankField(true)
  embed.addField("**Top #2**" + members[1].name + ":","**" + Math.round(members[1].score) + "**",true)
  embed.addBlankField(true)
  embed.addField("**Top #3**" + members[2].name + ":","**" + Math.round(members[2].score) + "**",true)
  for(let i=3;i<15;i++){
    embed.addField("**#"+(i+1)+"**" + members[i].name + ":","**" + Math.round(members[i].score) + "**",true)
  }
  embed.addBlankField(true)
  embed.addField("**#15**" + members[15].name + ":","**" + Math.round(members[15].score) + "**",true)
  embed.addBlankField(true)
  return embed;
}
