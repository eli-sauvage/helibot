const logs = require("./logs");
const save = require("./save");
const http = require("http");
const fs = require("fs");
class exp {
  load(guild,dir) {
    var members = [];
    var contents = fs.readFileSync(dir + "/save.json", "utf8")
    if(contents!=""){
      contents = JSON.parse(contents)
    }else{
      var files = fs.readdirSync(dir+"/save/")
      contents = JSON.parse(fs.readFileSync(dir + "/save/" + files[0],"utf8"))
    } 
    contents.forEach(e => {
      if (
        guild.members.cache.get(e.id) &&
        !members.map(e => e.user.user.id).includes(e.id)
      )
        members.push({ user: guild.members.cache.get(e.id), score: e.score || e.s || 0, name: guild.members.cache.get(e.id).user.username, ancienScore : e.ancienScore || e.as || 0 }); //ajout ds liste depuis fichier save
    });
    members = this.testNewMembers(guild, members)
    save.savefile(members);
    return members;
  }
  testNewMembers(guild, members){
    guild.members.cache.forEach(e => {
      if (!members.map(f => f.user.user.id).includes(e.id) && !e.user.bot)
        //si nv pas ds la liste
        members.push({ user: e, score: 0, name:e.user.username, ancienScore : 0});
    });
    return members
  }
}
 
module.exports = new exp()
