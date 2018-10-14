const fs = require('fs')
const fsPromises = require('fs').promises

const request = require('request-promise-native')
const Telegraf = require('telegraf')

const {download} = require('./download')

const {Extra} = Telegraf

const TARGET_CHAT = '-1001214301516'
const ERROR_TARGET = '-1001214301516'
const BASE_URL = 'https://www.wdrmaus.de/aktuelle-sendung/'
const FILE_PATH = './tmp/'

const tokenFilePath = process.env.NODE_ENV === 'production' ? process.env.npm_package_config_tokenpath : process.env.npm_package_config_tokenpathdebug
const token = fs.readFileSync(tokenFilePath, 'utf8').trim()
const bot = new Telegraf(token)

async function sendWhenNew() {
  const source = await request(BASE_URL)
  const imgPart = /aktuelle-sendung\/([^"]+.jpg)/.exec(source)[1]
  const img = BASE_URL + imgPart

  const mediaObjUrl = /mediaObj': { 'url': '(https:[^']+)/.exec(source)[1]
  const mediaObjString = await request(mediaObjUrl)
  const mediaObjJsonString = mediaObjString
    .replace('$mediaObject.jsonpHelper.storeAndPlay(', '')
    .slice(0, -2)
  const mediaObjJson = JSON.parse(mediaObjJsonString)

  const date = mediaObjJson.trackerData.trackerClipAirTime

  const last = await getLastRunMediaObj()
  const areEqual = JSON.stringify(last) === JSON.stringify(mediaObjJson)
  if (areEqual) {
    console.log('same as last time')
    // The last one is the same
    return
  }

  const photoMessage = await bot.telegram.sendPhoto(TARGET_CHAT, img, new Extra({
    caption: date
  }).notifications(false))

  await saveMediaObj(mediaObjJson)

  const {videoURL: normalVideo, slVideoURL: dgsVideo} = mediaObjJson.mediaResource.dflt
  const captionsUrl = mediaObjJson.mediaResource.captionsHash.srt

  console.time('download')
  await Promise.all([
    request.get(img)
      .pipe(fs.createWriteStream(FILE_PATH + '1image.jpg')),
    download('https:' + normalVideo, 'https:' + captionsUrl, FILE_PATH + '2normal.mp4')
    // Disable temporarily
    // download('https:' + dgsVideo, 'https:' + captionsUrl, FILE_PATH + '3dgs.mp4')
  ])
  console.timeEnd('download')

  let caption = date
  caption += '\nVideo: https:' + normalVideo
  caption += '\nDGS: https:' + dgsVideo
  caption += '\nUntertitel: https:' + captionsUrl

  await bot.telegram.sendMessage(TARGET_CHAT, caption, Extra.inReplyTo(photoMessage.message_id))
}

async function getLastRunMediaObj() {
  const content = await fsPromises.readFile(FILE_PATH + 'last.json', 'utf8')
    .catch(() => '{}')
  return JSON.parse(content)
}

function saveMediaObj(mediaObj) {
  return fsPromises.writeFile(FILE_PATH + 'last.json', JSON.stringify(mediaObj, null, 2), 'utf8')
}

async function run() {
  try {
    await sendWhenNew()
  } catch (error) {
    console.log(error)
    bot.telegram.sendMessage(ERROR_TARGET, '```\n' + JSON.stringify(error, null, 2) + '\n```', Extra.markdown())
  }
}
run()
if (process.env.NODE_ENV === 'production') {
  setInterval(run, 1000 * 60 * 15) // Every 15 minutes
}
