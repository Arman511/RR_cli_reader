#[warn(non_snake_case)]
use colored::{Color, Colorize};
use crossterm::execute;
use crossterm::terminal;
use serde::{Deserialize, Serialize};
use std::io::stdout;

#[derive(Serialize, Deserialize)]
pub struct SessionConfig {
    pub book_name: String,
    pub book_id: u64,
    pub chapter_id: u64,
    pub color: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            book_name: String::from(""),
            book_id: 0,
            chapter_id: 0,
            color: String::from("white"),
        }
    }
}

impl Clone for SessionConfig {
    fn clone(&self) -> Self {
        Self {
            book_name: self.book_name.clone(),
            book_id: self.book_id.clone(),
            chapter_id: self.chapter_id.clone(),
            color: self.color.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Chapter {
    id: u64,
    title: String,
    order: u64,
    url: String,
}

impl Clone for Chapter {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            title: self.title.clone(),
            order: self.order.clone(),
            url: self.url.clone(),
        }
    }
}

fn main_menu(config: &SessionConfig) -> Vec<Chapter> {
    let mut option = String::new();
    let mut local_config = config.clone();
    loop {
        display_menu();
        println!("Enter option: ");
        option.clear();
        std::io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");
        option = option.trim().to_string();
        match option.to_uppercase().as_str() {
            "B" => {
                let chapters = load_book(local_config.clone());
                if chapters.len() > 0 {
                    return chapters;
                }
            }
            "Q" => {
                println!("Goodbye!");
                std::process::exit(0);
            }
            "P" => return get_chapters(local_config.book_id.clone()),
            "C" => change_color(local_config.clone()),
            _ => println!("Invalid option"),
        }

        local_config = confy::load("RRlCliReader", "SessionConfig").unwrap_or_default();
    }
}

fn display_menu() {
    println!("P: Continue previous book");
    println!("B: Load book");
    println!("C: Change colour of text");
    println!("Q: Quit");
}

fn load_book(config: SessionConfig) -> Vec<Chapter> {
    let mut book_id: u64;
    let mut book_name = String::new();
    let chapter_id: u64;
    let mut response: String;
    loop {
        let mut option = String::new();
        println!("Enter book id(enter exit to go back): ");
        std::io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");
        if option.trim() == "exit" {
            return vec![];
        }
        match option.trim().parse::<u64>() {
            Ok(n) => {
                book_id = n;
            }
            Err(_) => {
                println!("Invalid book id");
                continue;
            }
        }
        let url = format!("https://www.royalroad.com/fiction/{}", book_id);
        response = reqwest::blocking::get(url).unwrap().text().unwrap();
        if response.contains("Not Found") {
            println!("Invalid book id");
        } else {
            break;
        }
    }
    if let Some(line) = response.lines().find(|line| line.contains("<title>")) {
        book_name = line
            .trim_end_matches(" | Royal Road</title>")
            .trim()
            .replace("<title>", "");
        println!("Book name: {}", book_name);
    } else {
        println!("No book name found");
    }
    let chapters = get_chapters(book_id);
    println!("{} chapters found", chapters.len());
    for chapter in &chapters {
        println!(
            "{}: {}",
            chapter.id.to_string().color(Color::Cyan),
            chapter.title
        );
    }
    loop {
        println!("Enter chapter id(enter exit to go back, enter to start from beginning): ");
        let mut option = String::new();
        std::io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");
        if option.trim() == "exit" {
            return vec![];
        } else if option.trim() == "" {
            chapter_id = chapters[0].id;
            break;
        } else {
            match option.trim().parse::<u64>() {
                Ok(n) => {
                    if chapters.iter().any(|chapter| chapter.id == n) {
                        chapter_id = n;
                        break;
                    } else {
                        println!("Invalid chapter id");
                        continue;
                    }
                }
                Err(_) => {
                    println!("Invalid chapter id");
                    continue;
                }
            }
        }
    }

    let new_config = SessionConfig {
        book_name: book_name.trim().to_string(),
        book_id: book_id,
        chapter_id: chapter_id,
        color: config.color.clone(),
    };
    confy::store("RRlCliReader", "SessionConfig", new_config).unwrap();
    chapters
}

fn get_chapters(book_id: u64) -> Vec<Chapter> {
    let url = format!("https://www.royalroad.com/fiction/{}", book_id);
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();
    if let Some(line) = response
        .lines()
        .find(|line| line.contains("window.chapters = "))
    {
        let json_data = line.trim_end_matches(";").replace("window.chapters = ", "");
        let chapters: Vec<Chapter> = serde_json::from_str(&json_data.trim()).unwrap();
        chapters
    } else {
        println!("No chapters found");
        vec![]
    }
}

fn change_color(config: SessionConfig) {
    let mut color = String::new();
    loop {
        println!("Enter color: ");
        std::io::stdin()
            .read_line(&mut color)
            .expect("Failed to read line");
        let check = colored::Color::from(color.trim());
        match check {
            Color::Black => break,
            Color::Red => break,
            Color::Green => break,
            Color::Yellow => break,
            Color::Blue => break,
            Color::Magenta => break,
            Color::Cyan => break,
            Color::White => match color.trim() {
                "white" => break,
                _ => println!("Invalid color"),
            },
            _ => println!("Invalid color"),
        }
    }
    let new_config = SessionConfig {
        book_name: config.book_name.clone(),
        book_id: config.book_id.clone(),
        chapter_id: config.chapter_id.clone(),
        color: color.trim().to_string(),
    };
    confy::store("RRlCliReader", "SessionConfig", new_config).unwrap();
}

fn main() {
    let mut config: SessionConfig;

    let mut chapters: Vec<Chapter> = vec![];
    loop {
        config = confy::load("RRlCliReader", "SessionConfig").unwrap_or_default();
        execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        chapters = main_menu(&mut config);
        if chapters.len() == 0 {
            continue;
        }

        show_book_page(config.clone(), &chapters);
    }
}

fn show_book_page(config: SessionConfig, chapters: &Vec<Chapter>) {
    let mut chapter = chapters
        .iter()
        .find(|chapter| chapter.id == config.chapter_id)
        .unwrap()
        .clone();
    let mut url = format!("https://www.royalroad.com{}", chapter.url);
    let mut chapter_number = chapter.order;
    let mut title: String;
    let mut filtered_data: String;
    loop {
        let new_config = SessionConfig {
            book_name: config.book_name.clone(),
            book_id: config.book_id.clone(),
            chapter_id: chapter.id.clone(),
            color: config.color.clone(),
        };
        confy::store("RRlCliReader", "SessionConfig", new_config).unwrap();
        execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
        filtered_data = filter_content(url.clone());
        title = chapter.title.clone();
        println!(
            "Press enter to further the story(type exit to quit)\n\n{}\n\n",
            title.color(config.color.clone()).bold()
        );
        let mut option: String = String::new();
        for line in filtered_data.split("\n") {
            println!("{}", line.color(config.color.clone()));
            std::io::stdin().read_line(&mut option).unwrap();
            if option.trim() == "exit" {
                return;
            }
        }

        println!(
            "{}",
            "<: Previous Chapter, >: Next Chapter, Q: Quit, C: Chapter list"
                .color(config.color.clone())
        );
        loop {
            option = String::new();
            std::io::stdin()
                .read_line(&mut option)
                .expect("Failed to read line");
            option = option.trim().to_string();
            match option.to_uppercase().as_str() {
                "<" => {
                    if chapter_number == 0 {
                        println!("No previous chapter");
                    } else {
                        chapter_number -= 1;
                        chapter = chapters[chapter_number as usize].clone();
                        url = format!("https://www.royalroad.com{}", chapter.url);
                        break;
                    }
                }
                ">" => {
                    if chapter_number == chapters.len() as u64 - 1 {
                        println!("No next chapter");
                    } else {
                        chapter_number += 1;
                        chapter = chapters[chapter_number as usize].clone();
                        url = format!("https://www.royalroad.com{}", chapter.url);
                        break;
                    }
                }
                "Q" => {
                    return;
                }
                "C" => {
                    for chapter in chapters {
                        println!(
                            "{}: {}",
                            chapter.id.to_string().color(Color::Cyan),
                            chapter.title
                        );
                    }

                    loop {
                        println!("Enter chapter id(enter exit to go back, enter to start from beginning): ");
                        let mut option = String::new();
                        std::io::stdin()
                            .read_line(&mut option)
                            .expect("Failed to read line");
                        if option.trim() == "exit" {
                            break;
                        } else if option.trim() == "" {
                            chapter = chapters[0].clone();
                            chapter_number = chapter.order;
                            url = format!("https://www.royalroad.com{}", chapter.url);
                            break;
                        } else {
                            match option.trim().parse::<u64>() {
                                Ok(n) => {
                                    if chapters.iter().any(|chapter| chapter.id == n) {
                                        chapter = chapters
                                            .iter()
                                            .find(|chapter| chapter.id == n)
                                            .unwrap()
                                            .clone();
                                        chapter_number = chapter.order;
                                        url = format!("https://www.royalroad.com{}", chapter.url);
                                        break;
                                    } else {
                                        println!("Invalid chapter id");
                                        continue;
                                    }
                                }
                                Err(_) => {
                                    println!("Invalid chapter id");
                                    continue;
                                }
                            }
                        }
                    }

                    break;
                }
                _ => {
                    println!("Invalid option");
                    println!(
                        "{}",
                        "<: Previous Chapter, >: Next Chapter, Q: Quit, C: Chapter list"
                            .color(config.color.clone())
                    );
                }
            }
        }
    }
}

fn filter_content(url: String) -> String {
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();
    let lines: Vec<&str> = response.lines().collect();
    let index = lines
        .iter()
        .position(|line| line.contains("chapter-inner chapter-content"))
        .unwrap();
    let lines: Vec<&str> = lines.iter().skip(index + 1).copied().collect();
    let end_index = lines
        .iter()
        .position(|line| line.contains("</div>"))
        .unwrap();
    let lines: Vec<&str> = lines.iter().take(end_index).copied().collect();
    let lines: Vec<&str> = lines.iter().map(|line| line.trim()).collect();
    let lines: Vec<String> = lines
        .iter()
        .map(|line| {
            let mut new_line = String::new();
            let mut skip = false;
            let line = line.replace("<br>", "\n");
            for c in line.chars() {
                if c == '<' {
                    skip = true;
                } else if c == '>' {
                    skip = false;
                } else if !skip {
                    new_line.push(c);
                }
            }
            new_line
        })
        .collect();

    lines.join("\n")
}
