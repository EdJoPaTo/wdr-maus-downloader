const fsPromises = require('fs').promises

async function doit() {
  const rslconfig = JSON.parse(await fsPromises.readFile('resilio-config.json', 'utf8'))
  const share = (await fsPromises.readFile('/run/secrets/resilio-share.txt', 'utf8')).trim()

  rslconfig.shared_folders[0].secret = share

  await fsPromises.writeFile('resilio-config.json', JSON.stringify(rslconfig, null, 2), 'utf8')
}

doit()
