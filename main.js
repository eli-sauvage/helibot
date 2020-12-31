// require("./bot/vocaux")

const SQL = require("./bot/sql/sqlconnector")
const Utils = require("./bot/utils");

const sql = new SQL()
const utils = new Utils()
const c = utils.catch

const discord = require("discord.js")

var scoreMessage, scoreChannel, logsChannel;
var client = new discord.Client({ws:{intents:discord.Intents.ALL}});
var guild;
var members = [];

client.on("ready", async function() {
  console.log(await sql.get('', "select * from connections").catch(c))
});

client.on("message", msg => {})

client.login(require("./token"))