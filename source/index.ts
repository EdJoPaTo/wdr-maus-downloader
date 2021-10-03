import {ApiClientOptions, Bot} from 'https://deno.land/x/grammy/mod.ts'

import {doit as loadFromMediaObjects} from './media-objects/index.ts'
import {META_TARGET_CHAT} from './constants.ts'
import {sleep} from './generics.ts'

const token = Deno.env.get('BOT_TOKEN')
if (!token) {
	throw new Error('You have to provide the bot-token from @BotFather via file (bot-token.txt) or environment variable (BOT_TOKEN)');
}

const client: ApiClientOptions = {}
const apiRoot = Deno.env.get('TELEGRAM_API_ROOT')
if (apiRoot) {
	client.apiRoot = apiRoot
	client.baseFetchConfig = {}
}

const bot = new Bot(token, {client});

async function handleError(context: string, error: unknown): Promise<void> {
	console.error('ERROR', context, error);
	let text = '';
	text += 'Error in context: ';
	text += context;
	text += '\n';

	if (error instanceof Error) {
		text += error.name;
		text += ': ';
		text += error.message;
	} else {
		text += String(error);
	}

	await bot.api.sendMessage(META_TARGET_CHAT, text);
}

async function run(): Promise<void> {
	await loadFromMediaObjects(bot.api, handleError)
	Deno.writeTextFileSync('.last-successful-run', new Date().toISOString())
}

async function startup(): Promise<void> {
	const {username} = await bot.api.getMe();
	console.log('bot connection works: bot is', username);

	if (Deno.env.get('NODE_ENV') === 'production') {
		// Dont run immediately as volume might need time to setup
		await sleep(1000 * 60 * 10); // 10 minutes

		// eslint-disable-next-line no-constant-condition
		while (true) {
			try {
				// eslint-disable-next-line no-await-in-loop
				await run();
			} catch (error: unknown) {
				console.error('main run procedure failed', error);
			}

			const now = new Date();
			const isSunday = now.getDay() === 0;
			const hour = now.getHours();
			if (isSunday && hour >= 7 && hour <= 12) {
				// eslint-disable-next-line no-await-in-loop
				await sleep(1000 * 60 * 42); // Every 42 minutes
			} else if (isSunday && hour >= 2 && hour <= 12) {
				// eslint-disable-next-line no-await-in-loop
				await sleep(1000 * 60 * 60 * 2.123); // Every 2 hours and a bit (in order not to miss the normal operation sunday window)
			} else {
				// eslint-disable-next-line no-await-in-loop
				await sleep(1000 * 60 * 60 * 8.123); // Every 8 hours and a bit
			}
		}
	} else {
		await run();
	}
}

// eslint-disable-next-line @typescript-eslint/no-floating-promises
startup();
