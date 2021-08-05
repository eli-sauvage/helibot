// import { StdioNull } from "child_process"
import { sql } from "../sql/sqlconnector"
// import * as fetch from 'node-fetch'
import fetch from "node-fetch"
import {client, connection} from 'websocket'
import { LolMessage } from "./message"
import { discord as discordToken } from "../../token"
import {getID} from './lolStats'

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
        "Authorization": "Bot "+ discordToken,
        "Content-Type":"application/json"
    }
    // socketConnectAndSetListeners()
    initiateWsConn()
    function onMessage(msg:message, c:connection){
        // l(msg)
        if(msg.op == 2)c.send(JSON.stringify({op:1, d:lastS}))
        if(msg.op == 6) console.log("ws is trying to reconnect")
        if(msg.op == 9){
            console.log("code 9, reconnecting")
            c.close()
        }
        // if (msg.op == 11)console.log("hb received")
        if(msg.op == 0 && msg.t == "INTERACTION_CREATE"){
            if(msg.d.data.name == "add_lol_player")
                (async ()=>{
                    let pseudo = msg.d.data.options[0].value
                    let id = await getID(pseudo)
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
                    })//.then(console.log)
                })()
            else if(msg.d.data.name == "refresh_lol_score" || msg.d.data.custom_id == "refresh_lol_score"){
                (async ()=>{
                    let res = await message.main(true)?"bien mis à jour":"réesayez plus tard (api de LoL limitée toutes les 2 mn)"
                fetch(`https://discord.com/api/v8/interactions/${msg.d.id}/${msg.d.token}/callback`,{
                        method:"POST",
                        headers:headers,
                    body: JSON.stringify({ type: 4, data: { content: res, flags: 1 << 6}})
                    })//.then(console.log)
                })()
            }
            else if (msg.d.data.name == "clear_lol_channel" || msg.d.data.custom_id == "clear_lol_channel") {
                fetch(`https://discord.com/api/v8/interactions/${msg.d.id}/${msg.d.token}/callback`, {
                    method: "POST",
                    headers: headers,
                    body: JSON.stringify({ type: 4, data: { content: "bien reçu", flags: 1 << 6 } })
                })//.then(console.log);
                message.clearMessages(false)
            }

        }
        if(msg.s)lastS=msg.s
    }
    // function socketConnectAndSetListeners():void{
    //     socket.on('connect', c => {
    //         c.on("message", m => { if (m.type == "utf8") onMessage(JSON.parse(m.utf8Data), c) })
    //         c.on("error", m => console.log("ws error" + m))
    //         c.on("close", (c, m) => {
    //             console.log("ws closed " + c + " " + m)
    //             socketConnectAndSetListeners()
    //         })
    //         // connectWs(c)
    //     })
    //     socket.connect("wss://gateway.discord.gg/?v=8&encoding=json")
    // }
    // function connectWs(c:connection){
    //     l("ws connecting")
    //     c.send(JSON.stringify({
    //         "op": 2,
    //         "d": {
    //             "token": discordToken,
    //             "intents": 1<<4,
    //             "properties": {
    //                 "$os": "windows",
    //                 "$browser": "disco",
    //                 "$device": "disco"
    //             }
    //         },
    //         "s": null,
    //         "t": null
    //     }))
    // }
    async function initiateWsConn(){
        await new Promise<void>((fres, frej)=>{
            socket.on('connect', async c => {
                c.on("error", m => console.log("ws error" + m))
                //waiting for hello code
                await new Promise<void>((res, rej) => c.once("message", (m) => {
                    m = JSON.parse(m.utf8Data)
                    if(m.op != 10)throw "not 10"
                    // console.log("ws hello")
                    setInterval(() => {
                        // console.log("hb sent")
                        c.send(JSON.stringify({ op: 1, d: lastS }))
                    }, m.d.heartbeat_interval)
                    c.send(JSON.stringify({ op: 1, d: lastS }))
                    res()
                }))
                //waiting for ACK code
                await new Promise<void>((res, rej) => c.once("message", (m) => {
                    m = JSON.parse(m.utf8Data)
                    if(m.op != 11)throw "not 11"
                    // console.log("ws ACK")
                    res()
                }))
                //connecting
                c.send(JSON.stringify({
                    "op": 2,
                    "d": {
                        "token": discordToken,
                        "intents": 1 << 4,
                        // "shard":[0,1],
                        "properties": {
                            "$os": "linux",
                            "$browser": "osef",
                            "$device": "osef"
                        }
                    },
                    "s": null,
                    "t": null
                }), console.log)
                //waiting for ready code
                await new Promise<void>((res, rej) => c.once("message", (m) => {
                    m = JSON.parse(m.utf8Data)
                    if(m.op != 0 || m.t != "READY")throw "not 0 or ready"
                    // console.log("ws READY")
                    res()
                }))
                console.log("ws LOGGED IN")
                c.on("message", m=>{
                    if (m.type == "utf8") onMessage(JSON.parse(m.utf8Data), c)
                    else throw "not utf8"
                })
                // c.on("message", m => { if (m.type == "utf8") onMessage(JSON.parse(m.utf8Data), c) })
                c.on("close", (c, m) => {
                    console.log("ws closed " + c + " " + m)
                    // socketConnectAndSetListeners()
                })
                // connectWs(c)
                fres()
            })
            socket.connect("wss://gateway.discord.gg/?v=8&encoding=json")
        })
    }
    function l(m:any){
        if(true)console.log(m)
    }
}