# WDR Maus Downloader

This tools checks for a new episode of the ["Maus"](https://wdrmaus.de) and downloads it.
It sends a notification to an update telegram channel when finished.

Without a selfhosted tdbotapi the file limit for bots is 50 MB which is not enough for the episodes.
The environment variable `TELEGRAM_API_ROOT` is used for a selfhosted tdbotapi like [tdlight-telegram-bot-api](https://github.com/tdlight-team/tdlight-telegram-bot-api)
When the variable is configured the episodes are automatically uploaded to the public channel [@wdrMaus](https://t.me/wdrMaus).

The files are added to the exposed VOLUME `/app/files` and are not automatically cleaned up.

## Disclaimer

I have nothing to do with WDR, "Das Erste" or the "Die Maus" team.
I just watch the "Maus" sometimes.

## Run it

This is not meant for everyone to run.
Mainly this is my own state of the sources.

So this is not documented well.
