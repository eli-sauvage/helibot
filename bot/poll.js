let reacts = [
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
var channel;
var discord;
var testDouble = function(arr){
  var enDouble = []
  	arr.forEach(e=>{
      if(arr.filter(item => item == e).length > 1){
        if(!enDouble.includes(e))
          enDouble.push(e)
      }
	})
  return enDouble
}
class exp {
  init(c, d) {
    channel = c;
    discord = d;
  }
  newPoll(q, r, m) {
        console.log("poll");
    var msg = new discord.MessageEmbed()
      .setColor("#0099ff")
      .setTitle("Sondage")
      //.setURL("site en cours")//https://murmuring-dawn-90139.herokuapp.com/poll")
      .setAuthor("helibot","https://images-na.ssl-images-amazon.com/images/I/615Q1Ms%2Bb4L._SX425_.jpg")
      .setDescription(q)
      .setTimestamp()
      .setFooter("http://helibot.biz.uz")
    var i = 0;
    r.forEach(e => {
      msg.addField(e, reacts[i], true);
      i++;
    });
    if(m)
      channel.send("@everyone, " + q).then(e=>e.delete())
    channel.send(msg).then(async function(m){
      for(var i=0;i<r.length;i++){
        await m.react(reacts[i])
      }
    }).catch();
  }
  newReact(react,user){
    /*if(react.me || !react.users.cache.array().map(e=>e.bot).includes(true))
      return
    var userArray = []
    Array.from(react.message.reactions.cache).forEach(e=>{
      userArray = userArray.concat(e[1].users.array())
    })
    console.log(userArray.map(e=>e.username))*/
    /*if(testDouble(userArray).length && testDouble(userArray).includes(user)){
      var msg = "double vote : " 
      testDouble(userArray).map(e=>e.username).forEach(e=>{
        msg = msg.concat(e + " ")
      })
      //react.message.channel.send(msg)
      //console.log("double=>" + testDouble(userArray).map(e=>e.username))
    }*/
  }
}
module.exports = new exp();
