import got from 'got'
import arrayFilterUnique from 'array-filter-unique'

import {matchAll, sequentialAsync, ErrorHandler} from '../generics'
import {parseMediaObjectJson} from './parse-media-obj'

export type Context = 'AktuelleSendung' | 'MausBlickClari'
export interface Entry {
	context: Context;
	mediaObject: any;
	imageUrl: string;
}

export async function getAll(errorHandler: ErrorHandler): Promise<Entry[]> {
	return [
		...await getAktuelleSendung(errorHandler),
		...await getMausBlickClari(errorHandler)
	]
}

async function getMediaObjectsFromSource(source: string): Promise<any[]> {
	const allMediaUrls = matchAll(/(https:[^'"]+\d+\.js)/g, source)
		.map(o => o[1])

	const allResponses = await sequentialAsync(async url => got(url), allMediaUrls)

	return allResponses
		.map(o => parseMediaObjectJson(o.body))
}

async function getAktuelleSendung(errorHandler: ErrorHandler): Promise<Entry[]> {
	const context: Context = 'AktuelleSendung'
	try {
		const BASE_URL = 'https://www.wdrmaus.de/aktuelle-sendung/'
		const {body} = await got(BASE_URL)

		const imageUrls = matchAll(/aktuelle-sendung\/([^"]+.jpg)/g, body)
			.map(o => o[1])
			.filter(arrayFilterUnique())
			.map(o => BASE_URL + o)

		const mediaObjects = await getMediaObjectsFromSource(body)
		if (mediaObjects.length !== imageUrls.length) {
			throw new Error(`should find the same amount of images and video media objects ${imageUrls.length} != ${mediaObjects.length}`)
		}

		return imageUrls.map((o, i) => {
			return {
				context,
				imageUrl: o,
				mediaObject: mediaObjects[i]
			}
		})
	} catch (error) {
		await errorHandler(context, error)
		return []
	}
}

async function getMausBlickClari(errorHandler: ErrorHandler): Promise<Entry[]> {
	const context: Context = 'MausBlickClari'
	try {
		const BASE_URL = 'https://www.wdrmaus.de/extras/mausthemen/mausblick/'
		const {body} = await got(BASE_URL)

		const imageUrls = matchAll(/<img src="(imggen\/.+\.jpg)/g, body)
			.map(o => o[1])
			.map(o => BASE_URL + o)

		const mediaObjects = await getMediaObjectsFromSource(body)
		if (mediaObjects.length !== imageUrls.length) {
			throw new Error(`should find the same amount of images and video media objects ${imageUrls.length} != ${mediaObjects.length}`)
		}

		return imageUrls.map((o, i) => {
			return {
				context,
				imageUrl: o,
				mediaObject: mediaObjects[i]
			}
		})
	} catch (error) {
		await errorHandler(context, error)
		return []
	}
}
