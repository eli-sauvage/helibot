import express from 'express'
import path from 'path'
import { Poll } from './poll'


export class Website{
    app:express.Application
    port:number
    poll:Poll
    constructor(poll:Poll){
        this.poll = poll
        this.app = express()
        this.app.use(express.static(path.resolve(__dirname + "/../../../views/")))
        this.app.use(express.json())
        this.port = 2832
        this.setup()
    }
    setup(){
        this.app.get('/', (req, res) => {
            res.send('En construction')
        })
        this.app.get('/poll', (req, res) => {
            res.sendFile(path.resolve(__dirname + "/../../../views/poll.html"))
        })
        this.app.post("/poll", (req, res)=>{
            res.setHeader("Access-Control-Allow-Origin", "*")
            res.send()
            this.poll.newpoll(req.body.quest, req.body.rep, req.body.every)
        })
        this.app.listen(this.port, () => {
            console.log(`website listening on port ${this.port}`)
        })
    }
}