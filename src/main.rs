use core::panic;
use scraper::Selector;

fn main() -> anyhow::Result<()> {
    let res = reqwest::blocking::get("https://www.dkom.hr/")?;
    let source = res.text()?;
    let document = scraper::Html::parse_document(&source);

    let content_selector = Selector::parse(
        "div.hh-items-wrap > div:nth-child(2) > div:first-child > div.hhi-content > ul > li",
    )
    .map_err(|e| eprintln!("{e}"))
    .unwrap();
    let content: Vec<String> = document
        .select(&content_selector)
        .map(|e| e.inner_html())
        .collect();

    for elem in content {
        let raw_url: String = elem
            .chars()
            .skip_while(|&c| c != '"')
            .skip(1)
            .take_while(|&c| c != '"')
            .collect();
        let raw_class: String = elem
            .chars()
            .skip_while(|&c| c != '>')
            .skip(1)
            .take_while(|&c| c != '<')
            .collect();
        let class: String = raw_class
            .chars()
            .skip_while(|&c| c != '/')
            .skip(1)
            .skip_while(|&c| c != '/')
            .skip(1)
            .skip_while(|&c| c != '/')
            .skip(1)
            .collect();

        let pdf_res = reqwest::blocking::get(raw_url)?;
        let pdf_bytes = pdf_res.bytes()?.to_vec();

        let doc = lopdf::Document::load_mem(&pdf_bytes).unwrap();
        let first_page = doc.extract_text(&[1])?;
        let idx = match first_page.find("Zagreb,") {
            Some(idx) => idx,
            None => {
                println!("U dokumentu nema 'Zagreb,'!!! \n Dokument je vjerovatno slika!");
                continue;
            }
        };
        let mut dot_counter = 0;
        let raw_date: String = first_page[idx..]
            .chars()
            .skip_while(|&c| c != ',')
            .skip(1)
            .take_while(|&c| {
                if c == '.' {
                    if dot_counter == 1 {
                        false
                    } else {
                        dot_counter += 1;
                        true
                    }
                } else {
                    true
                }
            })
            .collect();
        let raw_day: String = raw_date
            .chars()
            .take_while(|&c| c != '.')
            .filter(|&c| !c.is_whitespace())
            .collect();
        let day = raw_day.trim();
        let raw_month: String = raw_date
            .chars()
            .skip_while(|&c| c != '.')
            .skip(1)
            .take_while(|&c| !c.is_numeric())
            .collect();
        let month = month_str_to_digit(raw_month.trim());
        let raw_year: String = raw_date
            .chars()
            .skip_while(|&c| c != '.')
            .skip_while(|&c| !c.is_numeric())
            .take(4)
            .collect();
        let year = raw_year.trim();
        let date = format!("{}.{}.{}", year, month, day);
        std::fs::write(&format!("{} {}.pdf", date, class), &pdf_bytes)?;
        println!("preuzeo ");
    }

    Ok(())
}

fn month_str_to_digit(month: &str) -> u32 {
    match month {
        "siječnja" => 1,
        "veljače" => 2,
        "ožujka" => 3,
        "travnja" => 4,
        "svibnja" => 5,
        "lipnja" => 6,
        "srpnja" => 7,
        "kolovoza" => 8,
        "rujna" => 9,
        "listopada" => 10,
        "studenog" => 11,
        "prosinca" => 12,
        _ => panic!("nepoznat mjesec! '{}'", month),
    }
}
