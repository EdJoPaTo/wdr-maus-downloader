export function humanReadableFilesize(path: string): string {
	const {size} = Deno.statSync(path)
	let rest = size
	let unit = 0
	while (rest > 1000) {
		rest /= 1000;
		unit += 1;
	}

	const unitString = ['', 'k', 'M', 'G'][unit]!;
	return `${rest.toFixed(1)}${unitString}B`;
}
