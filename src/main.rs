use cumulus::{cerrln, cinfoln};
use fortnite_itemshop_discord_webhook::App;
use serenity::model::{channel::Embed, webhook::Webhook};
use tokio::main;

// Lookup values for the skins we want to find
const SKINS: [&str; 3] = [
    "CID_504_Athena_Commando_M_Lopex", // Fennix
    "Character_SirWolf",               // Wendell
    "Character_FeralTrash",            // Rufus
];

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let webhook_id = std::env::var("DISCORD_WEBHOOK_ID")?;
    let webhook_token = std::env::var("DISCORD_WEBHOOK_TOKEN")?;
    let mut app = App::new();

    // Create a new regex object to match the names of the items we want to find
    let skins_str = SKINS.join("|");
    let re = regex::Regex::new(format!(r"({})", skins_str).as_str())?;

    let client = reqwest::Client::new();
    let mut reset_time = chrono::Utc::now();

    loop {
        if reset_time - chrono::Utc::now() > chrono::Duration::seconds(0) {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            continue;
        }

        let req = client
            .get("https://fortnite-api.com/v2/shop/br?language=en")
            .send()
            .await?;

        if !req.status().is_success() {
            cerrln!("Failed to get shop data!");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        cinfoln!("Shop reset!");

        let data = req.text().await?;

        let body: serde_json::Value = serde_json::from_str(&data)?;

        // in-case the hash is the same, wait for the new one
        if app.hash == body["data"]["hash"].as_str().unwrap().to_string() {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        let reset = chrono::DateTime::parse_from_rfc3339(
            &body["data"]["date"].as_str().unwrap().to_string(),
        )?;

        let reset = reset.with_timezone(&chrono::Utc);

        let reset = reset + chrono::Duration::hours(24);

        reset_time = reset;

        let now = chrono::Utc::now();

        let duration = reset - now;

        cinfoln!(
            "Shop will reset in {} hour{}, {} minute{}, {} second{}",
            duration.num_hours(),
            if duration.num_hours() == 1 { "" } else { "s" },
            duration.num_minutes() % 60,
            if duration.num_minutes() == 1 { "" } else { "s" },
            duration.num_seconds() % 60,
            if duration.num_seconds() == 1 { "" } else { "s" }
        );

        app.hash = body["data"]["hash"].as_str().unwrap().to_string();

        let items = body["data"]["featured"]["entries"].as_array().unwrap();

        cinfoln!("Featured Items: {}", items.len());

        for item in items {
            let name = item["items"][0]["name"].as_str().unwrap();
            let price = item["finalPrice"].as_u64().unwrap();
            let image = item["items"][0]["images"]["icon"].as_str().unwrap();
            let id = item["items"][0]["id"].as_str().unwrap();

            cinfoln!("Checking {} ({})...", name, id);

            SKINS.iter().for_each(|skin| {
                if re.is_match(name) {
                    cinfoln!("Found {} ({})!", name, id);
                    app.date = body["data"]["date"]
                        .as_str()
                        .unwrap_or("No date")
                        .to_string();
                }
            });
        }

        let embed = Embed::fake(|e| {
            e.title("Featured Items").fields(
                items
                    .iter()
                    .map(|item| {
                        let name = item["items"][0]["name"].as_str().unwrap();
                        let price = item["finalPrice"].as_u64().unwrap();
                        let image = item["items"][0]["images"]["icon"].as_str().unwrap();
                        let id = item["items"][0]["id"].as_str().unwrap();

                        (name, format!("{} V-Bucks", price), false)
                    })
                    .collect::<Vec<(&str, String, bool)>>(),
            )
        });

        let discord_client = serenity::http::Http::new("");

        let webhook = Webhook::from_id_with_token(
            &discord_client,
            // parse id
            webhook_id.parse::<u64>().unwrap(),
            // parse token
            &webhook_token,
        )
        .await?;

        let mut webhook_message = String::new();

        // check if our skins are in
        if re.is_match(&data) {
            webhook_message.push_str("🐺🐺🐺 OWO FURRYBAIT ALERT 🐺🐺🐺");

            webhook
                .execute(discord_client, false, |w| {
                    w.username("Fortnite Item Shop")
                        .content(format!("{}", webhook_message))
                        .avatar_url(
                            "https://image.fnbr.co/outfit/65d72eaead43777eae1352a9/icon.png",
                        )
                        .embeds(vec![embed])
                })
                .await?;
        }

        // sleep for 1 second
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
