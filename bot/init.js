class Init{
    async init(){
        await this.initDB()
    }
    async initDB(){
        let sql = await require("./sqlQueries").init()
        sql.fillNULL()
    }
}