use anyhow::Context;
use chrono::NaiveDate;
use lopdf::Document;
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
        //std::thread::sleep(std::time::Duration::from_secs(1));
        let raw_pdf_url: String = elem
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
        let class_unparsed = raw_class
            .chars()
            .skip_while(|&c| c != '/')
            .skip(1)
            .skip_while(|&c| c != '/')
            .skip(1)
            .skip_while(|&c| c != '/')
            .skip(1)
            .collect::<String>();
        let class: u32 = match class_unparsed.parse() {
            Ok(c) => c,
            Err(_) => {
                println!("Klasa ima neispravan format: {}!!!\n\n", class_unparsed);
                continue;
            }
        };

        println!("\n Preuzimam dokument s: {}...", raw_pdf_url);
        let pdf_res = reqwest::blocking::get(raw_pdf_url)?;
        let pdf_bytes = pdf_res.bytes()?.to_vec();
        println!("Dokument preuzet!\n Čitam datum odluke...");

        let date = match get_date_from_pdf(&pdf_bytes) {
            Ok((year, month, day)) => match NaiveDate::from_ymd_opt(year, month, day) {
                Some(date) => {println!("Datum odluke: {}.{}.{}", day, month, year); date.format("%Y.%m.%d.").to_string()},
                None => {
                    println!(
                        "Neispravan datum (dan.mjesec.godina): {}.{}.{}",
                        day, month, year
                    );
                    String::from("_xx.xx.xx_")
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                String::from("_xx.xx.xx_")
            }
        };
        let name = &format!("{} {}.pdf", date, class);
        let path = std::env::current_exe().unwrap().parent().unwrap().join(name);
        std::fs::write(
            &path,
            &pdf_bytes,
        )?;
        println!("Dokument spremljen pod nazivom '{}' na putanji: {:?}.\n", name, path);
    }
    println!("GOTOVO!!!!!");

    Ok(())
}

fn get_date_from_pdf(pdf_bytes: &[u8]) -> anyhow::Result<(i32, u32, u32)> {
    let doc = Document::load_mem(pdf_bytes).unwrap();
    let first_page = doc.extract_text(&[1])?;
    let idx = first_page.find("Zagreb,").context(
        "!!!!!!!!!!!!!!!!!U dokumentu nema 'Zagreb,'!!!!!!!!!!!!!!!!!\
            \n Dokument je vjerovatno slika!!!!!!!!!!!!!!!!!",
    )?;
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
    let day: u32 = raw_day.trim().parse().unwrap();
    let raw_month = raw_date
        .chars()
        .skip_while(|&c| c != '.')
        .skip(1)
        .take_while(|&c| !c.is_numeric())
        .collect::<String>().replace(char::is_whitespace, "");
    let month = month_str_to_digit(raw_month.trim());
    let raw_year = raw_date
        .chars()
        .skip_while(|&c| c != '.')
        .skip_while(|&c| !c.is_numeric()).take_while(|_| {true})
        .collect::<String>().replace(char::is_whitespace, "");
    let year: i32 = raw_year.trim().parse().unwrap();
    println!("raw: {}", raw_date);

    Ok((year, month, day))
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
        "studenog" | "studenoga" => 11,
        "prosinca" => 12,
        _ => panic!("nepoznat mjesec! '{}'", month),
    }
}
