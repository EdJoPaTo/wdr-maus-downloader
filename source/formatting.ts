import {statSync} from 'fs'

export function humanReadableFilesize(path: string): string {
	const {size} = statSync(path)
	let rest = size
	let unit = 0
	while (rest > 1000) {
		rest /= 1000
		unit += 1
	}

	const unitString = ['', 'k', 'M', 'G'][unit]!
	return `${rest.toFixed(1)}${unitString}B`
}

export function captionInfoEntry(label: string | undefined, content: string | undefined): string | undefined {
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
