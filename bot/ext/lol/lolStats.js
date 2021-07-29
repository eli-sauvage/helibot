const fetch = require("node-fetch");
let tierValue = { 'IRON': 0, 'BRONZE': 1, 'SILVER': 2, 'GOLD': 3 }
let rankValue = { 'IV': 0, 'III': 1, 'II': 2, 'I': 3 }
async function computeGlobalScore(sqlConn) {
    let listOfIDs = await sqlConn.getColumn("SELECT lolID from LolIDs")
    let totRank = 0
    let moyRank = 0
    let promises = []
    let players = []
    let length = listOfIDs.length
    for (id of listOfIDs) {
        promises.push(new Promise(async (res, rej) => {
            // let id = await getID(item)
            //console.log(`https://euw1.api.riotgames.com/lol/league/v4/entries/by-summoner/${id}?api_key=${require("../../../token").lol}`)
            let response = await fetch(`https://euw1.api.riotgames.com/lol/league/v4/entries/by-summoner/${id}?api_key=${require("../../../token").lol}`)
            if(!response.ok){
                console.log("error code")
                let body = await response.json().catch(console.error)
                console.error(body.status.status_code || body)
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
            let tierFlex = body[index].tier;
            let rankFlex = body[index].rank;
            let leaguePointsFlex = body[index].leaguePoints;
            // console.log(`${summonerName} ${queueType} ${tierFlex} ${rankFlex} ${leaguePointsFlex}`);
            let rank = tierValue[tierFlex] * 400 + rankValue[rankFlex] * 100 + leaguePointsFlex
            totRank += rank 
            players.push({name:summonerName, tier:tierFlex, rank:rankFlex, lp:leaguePointsFlex, tot:rank})
            // console.log(body)
            res()
        }))
    }
    await Promise.all(promises)
    if(length == 0) return{moyTier:0, moyRank:0, moyleaguePoints:0, players:[]}
    moyenne = totRank / length
    let obj = {
        moyTier : Object.keys(tierValue).find(key => tierValue[key] === Math.floor(moyenne / 400)),
        moyRank : Object.keys(rankValue).find(key => rankValue[key] === Math.floor(Math.floor(moyenne % 400) / 100)),
        moyleaguePoints : Math.floor(Math.floor(moyenne % 400) % 100),
        players:players
    }
    sqlConn.query(`INSERT INTO lolStats (\`rank\`, tier, LP, time) VALUES("${obj.moyRank}", "${obj.moyTier}" , ${obj.moyleaguePoints}, now())`)

    return obj
}
async function getID(pseudo){
    let res = await fetch(`https://euw1.api.riotgames.com/lol/summoner/v4/summoners/by-name/${pseudo}?api_key=${require("../../../token").lol}`)
    body = await res.json()
    //console.log("id")
    //console.log(body)
    return body.id
}
module.exports = {getID, computeGlobalScore}