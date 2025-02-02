use markdown_table_formatter as tfm;
use regex::Regex;
use std::io::{self, BufRead};

const RX: &str = r"^([ ]*[/]?[*]*)(.*)";

fn main() {
    let mut reader = io::stdin().lock();
    let (lines, success) = collect_lines(&mut reader);
    drop(reader);

    let mut writer = io::stdout().lock();
    rewrite_lines(&mut writer, lines, success);
    drop(writer);
}

fn rewrite_lines(w: &mut impl io::Write, lines: Vec<Line>, success: bool) {
    if success {
        let lines_fx: Vec<String> = lines
            .iter()
            .map(|x| x.line[x.index..].to_string())
            .collect();
        let txt = lines_fx.join("");
        let txt = fmt_table(txt);
        let new: Vec<_> = txt.split_terminator("\n").collect();

        assert_eq!(new.len(), lines.len());
        assert!(new.len() <= lines.len());

        for i in 0..(new.len()) {
            let item = &lines[i];
            let _ = write!(w, "{}{}\n", &(item.line[0..item.index]), new[i]);
        }
    }

    if !success {
        for i in lines {
            let _ = write!(w, "{}", i.line);
        }
    }
}

fn collect_lines(reader: &mut impl BufRead) -> (Vec<Line>, bool) {
    let rx = Regex::new(&RX).unwrap();
    let mut result = true;
    let mut v = Vec::new();
    loop {
        let mut buffer = String::new();
        let size = reader.read_line(&mut buffer).unwrap_or(0);
        if size > 0 {
            let mut item = Line {
                line: (&buffer[0..size]).to_string(),
                index: 0,
            };

            if let Some(m) = rx.captures(&item.line) {
                item.index = m.get(2).unwrap().start();
            } else {
                result = false;
            }

            v.push(item);
        } else {
            break;
        }
    }
    (v, result)
}

struct Line {
    line: String,
    index: usize,
}

fn fmt_table(txt: String) -> String {
    tfm::format_tables(txt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Captures;

    const EXAMPLE: &str = r#"/** * terrible
 * my doxy shit
 *    ghfddd ghfddd
 * <<<>>>
 *
 * |   Field a   |   fb   |left-field|
 * |---:|   :---:|:-|
 * | 5 | 2 |bla|
 * | 7 |   blav cfddd |ha|
 */"#;
    const EXAMPLE_NICE: &str = r#"/** * terrible
 * my doxy shit
 *    ghfddd ghfddd
 * <<<>>>
 *
 * | Field a |     fb     | left-field |
 * | ------: | :--------: | :--------- |
 * |       5 |     2      | bla        |
 * |       7 | blav cfddd | ha         |
 */"#;

    #[test]
    fn test_read_lines() {
        let mut x = EXAMPLE[0..(EXAMPLE.len() - 3)].as_bytes();
        let (lines, success) = collect_lines(&mut x);

        assert!(success);

        let mut w = Vec::new();
        rewrite_lines(&mut w, lines, success);

        let result = String::from_utf8_lossy(&w).to_string();
        assert_eq!(&EXAMPLE_NICE[0..(EXAMPLE_NICE.len() - 3)], result.as_str());
    }

    fn capture<'a>(haystack: &'a str) -> Option<Captures<'a>> {
        let rx = Regex::new(&RX).unwrap();
        rx.captures(haystack)
    }

    #[test]
    fn test_table_regex_start() {
        let m = capture("   /**    | bla |").unwrap();
        assert_eq!(m.get(1).unwrap().as_str(), "   /**");
        assert_eq!(m.get(2).unwrap().as_str(), "    | bla |");
    }

    #[test]
    fn test_table_regex_comment() {
        let m = capture("   *    | bla |").unwrap();
        assert_eq!(m.get(1).unwrap().as_str(), "   *");
        assert_eq!(m.get(2).unwrap().as_str(), "    | bla |");
    }

    #[test]
    fn test_table_regex_comment_more_stars() {
        let m = capture("   ****    | bla |").unwrap();
        assert_eq!(m.get(1).unwrap().as_str(), "   ****");
        assert_eq!(m.get(2).unwrap().as_str(), "    | bla |");
    }

    #[test]
    fn test_table_regex_md_only_space() {
        let m = capture("   | bla |").unwrap();
        assert_eq!(m.get(1).unwrap().as_str(), "   ");
        assert_eq!(m.get(2).unwrap().as_str(), "| bla |");
    }
}
