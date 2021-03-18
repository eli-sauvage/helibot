class SqlQueries{
    async init(){
        this.sql = await new (require("./sqlconnector"))("helibot")
        return this
    }
    async addMember(memberID, name){
        name = SqlQueries.safeString(name)
        return new Promise(async (res, rej)=>{
            await this.sql.query(`INSERT INTO Session (User, ConnectionTime, DisconnectionTime, Point, Name) VALUES (${memberID}, now(), now(), 0, '${name}');`).catch(rej)
            // console.log(`added ${name} to the list`)
            res()
        })
    }
    async onMemberConnect(memberID, name){
        name = SqlQueries.safeString(name)
        var connTime = await this.sql.getSingleVal("select DisconnectionTime from Session where id = (Select max(id) from Session where User = 395370353460772865)")
        if(connTime==null)return //already has a session
        this.sql.query(`INSERT INTO Session (User, ConnectionTime, Point, Name) VALUES (${memberID}, now(), (SELECT MAX(Point) FROM (select * from Session) AS m2 WHERE User = ${memberID}), '${name}');`)
    }
    async onMemberDisconnect(memberID){
        let sessionID = await this.sql.getSingleVal(`SELECT max(id) FROM Session Where User = ${memberID}`)
        this.sql.query(`UPDATE Session SET DisconnectionTime = now(), Point = (TIMESTAMPDIFF(MINUTE, (SELECT ConnectionTime FROM Session WHERE id = ${sessionID}), now()) + (SELECT MAX(Point) FROM Session Where User = ${memberID} )) WHERE id = ${sessionID};`)
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
    static safeString(string){
        var arr = [...string]
        for(var i=0; i<arr.length; i++){
            if(arr[i] == "'"){
                arr.splice(i, 0, "\\")
                i++
            } 
        }
        return arr.join('')
    }
}
module.exports = new SqlQueries();

// sqlMessage:'You have an error in your SQL syntax; check the manual that corresponds to your MariaDB server version for the right syntax to use near ') + (SELECT MAX(Point) FROM Session Where User = 280769421604290560 )) WHERE ...' at line 1'
