class SqlQueries{
    async init(){
        this.sql = await new (require("./sqlconnector"))("helibot")
        return this
    }
    async addMember(memberID){
        return new Promise(async (res, rej)=>{
            await this.sql.query(`INSERT INTO Session (User, ConnectionTime, DisconnectionTime, Point) VALUES (${memberID}, now(), now(), 0);`).catch(rej)
            res()
        })
    }
    async onMemberConnect(memberID){
        this.sql.query(`INSERT INTO Session (User, ConnectionTime, Point) VALUES (${memberID}, now(), (SELECT MAX(Point) FROM (select * from Session) AS m2 WHERE User = ${memberID}));`)
    }
    async onMemberDisconnect(memberID){
        let sessionID = await this.sql.getSingleVal(`SELECT max(id) FROM Session Where User = ${memberID}`)
        this.sql.query(`UPDATE Session SET DisconnectionTime = now(), Point = (Select now() - (SELECT ConnectionTime FROM Session WHERE id = ${sessionID}) + (SELECT MAX(Point) FROM Session Where User = ${memberID} ) WHERE id = ${sessionID}) WHERE  id = ${sessionID};`)
    }
    getPoints(){
        let points = this.sql.getMultiple("select t.User, t.Point\
        from Session t\
        inner join (\
            select User, max(ConnectionTime) as maxCo\
            from Session\
            group by User\
        ) tm on t.User = tm.User and t.connectionTime = tm.maxCo")
        return points
    }
    fillNULL(){
        this.sql.query("update Session set \
        DisconnectionTime = ConnectionTime\
        WHERE id IN (select id from Session where DisconnectionTime IS NULL);")
    }
}
module.exports = new SqlQueries();