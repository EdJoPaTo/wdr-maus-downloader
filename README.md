# WDR Maus Downloader

This tools checks for a new episode of the ["Maus"](https://wdrmaus.de) and downloads it.
It sends a notification to an update telegram channel when finished.

The episode can then manually uploaded to the public channel [@wdrMaus](https://t.me/wdrMaus).
This has to be manual for now as one episode is way bigger than the [file size limit for bots (currently 50 MB)](https://core.telegram.org/bots/api#sendvideo).
Normal Users can send files up to 1.5 GB.

The file will be added to a given [Resilio Sync](https://www.resilio.com/individuals/) Share where I can upload them from later.


## Disclaimer

I have nothing to do with WDR, "Das Erste" or the "Die Maus" team.
I just watch the "Maus" sometimes.

## Run it

This is not meant for everyone to run.
Mainly this is my own state of the sources.

So this is not documented well.

### Docker Secrets that have to be set

`resilio-share.txt`: The Resilio Sync Share secret (needs write access).

`bot-token.txt`: The bot token of the Telegram Bot that is used.
