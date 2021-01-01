const fs = require("fs");
fetch = require("node-fetch")
// const axios = require("axios")

class exp {
  json(members, long) {
    let savedata = [];
    members.forEach(e => {
      if (!long) {
        savedata.push({
          username: e.user.user.username,
          score: Math.round(e.score),
          id: e.user.user.id,
          ancienScore : Math.round(e.ancienScore)
        });
      } else if (Math.round(e.ancienScore) != 0) {
        savedata.push({ id: e.user.user.id, s: Math.round(e.score), as:Math.round(e.ancienScore) });
      }
    });
    savedata.sort((a, b) => b.score - a.score);
    return savedata;
  }
  savefile(members) {
    fs.writeFile(
      "save.json",
      JSON.stringify(this.json(members, false)),
      function(err) {
        if (err) throw err;
      }
    );
  }
  saveFileHistorique(members,dir) {
    fs.readdir(dir + "/save/", function(err, fichiers) {
      if(fichiers)
        for (var i = 0; i < fichiers.length - 1; i++) {
          fs.unlink(dir + "/save/" + fichiers[i],(err)=>{});
        }
    });
    var json = this.json(members, true);
    if (json == "" || json == {} || json == []) return;
    fs.writeFile(
      dir + "/save/" + new Date().toLocaleString().replace(/ /gi, '_').replace(/:/gi, '.').replace(/\//gi,'-') + ".json",
      JSON.stringify(this.json(members, true)),
      function(err) {
        if (err) throw err;
      }
    );
  }
  saveOrdi(members, dir){
    var json = this.json(members, true);
    if (json == "" || json == {} || json == []) return;
    fs.writeFile(
      dir + "/saves/" + new Date().toLocaleString().replace(/ /gi, '_').replace(/:/gi, '.').replace(/\//gi,'-') + ".json",
      JSON.stringify(this.json(members, true)),
      function(err) {
        if (err) throw err;
      }
    );
  }
}
module.exports = new exp();
