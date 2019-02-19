use rustic::library::{Artist, Track};
use rustic::provider;
use soundcloud;

pub const META_SOUNDCLOUD_USER_ID: &'static str = "SOUNDCLOUD_USER_ID";
pub const META_SOUNDCLOUD_TRACK_ID: &'static str = "SOUNDCLOUD_TRACK_ID";

#[derive(Debug, Clone)]
pub struct SoundcloudTrack(soundcloud::Track);

impl From<SoundcloudTrack> for Track {
    fn from(track: SoundcloudTrack) -> Track {
        let track = track.0;

        Track {
            id: None,
            title: track.title,
            artist: Some(Artist {
                id: None,
                name: track.user.username,
                image_url: Some(track.user.avatar_url),
                uri: format!("soundcloud://user/{}", track.user.id),
                meta: hashmap!(
                    META_SOUNDCLOUD_USER_ID => track.user.id.into()
                )
            }),
            artist_id: None,
            album: None,
            album_id: None,
            stream_url: track.stream_url.unwrap(),
            provider: provider::Provider::Soundcloud,
            uri: format!("soundcloud://track/{}", track.id),
            image_url: track.artwork_url,
            duration: Some(track.duration),
            meta: hashmap!{
                META_SOUNDCLOUD_TRACK_ID => track.id.into()
            }
        }
    }
}

impl From<SoundcloudTrack> for provider::ProviderItem {
    fn from(track: SoundcloudTrack) -> provider::ProviderItem {
        provider::ProviderItem::from(Track::from(track))
    }
}

impl From<soundcloud::Track> for SoundcloudTrack {
    fn from(track: soundcloud::Track) -> Self {
        SoundcloudTrack(track)
    }
}
