// deno-lint-ignore-file no-explicit-any

export interface MediaInformation {
  uniqueId: string;
  title: string;
  airtime: string;
  airtimeISO: string;
  videoNormal: string;
  videoDgs?: string;
  captionsSrt?: string;
}

export function parseMediaObjectJson(source: string): any {
  const jsonPartString = source
    .replace("$mediaObject.jsonpHelper.storeAndPlay(", "")
    .slice(0, -2);
  return JSON.parse(jsonPartString);
}

export function mediaInformationFromMediaObjectJson(
  mediaObjectJson: any,
): MediaInformation {
  const uniqueId: string = mediaObjectJson.trackerData.trackerClipId;
  const title: string = mediaObjectJson.trackerData.trackerClipTitle;
  const airtime: string = mediaObjectJson.trackerData.trackerClipAirTime;
  const airtimeISO = parseAirtimeToISO(airtime);

  const videoNormal: string = httpsPrefix(
    mediaObjectJson.mediaResource.dflt.videoURL,
  )!;
  const videoDgs: string | undefined = httpsPrefix(
    mediaObjectJson.mediaResource.dflt.slVideoURL,
  );
  const captionsSrt: string | undefined = httpsPrefix(
    mediaObjectJson.mediaResource.captionsHash.srt,
  );

  return {
    uniqueId,
    title,
    airtime,
    airtimeISO,
    videoNormal,
    videoDgs,
    captionsSrt,
  };
}

function httpsPrefix(url: string | undefined): string | undefined {
  if (!url) {
    return undefined;
  }

  return "https:" + url;
}

function parseAirtimeToISO(airtime: string): string {
  const [day, month, year, hour, minute] = airtime.split(/[. :]/g);
  return `${year!}-${month!}-${day!}T${hour!}-${minute!}`;
}
