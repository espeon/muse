use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::api::{
    admin::RescanResponse,
    artist::AllArtistsPartial,
    home::{HomeRow, HomeRowType},
    index::{GenreEntry, IndexSong, SearchSong},
    me::MeResponse,
    playlist::{
        AddTrackRequest, CreatePlaylistRequest, PlaylistDetail, PlaylistSummary, PlaylistTrack,
        ReorderTrackRequest, UpdatePlaylistRequest,
    },
    sign::{BatchSignRequest, SignResult},
    song::{
        LikedResponse, MixProfileResponse, PlayHistoryEntry, SimilarTrack, TrackListItem,
        TracksResponse,
    },
    Album, AlbumPartial, AllAlbumsPartial, Artist, ArtistPartial, Track,
};

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Maki", version = "0.1.0", description = "Maki music server API"),
    modifiers(&BearerAuth),
    paths(
        crate::api::me::get_me,
        crate::api::admin::post_rescan,
        crate::api::admin::post_analyze,
        crate::api::album::get_album,
        crate::api::album::get_albums,
        crate::api::artist::get_artist,
        crate::api::artist::get_artists,
        crate::api::song::get_song,
        crate::api::song::get_similar_songs,
        crate::api::song::get_mix_profile,
        crate::api::song::like_song,
        crate::api::song::scrobble_song,
        crate::api::song::set_playing,
        crate::api::song::get_history,
        crate::api::song::get_tracks,
        crate::api::sign::sign_track_url,
        crate::api::sign::batch_sign_track_urls,
        crate::api::index::index_songs,
        crate::api::index::search_songs,
        crate::api::index::get_genres,
        crate::api::home::home,
        crate::api::playlist::list_playlists,
        crate::api::playlist::create_playlist,
        crate::api::playlist::get_playlist,
        crate::api::playlist::update_playlist,
        crate::api::playlist::delete_playlist,
        crate::api::playlist::add_track,
        crate::api::playlist::remove_track,
        crate::api::playlist::reorder_track,
    ),
    components(schemas(
        Track,
        Album,
        AlbumPartial,
        AllAlbumsPartial,
        Artist,
        ArtistPartial,
        AllArtistsPartial,
        TrackListItem,
        TracksResponse,
        SimilarTrack,
        MixProfileResponse,
        LikedResponse,
        PlayHistoryEntry,
        SignResult,
        BatchSignRequest,
        MeResponse,
        RescanResponse,
        IndexSong,
        SearchSong,
        GenreEntry,
        PlaylistSummary,
        PlaylistTrack,
        PlaylistDetail,
        CreatePlaylistRequest,
        UpdatePlaylistRequest,
        AddTrackRequest,
        ReorderTrackRequest,
        HomeRow,
        HomeRowType,
    ))
)]
pub struct ApiDoc;
