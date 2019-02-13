extern crate failure;
extern crate rustic_core as rustic;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate soundcloud;

mod error;
mod playlist;
mod track;

use failure::Error;
use rustic::library::{Playlist, SharedLibrary, Track};
use rustic::provider;
use std::str::FromStr;

use track::SoundcloudTrack;

#[derive(Debug, Clone, Deserialize)]
pub struct SoundcloudProvider {
    client_id: String,
    auth_token: Option<String>,
}

impl SoundcloudProvider {
    fn client(&self) -> soundcloud::Client {
        let mut client = soundcloud::Client::new(self.client_id.as_str());
        if self.auth_token.is_some() {
            let token = self.auth_token.clone().unwrap();
            client.authenticate_with_token(token);
        }
        client
    }
}

impl provider::ProviderInstance for SoundcloudProvider {
    fn setup(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn title(&self) -> &'static str {
        "Soundcloud"
    }

    fn uri_scheme(&self) -> &'static str {
        "soundcloud"
    }

    fn sync(&mut self, library: SharedLibrary) -> Result<provider::SyncResult, Error> {
        let client = self.client();
        let mut playlists: Vec<Playlist> = client
            .playlists()?
            .iter()
            .cloned()
            .map(|playlist| playlist::SoundcloudPlaylist::from(playlist, &self.client_id))
            .map(Playlist::from)
            .collect();
        library.sync_playlists(&mut playlists)?;
        Ok(provider::SyncResult {
            tracks: 0,
            albums: 0,
            artists: 0,
            playlists: playlists.len(),
        })
    }
    fn root(&self) -> provider::ProviderFolder {
        provider::ProviderFolder {
            folders: vec!["Likes".to_owned()],
            items: vec![],
        }
    }
    fn navigate(&self, path: Vec<String>) -> Result<provider::ProviderFolder, Error> {
        match path[0].as_str() {
            "Likes" => {
                let client = self.client();
                let likes = client.likes()?;
                let items = likes
                    .iter()
                    .cloned()
                    .filter(|like| like.track.is_some() || like.playlist.is_some())
                    .map(|like| (like.track, like.playlist))
                    .map(|like| match like {
                        (Some(track), _) => {
                            provider::ProviderItem::from(Track::from(SoundcloudTrack::from(track)))
                        }
                        (_, Some(playlist)) => provider::ProviderItem::from(Playlist::from(
                            playlist::SoundcloudPlaylist::from(playlist, &self.client_id),
                        )),
                        _ => panic!("something went horribly wrong"),
                    }).collect();
                let folder = provider::ProviderFolder {
                    folders: vec![],
                    items,
                };
                Ok(folder)
            }
            _ => Err(Error::from(provider::NavigationError::PathNotFound)),
        }
    }
    fn search(&self, query: String) -> Result<Vec<provider::ProviderItem>, Error> {
        trace!("search {}", query);
        let client = self.client();
        let results = client
            .tracks()
            .query(Some(query))
            .get()?
            .unwrap_or_else(|| vec![])
            .into_iter()
            .filter(|track| track.stream_url.is_some())
            .map(SoundcloudTrack::from)
            .map(|track| track.into())
            .collect();
        Ok(results)
    }
    fn resolve_track(&self, uri: &str) -> Result<Option<Track>, Error> {
        let id = &uri["soundcloud://".len()..];
        let id = usize::from_str(id)?;
        let client = self.client();
        let track = client
            .tracks()
            .id(id)
            .get()
            .ok()
            .map(|mut track| {
                track.stream_url = track
                    .stream_url
                    .map(|url| format!("{}?client_id={}", url, self.client_id.clone()));
                track
            }).map(SoundcloudTrack::from)
            .map(Track::from);
        Ok(track)
    }
}
