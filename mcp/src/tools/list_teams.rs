use std::sync::Arc;

use data_access::metadata::MetadataClient;
use data_processing::text::match_terms;
use data_structures::file::TeamName;

use crate::tools::ToolError;

pub async fn list_teams(
    metadata_client: Arc<dyn MetadataClient>,
    hint: Option<String>,
) -> Result<Vec<TeamName>, ToolError> {
    let mut teams = metadata_client
        .load_teams()
        .await
        .map_err(|err| ToolError::Internal(err.to_string()))?;

    if let Some(hint) = hint {
        let team_names = teams.iter().map(Into::into).collect();
        match_terms(team_names, hint);
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {

    struct MockMetadataClient {}
    impl MetadataClient for MockMetadataClient {}

    #[tokio::test]
    async fn test_list_teams() -> Result<_, Box<dyn std::error::Error>> {
        Ok(())
    }
}
