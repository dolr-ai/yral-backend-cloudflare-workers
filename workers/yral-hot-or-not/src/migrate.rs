use std::collections::HashMap;

use worker::*;
use yral_metadata_client::MetadataClient;

use crate::hon_game::UserHonGameState;

impl UserHonGameState {
    pub async fn migrate_games_to_user_principal_key(&self) -> Result<()> {
        let mut storage = self.storage();
        let schema_version = *self.schema_version.borrow_mut().read(&storage).await?;
        if schema_version >= 1 {
            return Ok(());
        }

        self.ensure_games_loaded().await?;
        let games = self.games.borrow().as_ref().unwrap().clone();
        let canister_ids: Vec<_> = games.keys().map(|(canister_id, _)| *canister_id).collect();

        let metadata_client: MetadataClient<false> = MetadataClient::default();
        let principals = metadata_client
            .get_canister_to_principal_bulk(canister_ids)
            .await
            .map_err(|e| {
                console_error!("failed to get principals from metadata: {:?}", e);
                Error::from("failed to get principals from metadata")
            })?;

        let mut games_by_user_principal = HashMap::new();
        for ((canister_id, post_id), game_info) in games {
            if let Some(principal_id) = principals.get(&canister_id) {
                games_by_user_principal.insert((*principal_id, post_id), game_info);
            }
        }

        for ((principal_id, post_id), game_info) in games_by_user_principal {
            storage
                .put(
                    &format!("games_by_user_principal-{principal_id}-{post_id}"),
                    &game_info,
                )
                .await?;
        }

        self.schema_version
            .borrow_mut()
            .update(&mut storage, |v| *v = 1)
            .await?;

        Ok(())
    }
}
