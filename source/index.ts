import {existsSync, readFileSync, writeFileSync} from 'fs'

import {Telegraf, Context as TelegrafContext} from 'telegraf'

import {doit as loadFromMediaObjects} from './media-objects'
import {META_TARGET_CHAT} from './constants'
import {sleep} from './generics'

process.title = 'wdrmaus-downloader'

const token = (existsSync('/run/secrets/bot-token.txt') && readFileSync('/run/secrets/bot-token.txt', 'utf8').trim()) ||
	(existsSync('bot-token.txt') && readFileSync('bot-token.txt', 'utf8').trim()) ||
	process.env.BOT_TOKEN
if (!token) {
	throw new Error('You have to provide the bot-token from @BotFather via file (bot-token.txt) or environment variable (BOT_TOKEN)')
}

const telegrafOptions: Partial<Telegraf.Options<TelegrafContext>> = {}
if (process.env.TELEGRAM_API_ROOT) {
	telegrafOptions.telegram = {
		apiRoot: process.env.TELEGRAM_API_ROOT
	}
}

const bot = new Telegraf(token, telegrafOptions)

async function handleError(context: string, error: unknown): Promise<void> {
	console.error('ERROR', context, error)
	let text = ''
	text += 'Error in context: '
	text += context
	text += '\n'

	if (error instanceof Error) {
		text += error.name
		text += ': '
		text += error.message
	} else {
		text += String(error)
	}

	await bot.telegram.sendMessage(META_TARGET_CHAT, text)
}

async function run(): Promise<void> {
	await loadFromMediaObjects(bot.telegram, handleError)
	writeFileSync('.last-successful-run', new Date().toISOString(), 'utf8')
}

async function startup(): Promise<void> {
	if (process.env.NODE_ENV === 'production') {
		// Dont run immediately as volume might need time to setup
		await sleep(1000 * 60 * 10) // 10 minutes

		while (true) {
			try {
				// eslint-disable-next-line no-await-in-loop
				await run()
			} catch (error: unknown) {
				console.error('main run procedure failed', error)
			}

			// eslint-disable-next-line no-await-in-loop
			await sleep(1000 * 60 * 35) // Every 35 minutes
		}
	} else {
		await run()
	}
}

// eslint-disable-next-line @typescript-eslint/no-floating-promises
startup()
