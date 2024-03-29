import {sql as connector} from './sqlconnector'
type sqlPoint = {
    User:string,
    Points:number,
    Name:string,
    AncienScore:number
}
export class SqlQueries{
    sql: connector
    constructor(){
        this.sql = new connector("helibot")
        // this.init()
    }
    async init(){
        await this.sql.init()
        return this
    }
    newSession(memberID:string, name:string, duration:number){
        name = SqlQueries.safeString(name)
        // console.log(`end of session for ${name}`)
        return  Promise.all([
            this.sql!.query(`INSERT INTO Session (User,  DisconnectionTime, Point, Name) VALUES (${memberID}, now(), ${duration}, "${name}")`).catch(console.error),
            this.sql!.query(`UPDATE Points SET Points = Points + ${duration} WHERE User=${memberID}`)
        ])
    }
    addToPoints(id:string, name:string){
        return this.sql!.query(`INSERT INTO Points (User, Points, Name) VALUES("${id}", 0, "${name}")`)
    }
    async getPoints():Promise<sqlPoint[]>{
        return Array.from(await this.sql!.getMultiple("SELECT * FROM Points"))
    }
    getPoint(memberID:string){
        let points = this.sql!.getSingleVal(`SELECT Points FROM Points WHERE User="${memberID}"`)
        return points
    }
    static safeString(string:string){
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

// sqlMessage:'You have an error in your SQL syntax; check the manual that corresponds to your MariaDB server version for the right syntax to use near ') + (SELECT MAX(Point) FROM Session Where User = 280769421604290560 )) WHERE ...' at line 1'
