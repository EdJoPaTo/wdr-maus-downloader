import * as childProcess from 'child_process';
import {promisify} from 'util';

const exec = promisify(childProcess.exec);

function downloadCommandLine(video: string, captions: string | undefined, targetfile: string): string {
	let command = 'nice ffmpeg -y -v error';
	command += ` -i "${video}"`;
	if (captions) {
		command += ` -i "${captions}"`;
	}

	command += ' -c copy';
	command += ' -c:s mov_text';
	command += ' -codec:v h264';
	if (process.env['NODE_ENV'] !== 'production') {
		// Only 15 seconds for faster finish
		command += ' -t 0:05';
	}

	command += ` "${targetfile}"`;
	return command;
}

export async function download(video: string, captions: string | undefined, targetfolder: string, filename: string) {
	const temporaryFile = 'tmp/' + filename;
	const command = downloadCommandLine(video, captions, temporaryFile);
	const result = await exec(command);
	await exec(`mv ${temporaryFile} ${targetfolder + filename}`);
	return result;
}
