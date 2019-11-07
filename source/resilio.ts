import {readFileSync} from 'fs'

import {ResilioSync} from 'resilio-sync'

const resilio = new ResilioSync()

export async function sync(): Promise<void> {
	const rslconfig = JSON.parse(readFileSync('resilio-config.json', 'utf8'))
	const share = readFileSync('/run/secrets/resilio-share.txt', 'utf8').trim()

	rslconfig.shared_folders[0].secret = share

	console.log('start resilio sync…')
	await resilio.syncConfig(rslconfig, (code, signal) => {
		console.log('resilio sync stopped', code, signal)
	})
}

function stop(): void {
	console.log('stop resilio…')
	resilio.stop()
}

process.on('SIGINT', stop)
process.on('SIGTERM', stop)
