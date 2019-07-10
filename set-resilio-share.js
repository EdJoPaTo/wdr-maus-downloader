const {readFileSync, writeFileSync} = require('fs')

function doit() {
  const rslconfig = JSON.parse(readFileSync('resilio-config.json', 'utf8'))
  const share = readFileSync('/run/secrets/resilio-share.txt', 'utf8').trim()

  rslconfig.shared_folders[0].secret = share

  writeFileSync('resilio-config.json', JSON.stringify(rslconfig, null, 2), 'utf8')
}

doit()
