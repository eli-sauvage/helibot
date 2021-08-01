// import { StdioNull } from "child_process"
import { sql } from "../../sqlconnector"
// import * as fetch from 'node-fetch'
import fetch from "node-fetch"
import {client, connection} from 'websocket'
import { LolMessage } from "./message"
interface message{
    op:number,
    d:any,
    s:number,
    t:string
}

export function Ws(sqlConn:sql, message:LolMessage){
    let socket = new client()
    let lastS:number | null = null
    let headers = {
        "Authorization": "Bot "+require("../../../token").discord,
        "Content-Type":"application/json"
    }
    socket.on('connect', c=>{
        console.log("connected")
        c.on("message", m=>{if(m.type == "utf8")onMessage(JSON.parse(m.utf8Data), c)})
        c.on("error", console.log)
        connectWs(c)
        c.on("close", console.log)
    })
    socket.connect("wss://gateway.discord.gg/?v=8&encoding=json")

    function onMessage(msg:message, c:connection){
        // if(msg.op !=0)l(msg)
        if(msg.op == 10){
            setInterval(()=>c.send(JSON.stringify({op:1, d:lastS})),msg.d.heartbeat_interval)
        }
        if(msg.op == 2)c.send(JSON.stringify({op:1, d:lastS}))
        if(msg.op == 9)connectWs(c)
        if(msg.op == 0 && msg.t == "INTERACTION_CREATE"){
            if(msg.d.data.name == "add_lol_player")
                (async ()=>{
                    let pseudo = msg.d.data.options[0].value
                    let id = await require("./lolStats").getID(pseudo)
                    let res
                    if(id){
                        await sqlConn.query(`INSERT INTO LolIDs (pseudo, lolID) VALUES("${pseudo}", "${id}")`)
                        res = `le joueur ${pseudo} a bien été ajouté`
                    }else{
                        res = "le joueur n'a pas été trouvé"
                    }
                    fetch(`https://discord.com/api/v8/interactions/${msg.d.id}/${msg.d.token}/callback`,{
                        method:"POST",
                        headers:headers,
                        body:JSON.stringify({type:4, data:{content:res}})
                    }).then(console.log)
                })()
            else if(msg.d.data.name == "refresh_lol_score"){
                (async ()=>{
                    let res = await message.main(true)?"bien mis à jour":"réesayez plus tard"
                fetch(`https://discord.com/api/v8/interactions/${msg.d.id}/${msg.d.token}/callback`,{
                        method:"POST",
                        headers:headers,
                        body:JSON.stringify({type:4, data:{content:res}})
                    }).then(console.log)
                })()
            }

        }
        if(msg.s)lastS=msg.s
    }
    function connectWs(c:connection){
        l("connecting")
        c.send(JSON.stringify({
            "op": 2,
            "d": {
                "token": require("../../../token").discord,
                "intents": 513,
                "properties": {
                    "$os": "windows",
                    "$browser": "helibot",
                    "$device": "helibot"
                }
            }
        }))
    }
    function l(m:string){
        if(true)console.log(m)
    }
}