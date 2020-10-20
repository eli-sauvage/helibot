const logs = require("./logs.js");
var rolesList = [
  { id: "768193206675832843", seuil: 100000, name: "bouf royal" },
  { id: "550342067210027030", seuil: 75000, name: "hokage" },
  { id: "550343591239614467", seuil: 50000, name: "batracien" },
  { id: "550343935869059073", seuil: 25000, name: "abu yaqub" },
  { id: "550343939744333836", seuil: 20000, name: "erudit" },
  { id: "550343942437077002", seuil: 15000, name: "wakam" },
  { id: "550343944018329611", seuil: 10000, name: "cresus" },
  { id: "550344215666753538", seuil: 5000, name: "damoiseau" },
  { id: "550344219341094932", seuil: 1500, name: "strapontin" },
  { id: "550344217524961310", seuil: 500, name: "cul-terreux" },
  { id: "546359889014947850", seuil: 0, name: "subalternes" }
];
class exp {
  up(members) {
    members.forEach(memb => {
      testuprole(memb);
    });
  }
  res() {}
}
module.exports = new exp();

function testuprole(memb) {
  var n = memb.user.user.username;
  for (var i = 0; i < rolesList.length; i++) {//on parcours les roles
    const role = rolesList[i];
    /*if(memb.score >= rolesList[i+1].seuil){//si score du joueur > role suivant
      memb.user.removeRole(role.id);
    }*/
    if (memb.score >= role.seuil) {//role max
      if (!memb.user.roles.map(e => e.id).includes(role.id)){//si a pas deja le role
        addRole(memb,role)
      }
      return;
    }
  }
}


async function addRole(memb,role) {
  //await memb.user.removeRole(rolesList.map(e=>e.id));//on retire les roles
  rolesList.forEach(role=>{
    memb.user.removeRole(role.id)
  }
  memb.user.addRole(role.id);//on ajoute le bon
  logs.role(memb, role);
}
