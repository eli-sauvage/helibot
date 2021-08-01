import * as mySql from 'mysql2'
import {sql as sqlToken} from '../token'
export class sql{
    conn:mySql.Connection
    constructor(db:string){
        this.conn = mySql.createConnection({
            host: '5.188.70.92', 
            port:3306,
            user:'bot',
            password: sqlToken,
            database: db,
            charset : "utf8mb4"
        })
    }
    async init():Promise<sql>{
        return new Promise((res, rej)=>{
            this.conn.connect((e)=>{
                if(e)
                    rej(e)
                else
                    res(this)
            })  
        })
    }
    query(query:string){
        return new Promise((res, rej)=>{
            this.conn.query(query, (e, result)=>{
                if(e){
                    console.error(e)
                    rej(e)
                }else
                    res(result)
            })
        }).catch(console.log)
    }

    getSingleVal(query:string){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res:any)=>{
                if(e) rej(e)
                // res = Array.from(res)
                if(res.length != 1)return rej("get single prop returned > 1 or 0 row")
                if(Object.keys(res[0]).length != 1) rej("get single prop returned > 1 or 0 columns")
                resolve(res[0][Object.keys(res[0])[0]])
            })
        })
    }
    getRow(query:string){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res:any)=>{
                if(e) rej(e)
                if(res.length != 1) rej("get single prop returned > 1 or 0 row")
                resolve(res[0])
            })
        })
    }

    getColumn(query:string){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res:any)=>{
                if(e) rej(e)
                if(!res.every((e:any)=>Object.keys(e).length == 1))rej("get getColumn returned > 1 or 0 columns")
                resolve(res.map((e:any)=>e[Object.keys(e)[0]]))
            })
        })
    }

    getMultiple(query:string):Promise<any>{
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res)=>{
                if(e) rej(e)
                resolve(res)
            })
        })
    }

    get(type:string, query:string){
        switch(type){
            case 'val':return this.getSingleVal(query)
            case 'row':return this.getRow(query)
            case 'column':return this.getColumn(query)
            case '':return this.getMultiple(query)
            default:console.error(`wrong type specified, received :${type}`)
        }
    }
}