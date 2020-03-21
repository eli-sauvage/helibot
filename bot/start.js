const logs = require("./logs");
const save = require("./save");
const http = require("http");
const fs = require("fs");
class exp {
  load(guild) {
    var members = [];
    var contents = fs.readFileSync("save.json", "utf8")
    if(contents!=""){
      contents = JSON.parse(contents)
    }else{
      var files = fs.readdirSync("../save/")
      contents = JSON.parse(fs.readFileSync("../save/" + files[0],"utf8"))
    } 
    contents.forEach(e => {
      if (
        guild.members.get(e.id) &&
        !members.map(e => e.user.user.id).includes(e.id)
      )
        members.push({ user: guild.members.get(e.id), score: e.score || e.s || 0}); //ajout ds liste depuis fichier save
    });
    guild.members.forEach(e => {
      if (!members.map(f => f.user.user.id).includes(e.id) && !e.user.bot)
        //si nv pas ds la liste
        members.push({ user: e, score: 0 });
    });
    save.savefile(members);
    return members;
  }
}
module.exports = new exp()
