module.exports = class sql{    
    constructor(db){
        return new Promise((res, rej)=>{
            this.mySQl = require('mysql')
            this.conn = this.mySQl.createConnection({
                host: '77.151.84.172', 
                port:'3306',
                user:'bot',
                password: require("../token").sql,
                database: db,
                charset : "utf8mb4"
            })
            this.conn.connect((e)=>{
                if(e)
                    rej(e)
                else
                    res(this)
            })  
        })
    }
    query(query){
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

    getSingleVal(query){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res)=>{
                if(e) rej(e)
                if(res.length != 1)return rej("get single prop returned > 1 or 0 row")
                if(Object.keys(res[0]).length != 1) rej("get single prop returned > 1 or 0 columns")
                resolve(res[0][Object.keys(res[0])])
            })
        })
    }
    getRow(query){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res)=>{
                if(e) rej(e)
                if(res.length != 1) rej("get single prop returned > 1 or 0 row")
                resolve(res[0])
            })
        })
    }

    getColumn(query){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res)=>{
                if(e) rej(e)
                if(!res.every(e=>Object.keys(e).length == 1))rej("get getColumn returned > 1 or 0 columns")
                resolve(res.map(e=>e[Object.keys(e)[0]]))
            })
        })
    }

    getMultiple(query){
        return new Promise((resolve, rej)=>{
            this.conn.query(query, (e,res)=>{
                if(e) rej(e)
                resolve(res)
            })
        })
    }

    get(type, query){
        switch(type){
            case 'val':return this.getSingleVal(query)
            case 'row':return this.getRow(query)
            case 'column':return this.getColumn(query)
            case '':return this.getMultiple(query)
            default:console.error(`wrong type specified, received :${type}`)
        }
    }
}