export interface TocHeading {
  id: string;
  text: string;
  level: number; // 1-6
}

/**
 * Transform custom TDP markdown into standard markdown.
 */
export function preprocessMarkdown(raw: string, lyti: string): string {
  const lines = raw.split('\n');
  const output: string[] = [];

  // Top-level section state
  type TopSection =
    | 'none'
    | 'title'
    | 'authors'
    | 'institutions'
    | 'mailboxes'
    | 'urls'
    | 'abstract'
    | 'paragraph'
    | 'references';

  // Sub-section state within a paragraph block
  type SubSection =
    | 'none'
    | 'paragraph_title'
    | 'paragraph_depth'
    | 'paragraph_text'
    | 'images'
    | 'image'
    | 'image_caption'
    | 'image_name'
    | 'tables'
    | 'table'
    | 'table_caption'
    | 'table_body';

  let topSection: TopSection = 'none';
  let subSection: SubSection = 'none';

  // Accumulated state for paragraph headings and image/table blocks
  let paragraphDepth = 1;
  let pendingParagraphTitle: string | null = null;
  let pendingImageCaption: string | null = null;
  let pendingTableCaption: string | null = null;

  // Whether we've emitted the abstract ">" prefix for this abstract block
  let inAbstract = false;

  // Frontmatter accumulators
  let frontmatterTitle: string | null = null;
  let frontmatterAuthors: string[] = [];
  let frontmatterInstitutions: string[] = [];
  let frontmatterUrls: string[] = [];
  let frontmatterEmitted = false;

  function emitFrontmatter() {
    if (frontmatterEmitted) return;
    frontmatterEmitted = true;

    if (frontmatterTitle) {
      output.push('# ' + frontmatterTitle);
      output.push('');
    }
    if (frontmatterAuthors.length > 0) {
      output.push(frontmatterAuthors.join(', '));
      output.push('');
    }
    if (frontmatterInstitutions.length > 0) {
      output.push('*' + frontmatterInstitutions.join('; ') + '*');
      output.push('');
    }
    if (frontmatterUrls.length > 0) {
      const links = frontmatterUrls.map((url) => `[${url}](${url})`);
      output.push(links.join(' · '));
      output.push('');
    }
    // Only add separator if we emitted anything
    if (frontmatterTitle || frontmatterAuthors.length > 0 || frontmatterInstitutions.length > 0 || frontmatterUrls.length > 0) {
      output.push('---');
      output.push('');
    }
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // ── Top-level section headers ──────────────────────────────────────────
    if (line === '# title') {
      topSection = 'title';
      subSection = 'none';
      continue;
    }
    if (line === '# authors') {
      topSection = 'authors';
      subSection = 'none';
      continue;
    }
    if (line === '# institutions') {
      topSection = 'institutions';
      subSection = 'none';
      continue;
    }
    if (line === '# mailboxes') {
      topSection = 'mailboxes';
      subSection = 'none';
      continue;
    }
    if (line === '# urls') {
      topSection = 'urls';
      subSection = 'none';
      continue;
    }
    if (line === '# abstract') {
      emitFrontmatter();
      topSection = 'abstract';
      subSection = 'none';
      inAbstract = false;
      continue;
    }
    if (line === '# paragraph') {
      emitFrontmatter();
      topSection = 'paragraph';
      subSection = 'none';
      paragraphDepth = 1;
      pendingParagraphTitle = null;
      continue;
    }
    if (line === '# references') {
      emitFrontmatter();
      topSection = 'references';
      subSection = 'none';
      output.push('## References');
      output.push('');
      continue;
    }

    // ── Accumulate front-matter sections ────────────────────────────────
    if (topSection === 'title') {
      if (line.trim()) frontmatterTitle = line.trim();
      continue;
    }
    if (topSection === 'authors') {
      const author = line.replace(/^\*\s*/, '').trim();
      if (author) frontmatterAuthors.push(author);
      continue;
    }
    if (topSection === 'institutions') {
      const inst = line.replace(/^\*\s*/, '').trim();
      if (inst) frontmatterInstitutions.push(inst);
      continue;
    }
    if (topSection === 'urls') {
      const url = line.replace(/^\*\s*/, '').trim();
      if (url) frontmatterUrls.push(url);
      continue;
    }
    if (topSection === 'mailboxes') {
      continue;
    }

    // ── Abstract ──────────────────────────────────────────────────────────
    if (topSection === 'abstract') {
      if (!inAbstract) {
        // First content line of abstract
        output.push('> **Abstract** ' + line);
        inAbstract = true;
      } else {
        if (line.trim() === '') {
          output.push('>');
        } else {
          output.push('> ' + line);
        }
      }
      continue;
    }

    // ── References ────────────────────────────────────────────────────────
    if (topSection === 'references') {
      output.push(line);
      continue;
    }

    // ── Paragraph blocks ──────────────────────────────────────────────────
    if (topSection === 'paragraph') {
      // Sub-section headers
      if (line === '## paragraph_title') {
        subSection = 'paragraph_title';
        continue;
      }
      if (line === '## paragraph_depth') {
        subSection = 'paragraph_depth';
        continue;
      }
      if (line === '## paragraph_text') {
        // Emit the heading now that we have both title and depth
        if (pendingParagraphTitle !== null) {
          const hashes = '#'.repeat(paragraphDepth + 1);
          output.push(hashes + ' ' + pendingParagraphTitle);
          output.push('');
          pendingParagraphTitle = null;
        }
        subSection = 'paragraph_text';
        continue;
      }
      if (line === '## images') {
        subSection = 'images';
        continue;
      }
      if (line === '## tables') {
        subSection = 'tables';
        continue;
      }

      // Sub-sub-section headers for images
      if (line === '### image') {
        subSection = 'image';
        pendingImageCaption = null;
        continue;
      }
      if (line === '#### image_caption') {
        subSection = 'image_caption';
        continue;
      }
      if (line === '#### image_name') {
        subSection = 'image_name';
        continue;
      }

      // Sub-sub-section headers for tables
      if (line === '### table') {
        subSection = 'table';
        pendingTableCaption = null;
        continue;
      }
      if (line === '#### table_caption') {
        subSection = 'table_caption';
        continue;
      }
      if (line === '#### table_body') {
        // Emit the caption before the table body
        if (pendingTableCaption !== null) {
          output.push('*' + pendingTableCaption + '*');
          output.push('');
          pendingTableCaption = null;
        }
        subSection = 'table_body';
        continue;
      }

      // Content lines by sub-section
      switch (subSection) {
        case 'paragraph_title':
          pendingParagraphTitle = line;
          subSection = 'none';
          break;

        case 'paragraph_depth': {
          const d = parseInt(line.trim(), 10);
          if (!isNaN(d)) {
            paragraphDepth = d;
          }
          subSection = 'none';
          break;
        }

        case 'paragraph_text':
          output.push(line);
          break;

        case 'image_caption':
          pendingImageCaption = line;
          subSection = 'none';
          break;

        case 'image_name': {
          const filename = line.trim();
          const alt = pendingImageCaption ?? filename;
          const caption = pendingImageCaption;
          output.push(`<figure style="text-align: center;">`);
          output.push(`<img src="/tdps/${lyti}/${filename}" alt="${alt}" />`);
          if (caption) {
            output.push(`<figcaption>${caption}</figcaption>`);
          }
          output.push(`</figure>`);
          output.push('');
          pendingImageCaption = null;
          subSection = 'none';
          break;
        }

        case 'table_caption':
          pendingTableCaption = line;
          subSection = 'none';
          break;

        case 'table_body':
          output.push(line);
          break;

        default:
          // 'none', 'images', 'image', 'tables', 'table' — ignore bare content
          break;
      }
      continue;
    }
  }

  // Trim trailing blank lines and return
  let result = output.join('\n');
  result = result.replace(/\n{3,}/g, '\n\n').trimEnd();
  return result;
}

/**
 * Generate a URL-safe heading ID from heading text.
 * Must match the marked renderer's heading ID logic.
 */
export function slugifyHeading(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .trim()
    .replace(/\s+/g, '-');
}

/**
 * Extract headings from pre-processed (standard) markdown for TOC.
 * Generates URL-safe IDs from heading text.
 */
export function extractHeadings(markdown: string): TocHeading[] {
  const headings: TocHeading[] = [];
  const lines = markdown.split('\n');

  for (const line of lines) {
    const match = line.match(/^(#{2,6})\s+(.+)$/);
    if (!match) continue;

    const hashes = match[1];
    const text = match[2].trim();
    const level = hashes.length;

    const id = slugifyHeading(text);

    headings.push({ id, text, level });
  }

  return headings;
}
