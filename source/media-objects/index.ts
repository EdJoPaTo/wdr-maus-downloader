import {createWriteStream, readdirSync} from 'fs'

import got from 'got'
import {Extra, Telegram} from 'telegraf'

import {addDownloaded, hasAlreadyDownloaded} from '../check-already-downloaded'
import {captionInfoEntry, humanReadableFilesize} from '../formatting'
import {download} from '../download'
import {ErrorHandler, sequentialAsync} from '../generics'
import {FILE_PATH, TARGET_CHAT} from '../constants'

import {Entry, getAll} from './currently-available'
import {mediaInformationFromMediaObjectJson} from './parse-media-obj'

export async function doit(telegram: Telegram, errorHandler: ErrorHandler) {
	const entries = await getAll(errorHandler)
	const toBeDownloaded = entries
		.filter(o => !hasAlreadyDownloaded(o.context, o.mediaObject))

	console.log('to be downloaded', toBeDownloaded.length, toBeDownloaded.map(o => o.context))

	await sequentialAsync(async o => {
		try {
			await doMediaObjectStuff(telegram, o)
		} catch (error) {
			await errorHandler(o.context, error)
		}
	}, toBeDownloaded)
}

async function doMediaObjectStuff(telegram: Telegram, {context, imageUrl, mediaObject}: Entry): Promise<void> {
	const mediaInformation = mediaInformationFromMediaObjectJson(mediaObject)
	const filenamePrefix = ['WDRMaus', context, mediaInformation.airtimeISO, mediaInformation.uniqueId, ''].join('-')

	let finalCaption = ''
	finalCaption += mediaInformation.title
	finalCaption += '\n'
	finalCaption += mediaInformation.airtime
	finalCaption += ' '
	finalCaption += '#' + context

	const captionLines = [
		captionInfoEntry(undefined, context),
		captionInfoEntry('Title', mediaInformation.title),
		captionInfoEntry('Airtime', mediaInformation.airtime),
		captionInfoEntry('Video', mediaInformation.videoNormal),
		captionInfoEntry('DGS', mediaInformation.videoDgs),
		captionInfoEntry('Caption', mediaInformation.captionsSrt)
	]
		.filter(o => o)
		.join('\n')

	let caption = ''
	caption += finalCaption
	caption += '\n\n'
	caption += captionLines

	const photoMessage = await telegram.sendPhoto(TARGET_CHAT, imageUrl, new Extra({
		caption
	}).notifications(false) as any)

	console.log(`start download ${context} ${mediaInformation.airtimeISO} ${mediaInformation.title}…`)
	console.time('download')

	console.time('download 1image')
	got.stream(imageUrl)
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
	finishedReportMessage += filenamePrefix + '\n'
	finishedReportMessage += relevantFiles
		.map(o => `${humanReadableFilesize(FILE_PATH + o)} ${o.slice(filenamePrefix.length)}`)
		.join('\n')

	await telegram.sendMessage(TARGET_CHAT, finishedReportMessage, Extra.inReplyTo(photoMessage.message_id) as any)
	addDownloaded(context, mediaObject)
}
