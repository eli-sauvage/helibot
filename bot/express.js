const express = require("express");
const http = require("http");
const testpl = require("./testpl");
const shell = require("shelljs")
const https = require("https")
var bodyParser = require("body-parser");
// const cors = require("cors")
const fs = require("fs")
var  listener
class exp {
  init(members, poll, appDirName) {
    const app = express();
    // var server = https.createServer({
    //   key : fs.readFileSync(appDirName + "/ssl/key.pem"),
    //   cert : fs.readFileSync(appDirName + "/ssl/cert.pem")
    // },app).listen(2832,()=>console.log("listening"))
    app.use(express.static("public"));
    app.use(bodyParser.json()); // to support JSON-encoded bodies
    app.use(
      bodyParser.urlencoded({
        // to support URL-encoded bodies
        extended: true
      })
    );
    // app.use(cors())
    app.get("/", function(request, response) {
      response.sendFile(appDirName + "/public/scores.html");
      console.log("req")
    });
    app.get("/memb", function(request, response) {
      var rep = [];
      members.forEach(e => {
        var name = e.user.nickname || e.user.user.username;
        rep.push({ score: e.score, name: name, co: testpl(e) });
      });
      response.send(rep);
    });
    // app.get("/points", (req, res) => {
    //   res.sendFile("/app/views/categoriesPtCool.html");
    // });
    app.get("/poll", (req, res) => {
      res.sendFile(appDirName + "/public/poll.html");
      console.log("poll req")
    });
    app.post("/poll", (req, res) => {
      res.send("")
      console.log(req.body.quest)
      if(!(req.body.quest && req.body.rep && req.body.every!=undefined)){
        console.log("mauvaise requete")
        return
      }
      console.log("requete")
      poll.newPoll(req.body.quest, req.body.rep, req.body.every);
    });
    app.post("/gitchange",(req,res)=>{
      res.send("")
      listener.close() 
      shell.exec("bash ~/startbot.sh")
      process.exit()
    })
    app.get("/ping",(req,res)=>{
      res.send("");
    })
    listener = app.listen(2832, function() {
      console.log("Your app is listening on port " + listener.address().port);
    });
  }
  stopPortAndApp(){
    listener.close(()=>{
      shell.exec("bash ~/startbot.sh")
      process.exit()
    })
  }
}
module.exports = new exp();
