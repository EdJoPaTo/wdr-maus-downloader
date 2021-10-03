const debuggingChannel = '-1001149205144';
const mausPublicChannel = '-1001155474248';
const mausUpdatesChannel = '-1001214301516';

export const PUBLIC_TARGET_CHAT = Deno.env.get('NODE_ENV') === 'production' ? mausPublicChannel : debuggingChannel
export const META_TARGET_CHAT = Deno.env.get('NODE_ENV') === 'production' ? mausUpdatesChannel : debuggingChannel
export const FILE_PATH = 'files/'
