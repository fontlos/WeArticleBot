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
        let Some(lt) = find_sub(&hay[i..], b"<") else { return None };
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
