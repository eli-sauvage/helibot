let guildId = '532956456492728320'
let scoreChannelId = "730526039570251807"
let generalChannelId = "729797578866163822"
let ancienBot = '672858838810230799'
let lol = '869264655124672652'
let pollChannel = "685183865437814976"
let VocCategory = "532956456492728323"
let generalVoc = "729002418473402438"

import { CategoryChannel, Client, Guild, TextChannel, VoiceChannel } from "discord.js"
export class Consts {
    guild: Guild
    scoreChannel: TextChannel
    generalChannel: TextChannel
    ancienBotChannel: TextChannel
    lolChannel: TextChannel
    pollChannel: TextChannel
    vocCategory:CategoryChannel
    generalVoc:VoiceChannel
    constructor(client: Client) {
        let tmp: any = client.guilds.cache.find(guild => guild.id == guildId)
        if (tmp) this.guild = tmp
        else throw "guild not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == scoreChannelId)
        if (tmp) this.scoreChannel = tmp as TextChannel
        else throw "scoreChannel not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == generalChannelId)
        if (tmp) this.generalChannel = tmp as TextChannel
        else throw "generalChannel not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == ancienBot)
        if (tmp) this.ancienBotChannel = tmp as TextChannel
        else throw "ancienBotChannel not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == lol)
        if (tmp) this.lolChannel = tmp as TextChannel
        else throw "lolChannel not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == pollChannel)
        if (tmp) this.pollChannel = tmp as TextChannel
        else throw "pollChannel not found"
        
        tmp = this.guild.channels.cache.find(channel => channel.id == VocCategory)
        if (tmp) this.vocCategory = tmp as CategoryChannel
        else throw "category not found"

        tmp = this.guild.channels.cache.find(channel => channel.id == generalVoc)
        if (tmp) this.generalVoc = tmp as VoiceChannel
        else throw "general vocal not found"
    }
}
