const express = require("express");
const http = require("http");
const testpl = require("./testpl");
var bodyParser = require("body-parser");
class exp {
  init(members, poll) {
    const http = require("http");
    setInterval(() => {
      http.get("http://botdiscordv2.glitch.me/");
    }, 250000);
    const app = express();
    app.use(express.static("public"));
    app.use(bodyParser.json()); // to support JSON-encoded bodies
    app.use(
      bodyParser.urlencoded({
        // to support URL-encoded bodies
        extended: true
      })
    );
    app.get("/", function(request, response) {
      response.sendFile("/app/views/index.html");
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
    app.get("/points", (req, res) => {
      res.sendFile("/app/views/categoriesPtCool.html");
    });
    app.get("/poll", (req, res) => {
      res.sendFile("/app/views/poll.html");
    });
    app.post("/poll", (req, res) => {
      poll.newPoll(req.body.quest, req.body.rep, req.body.every);
    });
    setInterval(() => {
      http.get("http://discordbotv2.glitch.me/");
    }, 1000);
    const listener = app.listen(process.env.PORT, function() {
      console.log("Your app is listening on port " + listener.address().port);
    });
  }
}
module.exports = new exp();
