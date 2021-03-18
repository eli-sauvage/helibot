const { SlowBuffer } = require("buffer")
const fs = require("fs")
let content = JSON.parse(fs.readFileSync(process.argv[2], "utf-8"))
let Sql = require("./bot/sqlconnector")
// f()
async function f(){
    let sql = await new Sql("helibot")
    // console.log(sql.query())
    let scores = await sql.query("select * from Session")
    // console.log(content)
    content.forEach(e=>{
        // sql.query(`UPDATE Session set Point = ${e.score * 60} where User = ${e.id}`)
        if(!scores.map(e=>e.User).includes(e.id)){
            sql.query(`INSERT INTO ancienScores (User, name, score) VALUES (${e.id}, "${e.username}", ${e.ancienScore-e.score});`)
            sql.query(`INSERT INTO Session (User, Name, ConnectionTime, DisconnectionTime, Point) VALUES (${e.id}, "${e.username}", now(), now(), ${e.score});`)
        }        
        // sql.query(`INSERT INTO ancienScores (User, name, score) VALUES (${e.id}, '${e.username}', ${e.ancienScore - e.score});`).catch("NON")
    })
}
// console.log("オウェール".charCodeAt(0))
// console.log(content)