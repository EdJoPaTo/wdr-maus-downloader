export type ErrorHandler = (context: string, error: unknown) => Promise<void>;

export function matchAll(regex: Readonly<RegExp>, text: string): ReadonlyArray<Readonly<RegExpExecArray>> {
	if (!regex.flags.includes('g')) {
		throw new Error('you probably want to set the g-lobal in the regex');
	}

	const localRegex = new RegExp(regex.source, regex.flags);
	const results: RegExpExecArray[] = [];
	let match: RegExpExecArray | null;
	while ((match = localRegex.exec(text))) {
		results.push(match);
	}

	return results;
}

export async function sequentialAsync<Argument, ReturnType>(func: (argument: Argument, index: number) => Promise<ReturnType>, missions: readonly Argument[]): Promise<readonly ReturnType[]> {
	const result: ReturnType[] = [];
	for (const mission of missions) {
		// eslint-disable-next-line no-await-in-loop
		result.push(await func(mission, result.length));
	}

	return result;
}

export async function sleep(ms: number): Promise<void> {
	await new Promise(resolve => {
		setTimeout(resolve, ms);
	});
}
