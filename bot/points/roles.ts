import { Guild, GuildMember, Role } from "discord.js";
import { Points } from "./points";

export class Roles {
    guild: Guild
    points: Points
    constructor(guild: Guild, points: Points) {
        this.guild = guild
        this.points = points
        setInterval(() => {
            this.testForNewRoles()
        }, 30 * 1000)
        this.testForNewRoles()
    }
    static toString(): string {
        let str = ""
        for (let i = rolesList.length - 1; i >= 0; i--)
            str += rolesList[i].name + ": " + rolesList[i].seuil + (i == 0 ? "" : ", ")
        return str
    }
    async testForNewRoles() {
        let currentPoints = await this.points.exportPoints()
        for (let point of currentPoints) {
            let guildUser = this.guild.members.cache.find(memb => memb.id == point.User)
            if (!guildUser) continue
            let userRoleIds = guildUser.roles.cache.map(role => role.id)
            for (let role of rolesList) {
                if (point.Points >= role.seuil) {
                    if (!userRoleIds.includes(role.id)) {//si superieur à seuil et que user ne l'a pas encore
                        this.addRole(guildUser, role.id)
                    }
                    break
                }
            }
        }
    }
    async addRole(memb: GuildMember, roleId: string): Promise<void> {
        rolesList.forEach(role => {
            memb.roles.remove(roleId)
        })
        memb.roles.add(roleId);//on ajoute le bon
    }
}


const rolesList = [
    { id: "768193206675832843", seuil: 100000, name: "bouf royal" },
    { id: "550342067210027030", seuil: 75000, name: "hokage" },
    { id: "550343591239614467", seuil: 50000, name: "batracien" },
    { id: "550343935869059073", seuil: 25000, name: "abu yaqub" },
    { id: "550343939744333836", seuil: 20000, name: "erudit" },
    { id: "550343942437077002", seuil: 15000, name: "wakam" },
    { id: "550343944018329611", seuil: 10000, name: "cresus" },
    { id: "550344215666753538", seuil: 5000, name: "damoiseau" },
    { id: "550344219341094932", seuil: 1500, name: "strapontin" },
    { id: "550344217524961310", seuil: 500, name: "cul-terreux" },
    { id: "546359889014947850", seuil: 0, name: "subalternes" }
];