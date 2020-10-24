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
          id: e.user.user.id
        });
      } else if (Math.round(e.score) != 0) {
        savedata.push({ id: e.user.user.id, s: Math.round(e.score) });
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
  saveOrdi(members){
    var json = this.json(members, true);
    if (json == "" || json == {} || json == []) return;
    fs.writeFile(
      dir + "/saves/" + new Date().toLocaleString().replace(/ /gi, '_').replace(/:/gi, '.').replace(/\//gi,'-') + ".json",
      JSON.stringify(this.json(members, true)),
      function(err) {
        if (err) throw err;
      }
    );
  //   try{
  //     fetch("http://77.151.84.172:5555", {
  //     method: "POST",
  //     headers: {
  //       Accept: "application/json",
  //       "Content-Type": "application/json"
  //     },
  //     body: JSON.stringify(json)
  //   })
  // }catch{}
  }
}
module.exports = new exp();
