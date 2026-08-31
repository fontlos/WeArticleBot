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

/// 保留的标签白名单: 标题 / 段落 / 行内 / 图片 / 代码
const KEEP_TAGS: &[&[u8]] = &[
    b"h1", b"h2", b"h3", b"h4", b"h5", b"h6", b"p", b"span", b"img", b"code", b"pre",
];

/// 简化文章正文: 只保留白名单标签, 其余标签标记丢弃但文本保留(解包);
/// img 按规则过滤(gif 丢弃), data-src 提升为 src; br 转为换行; script/style 连内容删除
pub fn simplify_article_section(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() / 4);
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            // 普通文本: 拷贝到下一个 '<'
            let rel = find_sub(&bytes[i..], b"<").unwrap_or(bytes.len() - i);
            out.extend_from_slice(&bytes[i..i + rel]);
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
            out.extend_from_slice(&bytes[i..]);
            break;
        };
        let tag = &bytes[i..=gt];

        // 结束标签
        if tag.get(1) == Some(&b'/') {
            let name = tag_name(&tag[2..]);
            if is_keep_tag(name) {
                out.extend_from_slice(tag);
            }
            i = gt + 1;
            continue;
        }

        let name = tag_name(&tag[1..]);

        // script/style: 连内容一起删除
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
            out.push(b'\n');
            i = gt + 1;
            continue;
        }

        // img: 应用图片规则
        if name.eq_ignore_ascii_case(b"img") {
            if let Some(img) = simplify_img(tag) {
                out.extend_from_slice(&img);
            }
            i = gt + 1;
            continue;
        }

        // 白名单标签原样保留, 其余解包(只删标签标记)
        if is_keep_tag(name) {
            out.extend_from_slice(tag);
        }
        i = gt + 1;
    }
    out
}

/// 图片规则: data-src 结尾是 wx_fmt=gif 则丢弃; 否则输出 <img src=url alt=alt>
/// (data-src 优先, 无则用 src; 解决 mdka 不认 data-src 的问题)
fn simplify_img(tag: &[u8]) -> Option<Vec<u8>> {
    let url = attr_value(tag, b"data-src").or_else(|| attr_value(tag, b"src"))?;
    if url.ends_with(b"wx_fmt=gif") {
        return None;
    }
    let alt = attr_value(tag, b"alt").unwrap_or(b"");
    let mut out = Vec::with_capacity(tag.len());
    out.extend_from_slice(b"<img src=\"");
    out.extend_from_slice(url);
    out.extend_from_slice(b"\" alt=\"");
    out.extend_from_slice(alt);
    out.extend_from_slice(b"\">");
    Some(out)
}

/// 提取标签名(到空白 / > / / 为止)
fn tag_name(bytes: &[u8]) -> &[u8] {
    let n = bytes
        .iter()
        .take_while(|&&b| !b.is_ascii_whitespace() && b != b'>' && b != b'/')
        .count();
    &bytes[..n]
}

fn is_keep_tag(name: &[u8]) -> bool {
    KEEP_TAGS.iter().any(|k| name.eq_ignore_ascii_case(k))
}

/// 读取标签内属性值(引号感知, 支持单双引号)
fn attr_value<'a>(tag: &'a [u8], attr: &[u8]) -> Option<&'a [u8]> {
    let mut i = 0;
    while i + attr.len() <= tag.len() {
        if tag[i..].starts_with(attr) {
            let after = i + attr.len();
            if matches!(tag.get(after), Some(&b'=') | Some(b' ') | Some(b'\t') | Some(b'\n')) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn simplify_str(html: &str) -> String {
        String::from_utf8_lossy(&simplify_article_section(html.as_bytes())).into_owned()
    }

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

    #[test]
    fn drop_gif_image() {
        let html = r#"<p>a</p><img data-src="https://mmbiz.qpic.cn/a/1?wx_fmt=gif" alt="gif"><p>b</p>"#;
        let out = simplify_str(html);
        assert!(!out.contains("img"), "gif 图应被丢弃: {out}");
        assert!(out.contains("a"));
        assert!(out.contains("b"));
    }

    #[test]
    fn keep_png_and_promote_data_src() {
        let html = r#"<p>x</p><img data-src="https://mmbiz.qpic.cn/a/2?wx_fmt=png" src="empty.gif" alt="图">"#;
        let out = simplify_str(html);
        assert_eq!(
            out,
            r#"<p>x</p><img src="https://mmbiz.qpic.cn/a/2?wx_fmt=png" alt="图">"#
        );
    }

    #[test]
    fn unwrap_non_keep_tags_keep_text() {
        let html = r#"<section><div><p>正文</p></div></section><ul><li>条目</li></ul>"#;
        let out = simplify_str(html);
        assert_eq!(out, "<p>正文</p>条目");
    }

    #[test]
    fn br_to_newline() {
        let html = r#"<p>a<br/>b</p>"#;
        let out = simplify_str(html);
        assert_eq!(out, "<p>a\nb</p>");
    }

    #[test]
    fn drop_script_with_content() {
        let html = r#"<p>a</p><script>var x = '<p>假的</p>';</script><p>b</p>"#;
        let out = simplify_str(html);
        assert_eq!(out, "<p>a</p><p>b</p>");
    }

    #[test]
    fn keep_code_and_pre() {
        let html = r#"<p>inline <code>let x</code></p><pre><code class="language-rust">fn main() {}</code></pre>"#;
        let out = simplify_str(html);
        assert_eq!(
            out,
            r#"<p>inline <code>let x</code></p><pre><code class="language-rust">fn main() {}</code></pre>"#
        );
    }
}
