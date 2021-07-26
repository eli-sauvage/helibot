let sqlConn, channel, message
function init(_sqlConn, client){
    sqlConn = _sqlConn
    channel =  client.guilds.cache.get(require("../../const").guildId).channels.cache.get("869264655124672652")//")672858838810230799
    // runEveryQuartDHeure(()=>console.log("ui"))
    setInterval(()=>main(sqlConn, channel), 60*60*1000)
    main(sqlConn, channel)
}
async function main(){
    let IDs = await sqlConn.getColumn("SELECT lolID from LolIDs")
    let stats =  await require("./lolStats").computeGlobalScore(IDs, sqlConn)
    if(!stats.moyTier || !stats.moyRank || stats.moyleaguePoints==undefined){
        return false
    }else{
        let embed = new (require("discord.js").MessageEmbed)()
        .setAuthor(
          "Helibot",
          "https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg"
        )
        .setTitle(`__**Le Molec est ${stats.moyTier} ${stats.moyRank}, ${stats.moyleaguePoints}LP**__`)
        //.setURL("http://77.151.84.172:2832/")
        .setColor(0)
        .setDescription("Elo des joueurs :")
        for(player of stats.players.sort((a,b)=>b.tot - a.tot).slice(0, 24)){
                embed.addField(player.name + ":", `${player.tier} ${player.rank}, ${player.lp}LP`,true)
        }
      // embed.setFooter(roles.toString())
        // if(message)
            // message.edit(embed)
        // else
            message = channel.send(embed)
        // return true
    }
}
module.exports = {init, main}
// const runEveryQuartDHeure = (callbackFn) => {
//     const Hour = 15 * 60 * 1000;
//     const currentDate = new Date();
//     const firstCall =  Hour - ((currentDate.getMinutes()%15) * 60 + currentDate.getSeconds()) * 1000 + currentDate.getMilliseconds();
//     setTimeout(() => {
//       callbackFn();
//       setInterval(callbackFn, Hour);
//     }, firstCall);
//   };