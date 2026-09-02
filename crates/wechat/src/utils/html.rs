//! 文章 HTML 解析工具

/// 定位 js_content 内正文的字节区间
pub fn extract_article_section(bytes: &[u8]) -> Option<&[u8]> {
    // 1. 定位 js_content 节点(id 可能是双引号或单引号)
    let js_start = find_sub(bytes, b"id=\"js_content\"")
        .into_iter()
        .chain(find_sub(bytes, b"id='js_content'"))
        .min()?;

    // 2. 从 js_content 起找第一个 <section 开始标签
    let body = &bytes[js_start..];
    let sec_start = find_start_tag(body, b"section")?;

    // 3. 深度匹配到闭合的 </section>
    let sec_end = find_section_end(body, sec_start)?;

    let start = js_start + sec_start;
    let end = js_start + sec_end;
    Some(&bytes[start..end])
}

/// 子串查找
fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// 找到第一个 <tag 开始标签的位置(排除 </tag>)
fn find_start_tag(hay: &[u8], tag: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 + tag.len() <= hay.len() {
        if hay[i] == b'<' && hay[i + 1] != b'/' && hay[i + 1..].starts_with(tag) {
            let next = hay.get(i + 1 + tag.len()).copied().unwrap_or(b'>');
            if !next.is_ascii_alphanumeric() {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// 从第一个 <section 开始深度匹配, 返回闭合 </section> 之后的偏移
fn find_section_end(hay: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = start;
    while i < hay.len() {
        let Some(lt) = find_sub(&hay[i..], b"<") else {
            return None;
        };
        let lt = i + lt;

        // 跳过注释
        if hay[lt..].starts_with(b"<!--") {
            let close = find_sub(&hay[lt + 4..], b"-->").map(|p| lt + 4 + p + 3)?;
            i = close;
            continue;
        }

        // 闭合标签 </section>
        if hay.get(lt + 1) == Some(&b'/') {
            let rest = &hay[lt + 2..];
            if rest.starts_with(b"section") {
                let next = rest.get(7).copied().unwrap_or(b'>');
                if !next.is_ascii_alphanumeric() {
                    depth -= 1;
                    if depth == 0 {
                        let gt = find_tag_end(hay, lt)?;
                        return Some(gt + 1);
                    }
                    i = lt + 1;
                    continue;
                }
            }
            i = lt + 1;
            continue;
        }

        // 开始标签 <section ...>
        let rest = &hay[lt + 1..];
        if rest.starts_with(b"section") {
            let next = rest.get(7).copied().unwrap_or(b'>');
            if !next.is_ascii_alphanumeric() {
                let gt = find_tag_end(hay, lt)?;
                // 自闭合 <section .../> 不增加深度
                if gt > 0 && hay[gt - 1] != b'/' {
                    depth += 1;
                }
                i = gt + 1;
                continue;
            }
        }
        i = lt + 1;
    }
    None
}

/// 找到标签的结束 '>' (跳过引号内的内容)
fn find_tag_end(hay: &[u8], lt: usize) -> Option<usize> {
    let mut quote = None;
    let mut i = lt + 1;
    while i < hay.len() {
        let b = hay[i];
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
        } else {
            match b {
                b'"' | b'\'' => quote = Some(b),
                b'>' => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_nested_sections() {
        let html = r#"<div id="js_content"><section style="x"><p>开头</p><section><p>内层</p></section><p>结尾</p></section><div>其他</div></div>"#;
        let sec = extract_article_section(html.as_bytes()).unwrap();
        let sec = std::str::from_utf8(sec).unwrap();
        assert!(sec.starts_with("<section"));
        assert!(sec.ends_with("</section>"));
        assert!(sec.contains("开头"));
        assert!(sec.contains("内层"));
        assert!(sec.contains("结尾"));
        assert!(!sec.contains("其他"));
    }
}

/// 文章 section -> 简化 Markdown
///
/// 白名单标签(h1-h6/p/span/img/code/pre), 其余标签解包(保留文本);
/// img 按规则过滤(gif 丢弃, data-src 提取)
pub fn article_to_markdown(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() / 3);
    let mut in_pre = false;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'<' {
            let rel = find_sub(&bytes[i..], b"<").unwrap_or(bytes.len() - i);
            out.push_str(&decode_entities(&String::from_utf8_lossy(
                &bytes[i..i + rel],
            )));
            i += rel;
            continue;
        }

        // 注释
        if bytes[i..].starts_with(b"<!--") {
            if let Some(p) = find_sub(&bytes[i..], b"-->") {
                i += p + 3;
            } else {
                i += 1;
            }
            continue;
        }

        let Some(gt) = find_tag_end(bytes, i) else {
            out.push_str(&decode_entities(&String::from_utf8_lossy(&bytes[i..])));
            break;
        };
        let tag = &bytes[i..=gt];

        // 结束标签
        if tag.get(1) == Some(&b'/') {
            let name = tag_name(&tag[2..]);
            if name.eq_ignore_ascii_case(b"pre") {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n");
                in_pre = false;
            } else if name.eq_ignore_ascii_case(b"code") {
                if !in_pre {
                    out.push('`');
                }
            } else if is_heading(name) {
                out.push_str("\n\n");
            } else if name.eq_ignore_ascii_case(b"p") {
                out.push_str("\n\n");
            }
            i = gt + 1;
            continue;
        }

        let name = tag_name(&tag[1..]);

        // script/style: 连内容删除
        if name.eq_ignore_ascii_case(b"script") || name.eq_ignore_ascii_case(b"style") {
            if let Some(close) = find_close_tag(&bytes[i..], name) {
                i += close;
            } else {
                i = gt + 1;
            }
            continue;
        }

        // br -> 换行
        if name.eq_ignore_ascii_case(b"br") {
            out.push('\n');
            i = gt + 1;
            continue;
        }

        // img -> ![alt](url)
        if name.eq_ignore_ascii_case(b"img") {
            if let Some(img) = img_to_markdown(tag) {
                out.push('\n');
                out.push_str(&img);
                out.push('\n');
            }
            i = gt + 1;
            continue;
        }

        // pre / code
        if name.eq_ignore_ascii_case(b"pre") {
            out.push_str("\n```\n");
            in_pre = true;
            i = gt + 1;
            continue;
        }
        if name.eq_ignore_ascii_case(b"code") {
            // pre 内 code 由围栏承载, 不再标记
            if !in_pre {
                out.push('`');
            }
            i = gt + 1;
            continue;
        }

        // 标题
        if is_heading(name) {
            let level = name[1] - b'0';
            out.push('\n');
            for _ in 0..level {
                out.push('#');
            }
            out.push(' ');
            i = gt + 1;
            continue;
        }

        // 其余标签: 解包(只删标记, 保留文本)
        i = gt + 1;
    }

    cleanup(out)
}

/// 提取标签名(到空白 / > / / 为止)
fn tag_name(bytes: &[u8]) -> &[u8] {
    let n = bytes
        .iter()
        .take_while(|&&b| !b.is_ascii_whitespace() && b != b'>' && b != b'/')
        .count();
    &bytes[..n]
}

/// 读取标签内属性值(引号感知, 支持单双引号)
fn attr_value<'a>(tag: &'a [u8], attr: &[u8]) -> Option<&'a [u8]> {
    let mut i = 0;
    while i + attr.len() <= tag.len() {
        if tag[i..].starts_with(attr) {
            let after = i + attr.len();
            if matches!(
                tag.get(after),
                Some(&b'=') | Some(b' ') | Some(b'\t') | Some(b'\n')
            ) {
                let mut j = after;
                while j < tag.len() && tag[j].is_ascii_whitespace() {
                    j += 1;
                }
                if tag.get(j) == Some(&b'=') {
                    j += 1;
                    while j < tag.len() && tag[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if let Some(&q) = tag.get(j) {
                        if q == b'"' || q == b'\'' {
                            let start = j + 1;
                            let end = start + find_sub(&tag[start..], &[q])?;
                            return Some(&tag[start..end]);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// 找到 </name 闭合标签结束后的偏移
fn find_close_tag(hay: &[u8], name: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 2 + name.len() <= hay.len() {
        if hay[i] == b'<' && hay[i + 1] == b'/' && hay[i + 2..].starts_with(name) {
            let next = hay.get(i + 2 + name.len()).copied().unwrap_or(b'>');
            if !next.is_ascii_alphanumeric() {
                let gt = find_tag_end(hay, i)?;
                return Some(gt + 1);
            }
        }
        i += 1;
    }
    None
}

fn is_heading(name: &[u8]) -> bool {
    name.len() == 2 && name[0].eq_ignore_ascii_case(&b'h') && (b'1'..=b'6').contains(&name[1])
}

/// img -> markdown 图片; gif 丢弃; data-src 优先
fn img_to_markdown(tag: &[u8]) -> Option<String> {
    let url = attr_value(tag, b"data-src").or_else(|| attr_value(tag, b"src"))?;
    if url.ends_with(b"wx_fmt=gif") {
        return None;
    }
    let url = decode_entities(&String::from_utf8_lossy(url));
    let alt = decode_entities(&String::from_utf8_lossy(
        attr_value(tag, b"alt").unwrap_or(b""),
    ));
    Some(format!("![{alt}]({url})"))
}

/// 解码常见 HTML 实体(&amp; &lt; &gt; &quot; &apos; &nbsp; 及 &#NN; / &#xHH;)
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        if ch == '&' {
            // 分号须在 12 字节内(字节迭代, 避免多字节字符边界问题)
            let start = i + 1;
            if let Some(rel) = s.as_bytes()[start..]
                .iter()
                .take(12)
                .position(|&b| b == b';')
            {
                let name = &s[start..start + rel];
                if let Some(decoded) = decode_entity_name(name) {
                    out.push(decoded);
                    i = start + rel + 1;
                    continue;
                }
            }
        }
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn decode_entity_name(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" | "x27" => Some('\''),
        "nbsp" | "#160" | "xa0" => Some(' '),
        "ensp" | "#8194" | "emsp" | "#8195" => Some(' '),
        "middot" | "#183" => Some('·'),
        "times" => Some('×'),
        "copy" => Some('©'),
        "reg" => Some('®'),
        _ => {
            // 数字实体 &#NN; / &#xHH;
            let Some(num) = name.strip_prefix('#') else {
                return None;
            };
            let code = if let Some(hex) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
                u32::from_str_radix(hex, 16).ok()
            } else {
                num.parse::<u32>().ok()
            };
            code.and_then(char::from_u32)
        }
    }
}

/// 行尾去空白, 压缩连续空行(代码块内保留)
fn cleanup(raw: String) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    let mut in_fence = false;
    let mut prev_blank = false;

    for line in raw.lines().map(str::trim_end) {
        let t = line.trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
        }
        if in_fence {
            out_lines.push(line.to_string());
            prev_blank = false;
        } else if t.is_empty() {
            if !prev_blank {
                out_lines.push(String::new());
            }
            prev_blank = true;
        } else {
            out_lines.push(line.to_string());
            prev_blank = false;
        }
    }

    while out_lines.first().is_some_and(|l| l.trim().is_empty()) {
        out_lines.remove(0);
    }
    while out_lines.last().is_some_and(|l| l.trim().is_empty()) {
        out_lines.pop();
    }
    out_lines.join("\n")
}

#[cfg(test)]
mod markdown_tests {
    use super::*;

    fn md(html: &str) -> String {
        article_to_markdown(html.as_bytes())
    }

    #[test]
    fn headings_and_inline_code() {
        let out = md(r#"<h2>标题</h2><p>段落 <code>let x</code> 和文字</p>"#);
        assert_eq!(out, "## 标题\n\n段落 `let x` 和文字");
    }
}
