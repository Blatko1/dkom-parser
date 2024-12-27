use anyhow::Error;
use chrono::NaiveDate;
use core::panic;
use regex::Regex;
use scraper::Selector;

fn main() -> anyhow::Result<()> {
    let res = reqwest::blocking::get("https://www.dkom.hr/")?;
    let source = res.text()?;
    let document = scraper::Html::parse_document(&source);

    let content_selector = Selector::parse(
        "div.hh-items-wrap > div:nth-child(2) > div:first-child > div.hhi-content > ul",
    )
    .map_err(|e| eprintln!("{e}"))
    .unwrap();
    let content_count = Selector::parse(
        "div.hh-items-wrap > div:nth-child(2) > div:first-child > div.hhi-content > ul > li",
    )
    .map_err(|e| eprintln!("{e}"))
    .unwrap();

    let list: String = document
        .select(&content_selector)
        .next()
        .unwrap()
        .inner_html();
    let list_len = document.select(&content_count).count();

    let data_regex = Regex::new(r#"<a href="(.+?)">[\s\S]*?\/(\d+)<\/a>"#).unwrap();

    let exe_dir = std::env::current_exe().unwrap();
    let save_dir = exe_dir.parent().unwrap();
    let mut succesfully_parsed_docs = 0;
    for (_, [pdf_url, class]) in data_regex.captures_iter(&list).map(|c| c.extract()) {
        print!("Preuzimanje PDF dokumenta s poveznice: {}...", pdf_url);
        let pdf_res = reqwest::blocking::get(pdf_url)?;
        let pdf_bytes = pdf_res.bytes()?.to_vec();
        print!(
            " Dokument preuzet!\nČitanje informacija... [klasa: {}]",
            class
        );

        let date = match get_date_from_pdf(&pdf_bytes) {
            Ok((year, month, day)) => match NaiveDate::from_ymd_opt(year as i32, month, day) {
                Some(date) => {
                    println!(" [datum: {}.{}.{}]", day, month, year);
                    succesfully_parsed_docs += 1;
                    date.format("%Y.%m.%d.").to_string()
                }
                None => {
                    println!(
                        "Neispravan datum (dan.mjesec.godina): {}.{}.{}\n",
                        day, month, year
                    );
                    String::from("_xx.xx.xx_")
                }
            },
            Err(err) => {
                eprintln!("\nError čitanja PDF-a: {}", err);
                String::from("_xx.xx.xx_")
            }
        };
        let name = &format!("{} {}.pdf", date, class);
        std::fs::write(save_dir.join(name), &pdf_bytes)?;
        println!("Dokument spremljen pod nazivom '{}'.\n", name);
    }

    print!("Preuzimanje završeno! Uspješno procesuirano {} od {} PDF dokumenata.\nSvi preuzeti dokumenti su spremljeni na lokaciji: {:?}", succesfully_parsed_docs, list_len, save_dir);

    Ok(())
}

fn get_date_from_pdf(pdf_bytes: &[u8]) -> anyhow::Result<(u32, u32, u32)> {
    let pdf_content = pdf_extract::extract_text_from_mem(pdf_bytes)?;
    let date_regex =
        Regex::new(r"A:[\s\S]*?J:[\s\S]*?, (\d+)\. ([^\d]+) (\d+).").unwrap();
    let Some((_, [day, month, year])) = date_regex
        .captures_iter(&pdf_content)
        .map(|c| c.extract())
        .next()
    else {
        return Err(Error::msg("Neuspjelo čitanje datuma iz PDF dokumenta!"));
    };
    let parsed_day: u32 = day.parse()?;
    let parsed_year: u32 = year.parse()?;
    let parsed_month = month_to_digit(month);

    Ok((parsed_year, parsed_month, parsed_day))
}

fn month_to_digit(month: &str) -> u32 {
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
