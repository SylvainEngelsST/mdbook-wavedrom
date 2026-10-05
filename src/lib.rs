// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind::*, Event, Options, Parser, Tag, TagEnd};

pub struct Wavedrom;

impl Preprocessor for Wavedrom {
    fn name(&self) -> &str {
        "wavedrom"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(Wavedrom::add_wavedrom(chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }
}

fn escape_html(s: &str) -> String {
    let mut output = String::new();
    for c in s.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            _ => output.push(c),
        }
    }
    output
}

fn add_wavedrom(content: &str) -> Result<String> {
    let mut wavedrom_content = String::new();
    let mut in_wavedrom_block = false;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut code_span = 0..0;
    let mut start_new_code_span = true;

    let mut wavedrom_blocks = vec![];

    let events = Parser::new_ext(content, opts);
    for (e, span) in events.into_offset_iter() {
        log::debug!("e={:?}, span={:?}", e, span);
        if let Event::Start(Tag::CodeBlock(Fenced(code))) = e.clone() {
            if &*code == "wavedrom" {
                in_wavedrom_block = true;
                wavedrom_content.clear();
            }
            continue;
        }

        if !in_wavedrom_block {
            continue;
        }

        // We're in the code block. The text is what we want.
        // Code blocks can come in multiple text events.
        if let Event::Text(_) = e {
            if start_new_code_span {
                code_span = span;
                start_new_code_span = false;
            } else {
                code_span = code_span.start..span.end;
            }

            continue;
        }

        if let Event::End(TagEnd::CodeBlock) = e {
            in_wavedrom_block = false;

            let wavedrom_content = &content[code_span.clone()];
            let wavedrom_content = escape_html(wavedrom_content);
            let wavedrom_content = wavedrom_content.replace("\r\n", "\n");
            let wavedrom_code = format!("<pre class=\"wavedrom\">{}</pre>\n\n", wavedrom_content);
            wavedrom_blocks.push((span, wavedrom_code));
            start_new_code_span = true;
        }
    }

    let mut content = content.to_string();
    for (span, block) in wavedrom_blocks.iter().rev() {
        let pre_content = &content[0..span.start];
        let post_content = &content[span.end..];
        content = format!("{}\n{}{}", pre_content, block, post_content);
    }
    Ok(content)
}

impl Wavedrom {
    fn add_wavedrom(chapter: &mut Chapter) -> Result<String> {
        add_wavedrom(&chapter.content)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::add_wavedrom;

    #[test]
    fn adds_wavedrom() {
        let content = r#"# Chapter

```wavedrom
{signal: [
  {name: 'clk', wave: 'p.....|...'}
]}
```

Text
"#;

        let expected = r#"# Chapter


<pre class="wavedrom">{signal: [
  {name: 'clk', wave: 'p.....|...'}
]}
</pre>



Text
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn leaves_tables_untouched() {
        // Regression test.
        // Previously we forgot to enable the same markdwon extensions as mdbook itself.

        let content = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        let expected = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn leaves_html_untouched() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML

        let content = r#"# Heading

<del>

*foo*

</del>
"#;

        let expected = r#"# Heading

<del>

*foo*

</del>
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn html_in_list() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML

        let content = r#"# Heading

1. paragraph 1
   ```
   code 1
   ```
2. paragraph 2
"#;

        let expected = r#"# Heading

1. paragraph 1
   ```
   code 1
   ```
2. paragraph 2
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn escape_in_wavedrom_block() {
        let _ = env_logger::try_init();
        let content = r#"
```wavedrom
classDiagram
    class PingUploader {
        <<interface>>
        +Upload() UploadResult
    }
```

hello
"#;

        let expected = r#"

<pre class="wavedrom">classDiagram
    class PingUploader {
        &lt;&lt;interface&gt;&gt;
        +Upload() UploadResult
    }
</pre>



hello
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn more_backticks() {
        let _ = env_logger::try_init();
        let content = r#"# Chapter

```wavedrom
{signal: [
  {name: 'clk', wave: 'p.....|...'}
]}
```

Text
"#;

        let expected = r#"# Chapter


<pre class="wavedrom">{signal: [
  {name: 'clk', wave: 'p.....|...'}
]}
</pre>



Text
"#;

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }

    #[test]
    fn crlf_line_endings() {
        let _ = env_logger::try_init();
        let content = "# Chapter\r\n\r\n````wavedrom\r\n\r\n{signal: [\r\n{name: 'clk', wave: 'p.....|...'}\r\n]}\r\n````";
        let expected =
            "# Chapter\r\n\r\n\n<pre class=\"wavedrom\">\n{signal: [\n{name: 'clk', wave: 'p.....|...'}\n]}\n</pre>\n\n";

        assert_eq!(expected, add_wavedrom(content).unwrap());
    }
}
