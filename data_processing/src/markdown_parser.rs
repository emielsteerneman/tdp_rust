use std::path::Path;

use anyhow::{Context, Result};
use tracing::warn;
use walkdir::WalkDir;

use data_structures::content::{Author, ContentItem, ContentType, FrontMatter, MarkdownTDP};
use data_structures::file::TDPName;
use data_structures::filter::Filter;

// ---------------------------------------------------------------------------
// Section state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Section {
    None,
    Title,
    Authors,
    Institutions,
    Mailboxes,
    Urls,
    Abstract,
    ParagraphTop,
    ParagraphTitle,
    ParagraphDepth,
    ParagraphText,
    Images,
    ImageTop,
    ImageCaption,
    ImageName,
    Tables,
    TableTop,
    TableCaption,
    TableBody,
    References,
}

// ---------------------------------------------------------------------------
// parse_markdown
// ---------------------------------------------------------------------------

pub fn parse_markdown(raw: &str, name: TDPName) -> MarkdownTDP {
    let mut front_matter = FrontMatter::default();
    let mut content_items: Vec<ContentItem> = Vec::new();
    let mut references: Vec<String> = Vec::new();

    let mut section = Section::None;

    // Current paragraph accumulator
    let mut para_title = String::new();
    let mut para_depth: u8 = 0;
    let mut para_text_lines: Vec<String> = Vec::new();

    // Image accumulator
    let mut images: Vec<(String, Option<String>)> = Vec::new(); // (caption, image_name)
    let mut current_image_caption = String::new();
    let mut current_image_name: Option<String> = None;

    // Table accumulator
    let mut tables: Vec<(String, String)> = Vec::new(); // (caption, body)
    let mut current_table_caption = String::new();
    let mut current_table_body_lines: Vec<String> = Vec::new();

    // Abstract accumulator
    let mut abstract_lines: Vec<String> = Vec::new();

    let mut content_seq: u32 = 0;

    // Helper: flush the current paragraph's text as a ContentItem
    let flush_text =
        |items: &mut Vec<ContentItem>, seq: &mut u32, lines: &mut Vec<String>, title: &str, depth: u8| {
            let body = lines.join("\n").trim().to_string();
            if !body.is_empty() {
                items.push(ContentItem {
                    content_seq: *seq,
                    content_type: ContentType::Text,
                    depth,
                    title: title.to_string(),
                    body,
                    image_path: None,
                });
                *seq += 1;
            }
            lines.clear();
        };

    let flush_images = |items: &mut Vec<ContentItem>,
                        seq: &mut u32,
                        images: &mut Vec<(String, Option<String>)>,
                        title: &str,
                        depth: u8| {
        for (caption, img_name) in images.drain(..) {
            items.push(ContentItem {
                content_seq: *seq,
                content_type: ContentType::Image,
                depth,
                title: title.to_string(),
                body: caption,
                image_path: img_name,
            });
            *seq += 1;
        }
    };

    let flush_tables =
        |items: &mut Vec<ContentItem>, seq: &mut u32, tables: &mut Vec<(String, String)>, title: &str, depth: u8| {
            for (caption, body) in tables.drain(..) {
                let full_body = if caption.is_empty() {
                    body
                } else {
                    format!("{}\n{}", caption, body)
                };
                items.push(ContentItem {
                    content_seq: *seq,
                    content_type: ContentType::Table,
                    depth,
                    title: title.to_string(),
                    body: full_body,
                    image_path: None,
                });
                *seq += 1;
            }
        };

    // Flush current image into images vec
    let flush_current_image =
        |images: &mut Vec<(String, Option<String>)>, caption: &mut String, img_name: &mut Option<String>| {
            if !caption.is_empty() || img_name.is_some() {
                images.push((caption.clone(), img_name.take()));
                caption.clear();
            }
        };

    // Flush current table into tables vec
    let flush_current_table =
        |tables: &mut Vec<(String, String)>, caption: &mut String, body_lines: &mut Vec<String>| {
            if !caption.is_empty() || !body_lines.is_empty() {
                let body = body_lines.join("\n").trim().to_string();
                tables.push((caption.clone(), body));
                caption.clear();
                body_lines.clear();
            }
        };

    // Flush the entire paragraph (text + images + tables)
    let flush_paragraph = |items: &mut Vec<ContentItem>,
                           seq: &mut u32,
                           text_lines: &mut Vec<String>,
                           images: &mut Vec<(String, Option<String>)>,
                           tables: &mut Vec<(String, String)>,
                           title: &str,
                           depth: u8| {
        flush_text(items, seq, text_lines, title, depth);
        flush_images(items, seq, images, title, depth);
        flush_tables(items, seq, tables, title, depth);
    };

    for line in raw.lines() {
        // Check for section headers
        if line == "# title" {
            section = Section::Title;
            continue;
        }
        if line == "# authors" {
            section = Section::Authors;
            continue;
        }
        if line == "# institutions" {
            section = Section::Institutions;
            continue;
        }
        if line == "# mailboxes" {
            section = Section::Mailboxes;
            continue;
        }
        if line == "# urls" {
            section = Section::Urls;
            continue;
        }
        if line == "# abstract" {
            section = Section::Abstract;
            abstract_lines.clear();
            continue;
        }
        if line == "# paragraph" {
            // Flush any pending abstract
            if section == Section::Abstract {
                let text = abstract_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    front_matter.abstract_text = Some(text);
                }
                abstract_lines.clear();
            }

            // Flush current image/table into their vecs
            flush_current_image(&mut images, &mut current_image_caption, &mut current_image_name);
            flush_current_table(&mut tables, &mut current_table_caption, &mut current_table_body_lines);

            // Flush previous paragraph
            flush_paragraph(
                &mut content_items,
                &mut content_seq,
                &mut para_text_lines,
                &mut images,
                &mut tables,
                &para_title,
                para_depth,
            );

            // Reset for new paragraph
            para_title.clear();
            para_depth = 0;
            section = Section::ParagraphTop;
            continue;
        }
        if line == "## paragraph_title" {
            section = Section::ParagraphTitle;
            continue;
        }
        if line == "## paragraph_depth" {
            section = Section::ParagraphDepth;
            continue;
        }
        if line == "## paragraph_text" {
            section = Section::ParagraphText;
            continue;
        }
        if line == "## images" {
            section = Section::Images;
            continue;
        }
        if line == "### image" {
            // Flush previous image if any
            flush_current_image(&mut images, &mut current_image_caption, &mut current_image_name);
            section = Section::ImageTop;
            continue;
        }
        if line == "#### image_caption" {
            section = Section::ImageCaption;
            continue;
        }
        if line == "#### image_name" {
            section = Section::ImageName;
            continue;
        }
        if line == "## tables" {
            section = Section::Tables;
            continue;
        }
        if line == "### table" {
            // Flush previous table if any
            flush_current_table(&mut tables, &mut current_table_caption, &mut current_table_body_lines);
            section = Section::TableTop;
            continue;
        }
        if line == "#### table_caption" {
            section = Section::TableCaption;
            continue;
        }
        if line == "#### table_body" {
            section = Section::TableBody;
            continue;
        }
        if line == "# references" {
            // Flush any pending abstract
            if section == Section::Abstract {
                let text = abstract_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    front_matter.abstract_text = Some(text);
                }
                abstract_lines.clear();
            }

            // Flush current image/table
            flush_current_image(&mut images, &mut current_image_caption, &mut current_image_name);
            flush_current_table(&mut tables, &mut current_table_caption, &mut current_table_body_lines);

            // Flush previous paragraph
            flush_paragraph(
                &mut content_items,
                &mut content_seq,
                &mut para_text_lines,
                &mut images,
                &mut tables,
                &para_title,
                para_depth,
            );

            section = Section::References;
            continue;
        }

        // Process line content based on current section
        match section {
            Section::None => {}
            Section::Title => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if front_matter.title.is_empty() {
                        front_matter.title = trimmed.to_string();
                    } else {
                        front_matter.title.push(' ');
                        front_matter.title.push_str(trimmed);
                    }
                }
            }
            Section::Authors => {
                let trimmed = line.trim();
                if let Some(name) = trimmed.strip_prefix("* ") {
                    let name = name.trim();
                    if !name.is_empty() {
                        front_matter.authors.push(Author {
                            name: name.to_string(),
                            affiliation: None,
                        });
                    }
                }
            }
            Section::Institutions => {
                let trimmed = line.trim();
                if let Some(inst) = trimmed.strip_prefix("* ") {
                    let inst = inst.trim();
                    if !inst.is_empty() {
                        front_matter.institutions.push(inst.to_string());
                    }
                }
            }
            Section::Mailboxes => {
                // Skip mailbox lines
            }
            Section::Urls => {
                let trimmed = line.trim();
                if let Some(url) = trimmed.strip_prefix("* ") {
                    let url = url.trim();
                    if !url.is_empty() {
                        front_matter.urls.push(url.to_string());
                    }
                }
            }
            Section::Abstract => {
                abstract_lines.push(line.to_string());
            }
            Section::ParagraphTop | Section::Images | Section::ImageTop | Section::Tables | Section::TableTop => {
                // Waiting for sub-section headers, ignore content
            }
            Section::ParagraphTitle => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    para_title = trimmed.to_string();
                }
            }
            Section::ParagraphDepth => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    para_depth = trimmed.parse().unwrap_or(0);
                }
            }
            Section::ParagraphText => {
                para_text_lines.push(line.to_string());
            }
            Section::ImageCaption => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if !current_image_caption.is_empty() {
                        current_image_caption.push(' ');
                    }
                    current_image_caption.push_str(trimmed);
                }
            }
            Section::ImageName => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    current_image_name = Some(trimmed.to_string());
                }
            }
            Section::TableCaption => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if !current_table_caption.is_empty() {
                        current_table_caption.push(' ');
                    }
                    current_table_caption.push_str(trimmed);
                }
            }
            Section::TableBody => {
                current_table_body_lines.push(line.to_string());
            }
            Section::References => {
                let trimmed = line.trim();
                if let Some(ref_text) = trimmed.strip_prefix("* ") {
                    let ref_text = ref_text.trim();
                    if !ref_text.is_empty() {
                        references.push(ref_text.to_string());
                    }
                }
            }
        }
    }

    // Final flush for abstract (if file ends after abstract without paragraphs)
    if section == Section::Abstract {
        let text = abstract_lines.join("\n").trim().to_string();
        if !text.is_empty() {
            front_matter.abstract_text = Some(text);
        }
    }

    // Final flush for any remaining paragraph/image/table content
    flush_current_image(&mut images, &mut current_image_caption, &mut current_image_name);
    flush_current_table(&mut tables, &mut current_table_caption, &mut current_table_body_lines);
    flush_paragraph(
        &mut content_items,
        &mut content_seq,
        &mut para_text_lines,
        &mut images,
        &mut tables,
        &para_title,
        para_depth,
    );

    MarkdownTDP {
        name,
        front_matter,
        content_items,
        references,
        raw_markdown: raw.to_string(),
    }
}

// ---------------------------------------------------------------------------
// load_all_markdown_tdps
// ---------------------------------------------------------------------------

pub fn load_all_markdown_tdps(
    root: &str,
    filter: Option<Filter>,
) -> Result<Vec<MarkdownTDP>> {
    let root_path = Path::new(root);
    anyhow::ensure!(root_path.is_dir(), "Markdown root directory does not exist: {}", root);

    let mut tdps = Vec::new();

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("md") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Could not extract file stem")?;

        let tdp_name = match TDPName::try_from(stem) {
            Ok(n) => n,
            Err(e) => {
                warn!("Skipping {}: {}", path.display(), e);
                continue;
            }
        };

        if let Some(ref f) = filter {
            if !f.matches_tdp_name(&tdp_name) {
                continue;
            }
        }

        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let tdp = parse_markdown(&raw, tdp_name);
        tdps.push(tdp);
    }

    Ok(tdps)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::content::ContentType;

    fn make_name() -> TDPName {
        TDPName::try_from("soccer_smallsize__2024__RoboTeam_Twente__0").unwrap()
    }

    #[test]
    fn test_parse_front_matter() {
        let md = "\
# title
My Paper Title
# authors
* Alice
* Bob
# institutions
* University A
* University B
# urls
* https://example.com
# abstract
This is the abstract.
It spans multiple lines.
";
        let tdp = parse_markdown(md, make_name());

        assert_eq!(tdp.front_matter.title, "My Paper Title");
        assert_eq!(tdp.front_matter.authors.len(), 2);
        assert_eq!(tdp.front_matter.authors[0].name, "Alice");
        assert_eq!(tdp.front_matter.authors[1].name, "Bob");
        assert_eq!(tdp.front_matter.institutions.len(), 2);
        assert_eq!(tdp.front_matter.institutions[0], "University A");
        assert_eq!(tdp.front_matter.institutions[1], "University B");
        assert_eq!(tdp.front_matter.urls.len(), 1);
        assert_eq!(tdp.front_matter.urls[0], "https://example.com");
        assert_eq!(
            tdp.front_matter.abstract_text.as_deref(),
            Some("This is the abstract.\nIt spans multiple lines.")
        );
    }

    #[test]
    fn test_parse_paragraphs() {
        let md = "\
# title
Test
# paragraph
## paragraph_title
1 Introduction
## paragraph_depth
1
## paragraph_text
This is the introduction text.
# paragraph
## paragraph_title
2.1 Mechanics
## paragraph_depth
2
## paragraph_text
This describes the mechanics.

With a blank line in between.
# references
* [1] Some reference
";
        let tdp = parse_markdown(md, make_name());

        assert_eq!(tdp.content_items.len(), 2);

        let item0 = &tdp.content_items[0];
        assert_eq!(item0.content_seq, 0);
        assert_eq!(item0.content_type, ContentType::Text);
        assert_eq!(item0.title, "1 Introduction");
        assert_eq!(item0.depth, 1);
        assert_eq!(item0.body, "This is the introduction text.");

        let item1 = &tdp.content_items[1];
        assert_eq!(item1.content_seq, 1);
        assert_eq!(item1.content_type, ContentType::Text);
        assert_eq!(item1.title, "2.1 Mechanics");
        assert_eq!(item1.depth, 2);
        assert!(item1.body.contains("This describes the mechanics."));
        assert!(item1.body.contains("With a blank line in between."));

        assert_eq!(tdp.references.len(), 1);
        assert_eq!(tdp.references[0], "[1] Some reference");
    }

    #[test]
    fn test_parse_images_and_tables() {
        let md = "\
# title
Test
# paragraph
## paragraph_title
3 Results
## paragraph_depth
1
## paragraph_text
Some results text.
## images
### image
#### image_caption
Figure 1: A robot
#### image_name
robot.png
### image
#### image_caption
Figure 2: Another robot
#### image_name
robot2.png
## tables
### table
#### table_caption
Table 1: Performance
#### table_body
| Team | Score |
| A | 10 |
| B | 20 |
# references
";
        let tdp = parse_markdown(md, make_name());

        // text + 2 images + 1 table = 4 items
        assert_eq!(tdp.content_items.len(), 4);

        let text = &tdp.content_items[0];
        assert_eq!(text.content_seq, 0);
        assert_eq!(text.content_type, ContentType::Text);
        assert_eq!(text.body, "Some results text.");

        let img1 = &tdp.content_items[1];
        assert_eq!(img1.content_seq, 1);
        assert_eq!(img1.content_type, ContentType::Image);
        assert_eq!(img1.body, "Figure 1: A robot");
        assert_eq!(img1.image_path.as_deref(), Some("robot.png"));
        assert_eq!(img1.depth, 1);

        let img2 = &tdp.content_items[2];
        assert_eq!(img2.content_seq, 2);
        assert_eq!(img2.content_type, ContentType::Image);
        assert_eq!(img2.body, "Figure 2: Another robot");
        assert_eq!(img2.image_path.as_deref(), Some("robot2.png"));

        let table = &tdp.content_items[3];
        assert_eq!(table.content_seq, 3);
        assert_eq!(table.content_type, ContentType::Table);
        assert!(table.body.contains("Table 1: Performance"));
        assert!(table.body.contains("| Team | Score |"));
        assert_eq!(table.depth, 1);
    }

    #[test]
    fn test_parse_real_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tdps_markdown/soccer/smallsize/2024/soccer_smallsize__2024__RoboTeam_Twente__0.md");

        if !path.exists() {
            eprintln!("Skipping test_parse_real_file: file not found at {}", path.display());
            return;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let name = TDPName::try_from("soccer_smallsize__2024__RoboTeam_Twente__0").unwrap();
        let tdp = parse_markdown(&raw, name);

        assert_eq!(
            tdp.front_matter.title,
            "RoboTeam Twente Extended Team Description Paper for RoboCup 2024"
        );
        assert!(!tdp.front_matter.authors.is_empty(), "Authors should not be empty");
        assert!(!tdp.content_items.is_empty(), "Content items should not be empty");

        // Verify content_seq is sequential: 0, 1, 2, ...
        for (i, item) in tdp.content_items.iter().enumerate() {
            assert_eq!(
                item.content_seq, i as u32,
                "content_seq should be sequential, expected {} but got {} at index {}",
                i, item.content_seq, i
            );
        }

        // At least one Table and one Image
        let has_table = tdp.content_items.iter().any(|i| i.content_type == ContentType::Table);
        let has_image = tdp.content_items.iter().any(|i| i.content_type == ContentType::Image);
        assert!(has_table, "Should have at least one Table content type");
        assert!(has_image, "Should have at least one Image content type");
    }
}
