import {existsSync, readFileSync, createWriteStream, promises as fsPromises} from 'fs'

import request from 'request-promise-native'
import Telegraf, {Extra} from 'telegraf'

import {download} from './download'
import {sync} from './resilio'

const TARGET_CHAT = '-1001214301516'
const ERROR_TARGET = '-1001214301516'
const BASE_URL = 'https://www.wdrmaus.de/aktuelle-sendung/'
const FILE_PATH = './tmp/'

if (process.env.NODE_ENV === 'production') {
	sync()
}

const tokenFilePath = existsSync('/run/secrets') ? '/run/secrets/bot-token.txt' : 'bot-token.txt'
const token = readFileSync(tokenFilePath, 'utf8').trim()
const bot = new Telegraf(token)

async function sendWhenNew(): Promise<void> {
	const source = await request(BASE_URL)
	const imgPart = /aktuelle-sendung\/([^"]+.jpg)/.exec(source)![1]
	const img = BASE_URL + imgPart

	const mediaObjUrl = /mediaObj': { 'url': '(https:[^']+)/.exec(source)![1]
	const mediaObjString = await request(mediaObjUrl)
	const mediaObjJsonString = mediaObjString
		.replace('$mediaObject.jsonpHelper.storeAndPlay(', '')
		.slice(0, -2)
	const mediaObjJson = JSON.parse(mediaObjJsonString)

	const date: string = mediaObjJson.trackerData.trackerClipAirTime
	const dateFilenamePart = parseDateToFilenamePart(date)
	const filenamePrefix = FILE_PATH + 'wdrmaus-' + dateFilenamePart + '-'

	const last = await getLastRunMediaObj()
	const areEqual = JSON.stringify(last) === JSON.stringify(mediaObjJson)
	if (areEqual) {
		console.log('same as last time')
		// The last one is the same
		return
	}

	const photoMessage = await bot.telegram.sendPhoto(TARGET_CHAT, img, new Extra({
		caption: date
	}).notifications(false) as any)

	await saveMediaObj(mediaObjJson)

	const dgsVideo: string = mediaObjJson.mediaResource.dflt.slVideoURL
	const normalVideo: string = mediaObjJson.mediaResource.dflt.videoURL
	const captionsUrl: string = mediaObjJson.mediaResource.captionsHash.srt

	let caption = ''
	caption += '\n' + date
	caption += '\nVideo: https:' + normalVideo
	caption += '\nDGS: https:' + dgsVideo
	caption += '\nUntertitel: https:' + captionsUrl

	await bot.telegram.sendMessage(TARGET_CHAT, caption, Extra.inReplyTo(photoMessage.message_id) as any)

	console.log('start download…')
	console.time('download')

	console.time('download 1image')
	request.get(img)
		.pipe(createWriteStream(filenamePrefix + '1image.jpg'))
	console.timeEnd('download 1image')

	console.time('download 2normal')
	await download('https:' + normalVideo, 'https:' + captionsUrl, filenamePrefix + '2normal.mp4')
	console.timeEnd('download 2normal')

	// Disable temporarily
	// console.time('download 3dgs')
	// await download('https:' + dgsVideo, 'https:' + captionsUrl, filenamePrefix + '3dgs.mp4')
	// console.timeEnd('download 3dgs')
	// console.timeEnd('download')

	await bot.telegram.sendMessage(TARGET_CHAT, 'finished download', Extra.inReplyTo(photoMessage.message_id) as any)
}

async function getLastRunMediaObj(): Promise<any> {
	const content = await fsPromises.readFile(FILE_PATH + 'last.json', 'utf8')
		.catch(() => '{}')
	return JSON.parse(content)
}

async function saveMediaObj(mediaObj: any): Promise<void> {
	return fsPromises.writeFile(FILE_PATH + 'last.json', JSON.stringify(mediaObj, null, 2), 'utf8')
}

function parseDateToFilenamePart(date: string): string {
	const [day, month, year, hour, minute] = date.split(/[. :]/g)
	return `${year}-${month}-${day}T${hour}-${minute}`
}

async function run(): Promise<void> {
	try {
		await sendWhenNew()
	} catch (error) {
		console.log(error)
		bot.telegram.sendMessage(ERROR_TARGET, '```\n' + JSON.stringify(error, null, 2) + '\n```', Extra.markdown() as any)
	}
}

if (process.env.NODE_ENV === 'production') {
	// Dont run immediately as resilio might need time to setup
	setInterval(run, 1000 * 60 * 15) // Every 15 minutes
} else {
	run()
}