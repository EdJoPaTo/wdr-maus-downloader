const util = require('util')
const childProcess = require('child_process')

const exec = util.promisify(childProcess.exec)

function downloadCommandLine(video, captions, targetfile) {
  let command = 'nice ffmpeg -y -v error'
  command += ` -i "${video}"`
  command += ` -i "${captions}"`
  command += ' -c copy  -c:s mov_text'
  command += ' -codec:v h264'
  if (process.env.NODE_ENV !== 'production') {
    command += ' -t 0:15'
  }
  command += ` "${targetfile}"`
  return command
}

async function download(video, captions, targetfile) {
  const command = downloadCommandLine(video, captions, targetfile)
  const result = await exec(command)
  return result
}

module.exports = {
  download
}
