import {existsSync, readFileSync, readdirSync, statSync, createWriteStream, promises as fsPromises} from 'fs'

import request from 'request-promise-native'
import Telegraf, {Extra} from 'telegraf'

import {download} from './download'
import {parseMediaObjectJson, mediaInformationFromMediaObjectJson} from './parse-media-obj'
import {sync} from './resilio'

const TARGET_CHAT = '-1001214301516'
const ERROR_TARGET = '-1001214301516'
const FILE_PATH = 'resilio-share/'

if (process.env.NODE_ENV === 'production') {
	sync()
}

const tokenFilePath = existsSync('/run/secrets') ? '/run/secrets/bot-token.txt' : 'bot-token.txt'
const token = readFileSync(tokenFilePath, 'utf8').trim()
const bot = new Telegraf(token)

async function checkAktuelleSendung(): Promise<void> {
	const BASE_URL = 'https://www.wdrmaus.de/aktuelle-sendung/'
	const source = await request(BASE_URL)
	const imgPart = /aktuelle-sendung\/([^"]+.jpg)/.exec(source)![1]
	const img = BASE_URL + imgPart

	const mediaObjJson = await getMediaObjJson(source)

	return sendWhenNew('aktuelle-sendung', img, mediaObjJson)
}

async function checkZusatzsendung(): Promise<void> {
	const BASE_URL = 'https://www.wdrmaus.de/'
	const source = await request(BASE_URL + 'zusatzsendungen.php5')
	const imgPart = /<img src="(imggen\/.+\.jpg)/.exec(source)![1]
	const img = BASE_URL + imgPart

	const mediaObjJson = await getMediaObjJson(source)

	return sendWhenNew('zusatzsendung', img, mediaObjJson)
}

async function checkCorona(): Promise<void> {
	const BASE_URL = 'https://www.wdrmaus.de/extras/mausthemen/corona/'
	const source = await request(BASE_URL)
	const imgPart = /<img src=".+(imggen\/.+\.jpg)/.exec(source)![1]
	const img = BASE_URL + imgPart

	const mediaObjJson = await getMediaObjJson(source)
	return sendWhenNew('corona', img, mediaObjJson)
}

async function getMediaObjJson(source: string): Promise<any> {
	const mediaObjUrl = /mediaObj': { 'url': '(https:[^']+)/.exec(source)![1]
	const mediaObjString = await request(mediaObjUrl)
	return parseMediaObjectJson(mediaObjString)
}

async function sendWhenNew(context: string, img: string, mediaObjJson: any): Promise<void> {
	const last = await getLastRunMediaObj(context)
	const areEqual = JSON.stringify(last) === JSON.stringify(mediaObjJson)
	if (areEqual) {
		console.log(context + ' same as last time')
		// The last one is the same
		return
	}

	const mediaInformation = mediaInformationFromMediaObjectJson(mediaObjJson)
	const filenamePrefix = 'wdrmaus-' + context + '-' + mediaInformation.airtimeISO + '-'

	const caption = [
		captionInfoEntry(undefined, context),
		captionInfoEntry('Title', mediaInformation.title),
		captionInfoEntry('Airtime', mediaInformation.airtime),
		captionInfoEntry('Video', mediaInformation.videoNormal),
		captionInfoEntry('DGS', mediaInformation.videoDgs),
		captionInfoEntry('Caption', mediaInformation.captionsSrt)
	]
		.filter(o => o)
		.join('\n')

	const photoMessage = await bot.telegram.sendPhoto(TARGET_CHAT, img, new Extra({
		caption
	}).notifications(false) as any)

	await saveMediaObj(context, mediaObjJson)

	console.log(`start download ${context}…`)
	console.time('download')

	console.time('download 1image')
	request.get(img)
		.pipe(createWriteStream(FILE_PATH + filenamePrefix + '1image.jpg'))
	console.timeEnd('download 1image')

	console.time('download 2normal')
	await download(mediaInformation.videoNormal, mediaInformation.captionsSrt, FILE_PATH, filenamePrefix + '2normal.mp4')
	console.timeEnd('download 2normal')

	if (mediaInformation.videoDgs) {
		console.time('download 3dgs')
		await download(mediaInformation.videoDgs, mediaInformation.captionsSrt, FILE_PATH, filenamePrefix + '3dgs.mp4')
		console.timeEnd('download 3dgs')
	}

	console.timeEnd('download')

	const relevantFiles = readdirSync(FILE_PATH)
		.filter(o => o.startsWith(filenamePrefix))

	let finishedReportMessage = 'finished download\n\n'
	finishedReportMessage += relevantFiles
		.map(o => `${humanReadableFilesize(FILE_PATH + o)} ${o}`)
		.join('\n')

	await bot.telegram.sendMessage(TARGET_CHAT, finishedReportMessage, Extra.inReplyTo(photoMessage.message_id) as any)
}

function humanReadableFilesize(path: string): string {
	const {size} = statSync(path)
	let rest = size
	let unit = 0
	while (rest > 1000) {
		rest /= 1000
		unit += 1
	}

	const unitString = ['', 'k', 'M', 'G'][unit]
	return `${rest.toFixed(1)}${unitString}B`
}

function captionInfoEntry(label: string | undefined, content: string | undefined): string | undefined {
	if (!content) {
		return undefined
	}

	let result = ''
	if (label) {
		result += label
		result += ': '
	}

	result += content
	return result
}

async function getLastRunMediaObj(filename: string): Promise<any> {
	const content = await fsPromises.readFile(FILE_PATH + filename + '.json', 'utf8')
		.catch(() => '{}')
	return JSON.parse(content)
}

async function saveMediaObj(filename: string, mediaObj: any): Promise<void> {
	return fsPromises.writeFile(FILE_PATH + filename + '.json', JSON.stringify(mediaObj, null, 2), 'utf8')
}

async function handleError(error: any): Promise<void> {
	console.log(error)
	await bot.telegram.sendMessage(ERROR_TARGET, '```\n' + JSON.stringify(error, null, 2) + '\n```', Extra.markdown() as any)
}

let currentlyRunning = false
async function run(): Promise<void> {
	if (currentlyRunning) {
		return
	}

	currentlyRunning = true

	await checkAktuelleSendung().catch(handleError)
	await checkCorona().catch(handleError)
	await checkZusatzsendung().catch(handleError)

	currentlyRunning = false
}

if (process.env.NODE_ENV === 'production') {
	// Dont run immediately as resilio might need time to setup
	const interval = setInterval(run, 1000 * 60 * 15) // Every 15 minutes
	process.on('SIGINT', () => clearInterval(interval))
	process.on('SIGTERM', () => clearInterval(interval))
} else {
	run()
}
