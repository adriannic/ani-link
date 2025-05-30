// fn try_open<T: Scraper>(mirrors: &[String]) -> bool {
//     let viewable = mirrors
//         .iter()
//         .filter(|mirror| WHITELIST.iter().any(|elem| mirror.contains(elem)))
//         .collect_vec();
//
//     println!("Intentando abrir en mpv...");
//     let success = viewable.iter().any(|mirror| {
//         println!("Intentando {}...", mirror);
//
//         let mut command = if cfg!(target_os = "windows") {
//             Command::new("mpv.exe")
//         } else {
//             Command::new("mpv")
//         };
//
//         command
//             .arg(mirror)
//             .output()
//             .ok()
//             .and_then(|output| output.status.code().filter(|&code| code == 0))
//             .is_some()
//     });
//     success
// }
//
// async fn get_episodes<T: Scraper>(
//     client: &Client,
//     anime: &Anime,
// ) -> Result<Vec<usize>, Box<dyn Error>> {
//     let episodes = T::try_get_episodes(client, &anime.url).await?;
//     let episode_index = FuzzySelect::new()
//         .with_prompt("Elige un episodio")
//         .items(&episodes)
//         .interact()?;
//     let episodes = episodes.iter().skip(episode_index).copied().collect_vec();
//     Ok(episodes)
// }
//
// async fn select_anime<T: Scraper>(client: &Client) -> Result<Anime, Box<dyn Error>> {
//     let query: String = Input::new().with_prompt("Buscar anime").interact()?;
//     let animes = T::try_search(client, &query).await?;
//     if animes.is_empty() {
//         eprintln!("No se ha encontrado ningún anime.");
//         exit(1);
//     }
//     let display_anime = animes.iter().map(|anime| anime.name.as_str()).collect_vec();
//     let anime_index = FuzzySelect::new()
//         .with_prompt("Elige un anime")
//         .items(&display_anime)
//         .interact()?;
//     let anime = animes[anime_index].clone();
//     Ok(anime)
// }
