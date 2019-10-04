#[macro_use]
extern crate err_derive;

use reqwest::{Client, Error as ReqError};

use crate::models::{
	BeatMap,
	BeatMaps,
};

#[derive(Debug, Error)]
pub enum Error {
	#[error(display = "Unable to build client")]
	ClientBuildError(#[error(cause)] ReqError),
	#[error(display = "Unable to send request")]
	ClientSendError(#[error(cause)] ReqError),
	#[error(display = "Unable to parse request")]
	ClientJsonError(#[error(cause)] ReqError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct BeatSaber {
	client: Client,
}

impl BeatSaber {
	pub fn new() -> Result<Self> {
		let client = Client::builder()
			.build()
			.map_err(|err| Error::ClientBuildError(err))?;

		Ok(BeatSaber {
			client,
		})
	}

	pub fn maps(&self, sort_by: SortBy) -> Maps {
		Maps::new(&self.client, sort_by)
	}
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum SortBy {
	Downloads,
	Latest,
	Plays,
	Hot,
	Rating,
}

impl SortBy {
	#[inline(always)]
	pub fn path(&self) -> &'static str {
		match self {
			SortBy::Downloads => "downloads",
			SortBy::Latest => "latest",
			SortBy::Plays => "plays",
			SortBy::Hot => "hot",
			SortBy::Rating => "rating",
		}
	}
}

pub struct Maps<'client> {
	client: &'client Client,
	sort_by: SortBy,
}

fn fetch_url<T: serde::de::DeserializeOwned>(client: &Client, url: &str) -> Result<T> {
	client.get(url)
		.send()
		.map_err(|err| Error::ClientSendError(err))?
		.json()
		.map_err(|err| Error::ClientJsonError(err))
}

impl<'client> Maps<'client> {
	fn new(client: &Client, sort_by: SortBy) -> Maps {
		Maps {
			client,
			sort_by,
		}
	}

	pub fn page(&self, page: u32) -> Result<BeatMaps> {
		let path = self.sort_by.path();
		let url = format!("https://beatsaver.com/api/maps/{}/{}", path, page);

		fetch_url(self.client, &url)
	}

	pub fn iter_pages(&self) -> impl Iterator<Item = Vec<BeatMap>> {
		let path = self.sort_by.path();
		let client = self.client.clone();

		let mut page = 0;

		std::iter::from_fn(move || {
			let url = format!("https://beatsaver.com/api/maps/{}/{}", path, page);

			let BeatMaps {
				docs,
				next_page,
				..
			} = fetch_url(&client, &url).ok()?;

			page = next_page?;
			Some(docs)
		})
	}

	pub fn iter_all(&self) -> impl Iterator<Item = BeatMap> {
		self.iter_pages().flat_map(<_>::into_iter)
	}
}

pub mod models {
	use serde::Deserialize;

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct BeatMaps {
		pub docs: Vec<BeatMap>,
		pub total_docs: u32,
		pub last_page: u32,
		pub prev_page: Option<u32>,
		pub next_page: Option<u32>,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct BeatMap {
		pub metadata: Metadata,
		pub stats: Stats,
		pub description: String,
		#[serde(rename = "_id")]
		pub id: String,
		pub key: String,
		pub name: String,
		pub uploader: Uploader,
		pub uploaded: String,
		pub hash: String,
		pub direct_download: String,
		#[serde(rename = "downloadURL")]
		pub download_url: String,
		#[serde(rename = "coverURL")]
		pub cover_url: String,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Metadata {
		pub song_name: String,
		pub song_sub_name: String,
		pub song_author_name: String,
		pub level_author_name: String,

		pub bpm: f64,

		pub difficulties: Difficulties,

		pub characteristics: Vec<Characteristic>,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Difficulties {
		pub easy: bool,
		pub normal: bool,
		pub hard: bool,
		pub expert: bool,
		pub expert_plus: bool,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Characteristic {
		pub name: String,
		pub difficulties: CharacteristicDifficulties,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct CharacteristicDifficulties {
		pub easy: Option<Difficulty>,
		pub normal: Option<Difficulty>,
		pub hard: Option<Difficulty>,
		pub expert: Option<Difficulty>,
		pub expert_plus: Option<Difficulty>,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Difficulty {
		pub duration: Option<f64>,
		pub length: Option<f64>,
		pub bombs: Option<f64>,
		pub notes: Option<f64>,
		pub obstacles: Option<f64>,
		pub njs: Option<f64>,
		pub njs_offset: Option<f64>,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Stats {
		pub downloads: u64,
		pub plays: u64,

		pub down_votes: u64,
		pub up_votes: u64,

		pub heat: f64,
		pub rating: f64,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct Uploader {
		#[serde(rename = "_id")]
		pub id: String,
		pub username: String,
	}
}

