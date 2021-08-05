import fetch from "node-fetch"
import {lol} from '../../token'
import { sql } from "../sql/sqlconnector"

let tierValue = { 'IRON': 0, 'BRONZE': 1, 'SILVER': 2, 'GOLD': 3 }
type tier ="IRON" |"BRONZE" | "SILVER" | "GOLD"
let rankValue = { 'IV': 0, 'III': 1, 'II': 2, 'I': 3 }
type rank = "IV" | 'III' | "II" | "I"

export interface stats{
    moyTier:string,
    moyRank:string,
    moyleaguePoints:number,
    players:{
        name:string,
        tier:string,
        rank:string,
        lp:number,
        tot:number
    }[]
}
export async function computeGlobalScore(sqlConn:sql) {
    let listOfIDs = await sqlConn.getColumn("SELECT lolID from LolIDs") as number[]
    let totRank = 0
    let moyRank = 0
    let promises:Promise<void>[] = []
    let players:stats["players"] = []
    let length = listOfIDs.length
    for (let id of listOfIDs) {
        promises.push(new Promise<void>(async (res, rej) => {
            // let id = await getID(item)
            let response = await fetch(`https://euw1.api.riotgames.com/lol/league/v4/entries/by-summoner/${id}?api_key=${lol}`)
            if(!response.ok){
                console.log("error code")
                let body = await response.json().catch(console.error)
                console.error(body.status.message || body)
                rej("error from lol api : " + body.status.message || body)
                return
            }
            let body = await response.json()//.catch(console.log)
            if(!body || !body[0] || !body[0].queueType){
                length--
                res()
                return
            }
            let index
            if ((body[0].queueType) == "RANKED_FLEX_SR") {index = 0}
            else if(body[1] && body[1].queueType == "RANKED_FLEX_SR"){index = 1}
            else{
                length--
                res()
                return
            }
            let queueType = (body[index].queueType);
            let summonerName = (body[index].summonerName);
            let tierFlex = body[index].tier as tier;
            let rankFlex = body[index].rank as rank;
            let leaguePointsFlex = body[index].leaguePoints;
            // console.log(`${summonerName} ${queueType} ${tierFlex} ${rankFlex} ${leaguePointsFlex}`);
            let rank = tierValue[tierFlex] * 400 + rankValue[rankFlex] * 100 + leaguePointsFlex
            totRank += rank 
            players.push({name:summonerName, tier:tierFlex, rank:rankFlex, lp:leaguePointsFlex, tot:rank})
            // console.log(body)
            res()
        }))
    }
    try{await Promise.all(promises)}
    catch(e){
        console.error("error : " + e)
        return 
    }
    if(length == 0) return{moyTier:'', moyRank:'', moyleaguePoints:0, players:[]}
    let moyenne = totRank / length
    let obj = {
        moyTier : Object.keys(tierValue).find(key => tierValue[key as tier] === Math.floor(moyenne / 400)),
        moyRank : Object.keys(rankValue).find(key => rankValue[key as rank] === Math.floor(Math.floor(moyenne % 400) / 100)),
        moyleaguePoints : Math.floor(Math.floor(moyenne % 400) % 100),
        players:players
    } as stats
    sqlConn.query(`INSERT INTO lolStats (\`rank\`, tier, LP, time) VALUES("${obj.moyRank}", "${obj.moyTier}" , ${obj.moyleaguePoints}, now())`)
    return obj
}
export async function getID(pseudo:string){
    let res = await fetch(`https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-name/${pseudo}?api_key=${lol}`)
    let body = await res.json()
    //console.log("id")
    //console.log(body)
    return body.id
}
module.exports = {getID, computeGlobalScore}