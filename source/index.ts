/* eslint-disable @typescript-eslint/no-floating-promises */
import {existsSync, readFileSync} from 'fs'

import Telegraf, {Extra} from 'telegraf'

import {doit as loadFromMediaObjects} from './media-objects'
import {ERROR_TARGET} from './constants'
import {sync} from './resilio'

if (process.env.NODE_ENV === 'production') {
	sync()
}

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

	currentlyRunning = false
}

if (process.env.NODE_ENV === 'production') {
	// Dont run immediately as resilio might need time to setup
	const interval = setInterval(run, 1000 * 60 * 60) // Every 60 minutes
	process.on('SIGINT', () => clearInterval(interval))
	process.on('SIGTERM', () => clearInterval(interval))
} else {
	run()
}
