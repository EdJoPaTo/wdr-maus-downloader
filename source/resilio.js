const {ResilioSync} = require('resilio-sync')

const {readFileSync, writeFileSync} = require('fs')

const resilio = new ResilioSync()

function sync() {
  const rslconfig = JSON.parse(readFileSync('resilio-config.json', 'utf8'))
  const share = readFileSync('/run/secrets/resilio-share.txt', 'utf8').trim()

  rslconfig.shared_folders[0].secret = share

  console.log('start resilio sync…')
  resilio.syncConfig(rslconfig, (code, signal) => {
    console.log('resilio sync stopped', code, signal)
  })
}

function stop() {
  console.log('stop resilio…')
  resilio.stop()
}

process.on('SIGINT', stop)
process.on('SIGTERM', stop)

module.exports = {
  sync
}