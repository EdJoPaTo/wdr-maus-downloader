// deno-lint-ignore-file no-explicit-any

import arrayFilterUnique from 'https://esm.sh/array-filter-unique'

import {matchAll, sequentialAsync, ErrorHandler} from '../generics.ts'

import {parseMediaObjectJson} from './parse-media-obj.ts'

export type Context = 'AktuelleSendung' | 'MausBlick' | 'Corona';
export interface Entry {
	context: Context;
	mediaObject: any;
	imageUrl: string;
}

export async function getAll(errorHandler: ErrorHandler): Promise<Entry[]> {
	return [
		...await getAktuelleSendung(errorHandler),
		...await getMausBlick(errorHandler),
		...await getCorona(errorHandler),
	];
}

async function getMediaObjectsFromSource(source: string): Promise<any[]> {
	const allMediaUrls = matchAll(/(https:[^'"]+\d+\.js)/g, source)
		.map(o => o[1]!);

	const allResponses = await sequentialAsync(async url => await (await fetch(url)).text(), allMediaUrls)

	return allResponses
		.map(o => parseMediaObjectJson(o))
}

function createEntries(context: Context, imageUrls: readonly string[], mediaObjects: readonly any[]): Entry[] {
	if (mediaObjects.length !== imageUrls.length) {
		throw new Error(`should find the same amount of images and video media objects ${imageUrls.length} != ${mediaObjects.length}`);
	}

	return imageUrls.map((o, i): Entry => ({
		context,
		imageUrl: o,
		mediaObject: mediaObjects[i],
	}));
}

async function getAktuelleSendung(errorHandler: ErrorHandler): Promise<Entry[]> {
	const context: Context = 'AktuelleSendung';
	try {
		const BASE_URL = 'https://www.wdrmaus.de/aktuelle-sendung/'
		const body = await (await fetch(BASE_URL)).text()

		const imageUrls = matchAll(/aktuelle-sendung\/([^"]+.jpg)/g, body)
			.map(o => o[1]!)
			.filter(arrayFilterUnique())
			.map(o => BASE_URL + o);

		const mediaObjects = await getMediaObjectsFromSource(body);
		return createEntries(context, imageUrls, mediaObjects);
	} catch (error: unknown) {
		await errorHandler(context, error);
		return [];
	}
}

async function getMausBlick(errorHandler: ErrorHandler): Promise<Entry[]> {
	const context: Context = 'MausBlick';
	try {
		const BASE_URL = 'https://www.wdrmaus.de/extras/mausthemen/mausblick/'
		const body = await (await fetch(BASE_URL)).text()

		const imageUrls = matchAll(/<img src="(imggen\/.+\.jpg)/g, body)
			.map(o => o[1]!)
			.map(o => BASE_URL + o);

		const mediaObjects = await getMediaObjectsFromSource(body);
		return createEntries(context, imageUrls, mediaObjects);
	} catch (error: unknown) {
		await errorHandler(context, error);
		return [];
	}
}

async function getCorona(errorHandler: ErrorHandler): Promise<Entry[]> {
	const context: Context = 'Corona';
	try {
		const BASE_URL = 'https://www.wdrmaus.de/extras/mausthemen/corona/'
		const body = await (await fetch(BASE_URL)).text()

		const imageUrls = matchAll(/<img src="..\/..\/..\/extras\/mausthemen\/corona\/(imggen\/.+\.jpg)/g, body)
			.map(o => o[1]!)
			.map(o => BASE_URL + o);

		const mediaObjects = await getMediaObjectsFromSource(body);
		return createEntries(context, imageUrls, mediaObjects);
	} catch (error: unknown) {
		await errorHandler(context, error);
		return [];
	}
}
