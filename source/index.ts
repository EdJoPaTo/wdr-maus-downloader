/* eslint-disable @typescript-eslint/no-floating-promises */
import {existsSync, readFileSync, writeFileSync} from 'fs'

import Telegraf, {Extra} from 'telegraf'

import {doit as loadFromMediaObjects} from './media-objects'
import {ERROR_TARGET} from './constants'

process.title = "wdrmaus-downloader"

const tokenFilePath = existsSync('/run/secrets') ? '/run/secrets/bot-token.txt' : 'bot-token.txt'
const token = readFileSync(tokenFilePath, 'utf8').trim()
const bot = new Telegraf(token)

async function handleError(context: string, error: any): Promise<void> {
	console.log(error)
	await bot.telegram.sendMessage(ERROR_TARGET, context + '\n```\n' + JSON.stringify(error, null, 2) + '\n```', Extra.markdown() as any)
}

let currentlyRunning = false
async function run(): Promise<void> {
	if (currentlyRunning) {
		return
	}

	currentlyRunning = true

	await loadFromMediaObjects(bot.telegram, handleError)

	writeFileSync('.last-successful-run', new Date().toISOString(), 'utf8')
	currentlyRunning = false
}

if (process.env.NODE_ENV === 'production') {
	// Dont run immediately as volume might need time to setup
	setInterval(run, 1000 * 60 * 35) // Every 35 minutes
} else {
	run()
}
