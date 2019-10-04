use beatsaber::{
	BeatSaber,
	Result,
	SortBy,
};

fn main() -> Result<()> {
	let bs = BeatSaber::new()?;

	let maps = bs.maps(SortBy::Downloads);

	for map in maps.iter_pages().take(1).flat_map(<_>::into_iter) {
		println!("{:#?}", map);
	}

	Ok(())
}
