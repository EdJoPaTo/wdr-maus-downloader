const debuggingChannel = '-1001149205144'
const mausUpdatesChannel = '-1001214301516'

export const TARGET_CHAT = process.env.NODE_ENV === 'production' ? mausUpdatesChannel : debuggingChannel
export const ERROR_TARGET = process.env.NODE_ENV === 'production' ? mausUpdatesChannel : debuggingChannel
export const FILE_PATH = 'files/'
