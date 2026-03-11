use std::sync::Arc;

use data_access::activity::ActivityClient;
use data_access::metadata::MetadataClient;
use data_structures::intermediate::SectionResult;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::activity::{EventSource, log_activity};
use crate::error::ApiError;
use crate::paper_navigation::{compute_breadcrumbs, compute_section_range};

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetSectionArgs {
    #[schemars(
        description = "The lyti identifier of the paper (e.g. 'soccer_smallsize__2024__RoboTeam_Twente__0')"
    )]
    pub paper: String,
    #[schemars(description = "The content sequence number from search results or get_table_of_contents")]
    pub content_seq: u32,
    #[schemars(
        description = "If true (default), returns the section and all its subsections. If false, returns only the single content item."
    )]
    pub include_children: Option<bool>,
}

pub async fn get_section(
    metadata_client: Arc<dyn MetadataClient>,
    args: GetSectionArgs,
    activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    source: EventSource,
) -> Result<SectionResult, ApiError> {
    let include_children = args.include_children.unwrap_or(true);

    // Load ToC for breadcrumbs
    let toc = metadata_client
        .load_toc(args.paper.clone())
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let breadcrumbs = compute_breadcrumbs(&toc, args.content_seq);

    // Load content items
    let items = if include_children {
        let (start, end) = compute_section_range(&toc, args.content_seq).ok_or_else(|| {
            ApiError::Internal(format!(
                "content_seq {} not found in ToC for {}",
                args.content_seq, args.paper
            ))
        })?;
        metadata_client
            .load_content_items_range(args.paper.clone(), start, end)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?
    } else {
        let item = metadata_client
            .load_content_item(args.paper.clone(), args.content_seq)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        vec![item]
    };

    log_activity(
        activity_client,
        source,
        "get_section",
        serde_json::json!({
            "paper": args.paper,
            "content_seq": args.content_seq,
            "include_children": include_children,
            "items_returned": items.len(),
        }),
    );

    Ok(SectionResult {
        lyti: args.paper,
        breadcrumbs,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_access::metadata::MockMetadataClient;
    use data_structures::content::{ContentItem, ContentType, TocEntry};

    fn sample_toc() -> Vec<TocEntry> {
        vec![
            TocEntry {
                content_seq: 0,
                content_type: ContentType::Text,
                depth: 1,
                title: "Vision".to_string(),
            },
            TocEntry {
                content_seq: 1,
                content_type: ContentType::Text,
                depth: 2,
                title: "Camera".to_string(),
            },
            TocEntry {
                content_seq: 2,
                content_type: ContentType::Text,
                depth: 3,
                title: "Mirror".to_string(),
            },
            TocEntry {
                content_seq: 3,
                content_type: ContentType::Table,
                depth: 3,
                title: "Specs".to_string(),
            },
            TocEntry {
                content_seq: 4,
                content_type: ContentType::Text,
                depth: 2,
                title: "Detection".to_string(),
            },
        ]
    }

    fn sample_section_items() -> Vec<ContentItem> {
        vec![
            ContentItem {
                content_seq: 1,
                content_type: ContentType::Text,
                depth: 2,
                title: "Camera".to_string(),
                body: "Camera description.".to_string(),
                image_path: None,
            },
            ContentItem {
                content_seq: 2,
                content_type: ContentType::Text,
                depth: 3,
                title: "Mirror".to_string(),
                body: "Mirror details.".to_string(),
                image_path: None,
            },
            ContentItem {
                content_seq: 3,
                content_type: ContentType::Table,
                depth: 3,
                title: "Specs".to_string(),
                body: "| Spec | Value |".to_string(),
                image_path: None,
            },
        ]
    }

    #[tokio::test]
    async fn test_get_section_with_children() {
        let lyti = "soccer_smallsize__2024__Test__0".to_string();
        let mut mock = MockMetadataClient::new();

        let toc = sample_toc();
        let items = sample_section_items();

        let lyti_c = lyti.clone();
        mock.expect_load_toc()
            .withf(move |l| *l == lyti_c)
            .returning(move |_| {
                let t = toc.clone();
                Box::pin(std::future::ready(Ok(t)))
            });

        let lyti_c = lyti.clone();
        mock.expect_load_content_items_range()
            .withf(move |l, start, end| *l == lyti_c && *start == 1 && *end == 4)
            .returning(move |_, _, _| {
                let i = items.clone();
                Box::pin(std::future::ready(Ok(i)))
            });

        let result = get_section(
            Arc::new(mock),
            GetSectionArgs {
                paper: lyti.clone(),
                content_seq: 1,
                include_children: Some(true),
            },
            None,
            EventSource::Dev,
        )
        .await
        .unwrap();

        assert_eq!(result.lyti, lyti);
        assert_eq!(result.breadcrumbs.len(), 1);
        assert_eq!(result.breadcrumbs[0].title, "Vision");
        assert_eq!(result.items.len(), 3);
    }

    #[tokio::test]
    async fn test_get_section_without_children() {
        let lyti = "soccer_smallsize__2024__Test__0".to_string();
        let mut mock = MockMetadataClient::new();

        let toc = sample_toc();

        let lyti_c = lyti.clone();
        mock.expect_load_toc()
            .withf(move |l| *l == lyti_c)
            .returning(move |_| {
                let t = toc.clone();
                Box::pin(std::future::ready(Ok(t)))
            });

        let lyti_c = lyti.clone();
        mock.expect_load_content_item()
            .withf(move |l, seq| *l == lyti_c && *seq == 2)
            .returning(move |_, _| {
                let item = ContentItem {
                    content_seq: 2,
                    content_type: ContentType::Text,
                    depth: 3,
                    title: "Mirror".to_string(),
                    body: "Mirror details.".to_string(),
                    image_path: None,
                };
                Box::pin(std::future::ready(Ok(item)))
            });

        let result = get_section(
            Arc::new(mock),
            GetSectionArgs {
                paper: lyti.clone(),
                content_seq: 2,
                include_children: Some(false),
            },
            None,
            EventSource::Dev,
        )
        .await
        .unwrap();

        assert_eq!(result.breadcrumbs.len(), 2);
        assert_eq!(result.breadcrumbs[0].title, "Vision");
        assert_eq!(result.breadcrumbs[1].title, "Camera");
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "Mirror");
    }
}
