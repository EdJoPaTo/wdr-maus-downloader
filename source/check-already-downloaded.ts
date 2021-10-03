import {readFileSync, writeFileSync, mkdirSync} from 'fs';

import jsonStableStringify from 'json-stable-stringify';

type FileContent = readonly unknown[];

const FILE_PATH = 'files/.downloaded/';
mkdirSync(FILE_PATH, {recursive: true});

function filepath(context: string): string {
	return `${FILE_PATH}${context}.json`;
}

function loadFile(context: string): FileContent {
	try {
		return JSON.parse(readFileSync(filepath(context), 'utf8')) as FileContent;
	} catch {
		return [];
	}
}

function writeFile(context: string, content: FileContent): void {
	const stringContent = jsonStableStringify(content, {space: '\t'});
	writeFileSync(filepath(context), stringContent, 'utf8');
}

export function hasAlreadyDownloaded(context: string, stuff: unknown): boolean {
	const downloaded = new Set(loadFile(context).map(o => jsonStableStringify(o)));
	const stuffString = jsonStableStringify(stuff);
	return downloaded.has(stuffString);
}

export function addDownloaded(context: string, stuff: unknown): void {
	const downloaded = new Set(loadFile(context).map(o => jsonStableStringify(o)));
	downloaded.add(jsonStableStringify(stuff));
	const content = [...downloaded].map(o => JSON.parse(o));
	writeFile(context, content);
}
