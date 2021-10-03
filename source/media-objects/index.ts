import {createWriteStream, readdirSync} from 'fs';

import {Api as Telegram, InputFile} from 'grammy';
import {InputMediaPhoto, InputMediaVideo} from 'grammy/out/platform';
import got from 'got';

import {addDownloaded, hasAlreadyDownloaded} from '../check-already-downloaded.js';
import {download} from '../download.js';
import {ErrorHandler, sequentialAsync} from '../generics.js';
import {FILE_PATH, PUBLIC_TARGET_CHAT, META_TARGET_CHAT} from '../constants.js';
import {humanReadableFilesize} from '../formatting.js';

import {Entry, getAll} from './currently-available.js';
import {mediaInformationFromMediaObjectJson} from './parse-media-obj.js';

export async function doit(telegram: Telegram, errorHandler: ErrorHandler) {
	const entries = await getAll(errorHandler);
	const toBeDownloaded = entries
		.filter(o => !hasAlreadyDownloaded(o.context, o.mediaObject));

	console.log('to be downloaded', toBeDownloaded.length, toBeDownloaded.map(o => o.context));

	await sequentialAsync(async o => {
		try {
			await doMediaObjectStuff(telegram, o);
		} catch (error: unknown) {
			await errorHandler(o.context, error);
		}
	}, toBeDownloaded);
}

async function doMediaObjectStuff(telegram: Telegram, {context, imageUrl, mediaObject}: Entry): Promise<void> {
	const mediaInformation = mediaInformationFromMediaObjectJson(mediaObject);
	const filenamePrefix = ['WDRMaus', context, mediaInformation.airtimeISO, mediaInformation.uniqueId, ''].join('-');

	let finalCaption = '';
	finalCaption += mediaInformation.title;
	finalCaption += '\n';
	finalCaption += mediaInformation.airtime;
	finalCaption += ' ';
	finalCaption += '#' + context;

	console.log();
	console.log();
	console.log('download now', context, 'Title:', mediaInformation.title, 'AirTime:', mediaInformation.airtime);
	console.log('image', imageUrl);
	console.log('video', mediaInformation.videoNormal);
	console.log('DGS', mediaInformation.videoDgs);
	console.log('Caption', mediaInformation.captionsSrt);

	const photoMessage = await telegram.sendPhoto(META_TARGET_CHAT, imageUrl, {disable_notification: true, caption: 'Start download...\n\n' + finalCaption});

	console.log(`start download ${context} ${mediaInformation.airtimeISO} ${mediaInformation.title}…`);
	console.time('download');

	console.time('download 1image');
	got.stream(imageUrl)
		.pipe(createWriteStream(FILE_PATH + filenamePrefix + '1image.jpg'));
	console.timeEnd('download 1image');

	console.time('download 2normal');
	await download(mediaInformation.videoNormal, mediaInformation.captionsSrt, FILE_PATH, filenamePrefix + '2normal.mp4');
	console.timeEnd('download 2normal');

	if (mediaInformation.videoDgs) {
		console.time('download 3dgs');
		await download(mediaInformation.videoDgs, mediaInformation.captionsSrt, FILE_PATH, filenamePrefix + '3dgs.mp4');
		console.timeEnd('download 3dgs');
	}

	console.timeEnd('download');

	if (process.env['TELEGRAM_API_ROOT']?.includes('http://')) {
		console.time('upload to TG');

		const media: Array<InputMediaPhoto | InputMediaVideo> = [
			{type: 'photo', media: imageUrl, caption: finalCaption},
			{type: 'video', media: new InputFile(FILE_PATH + filenamePrefix + '2normal.mp4')},
		];

		if (mediaInformation.videoDgs) {
			media.push({type: 'video', media: new InputFile(FILE_PATH + filenamePrefix + '3dgs.mp4')});
		}

		await telegram.sendMediaGroup(PUBLIC_TARGET_CHAT, media);
		console.timeEnd('upload to TG');
	}

	const relevantFiles = readdirSync(FILE_PATH)
		.filter(o => o.startsWith(filenamePrefix));

	let finishedReportMessage = 'finished download\n\n';
	finishedReportMessage += filenamePrefix + '\n';
	finishedReportMessage += relevantFiles
		.map(o => `${humanReadableFilesize(FILE_PATH + o)} ${o.slice(filenamePrefix.length)}`)
		.join('\n');

	await telegram.sendMessage(META_TARGET_CHAT, finishedReportMessage, {reply_to_message_id: photoMessage.message_id});
	addDownloaded(context, mediaObject);
}
