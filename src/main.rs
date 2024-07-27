use std::process::Command;
use std::{borrow::Cow, collections::BTreeMap, fs};

use std::io::{self, stdout, BufRead};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect};
use ratatui::{prelude::*, widgets::*};

use rustix::{
    process::geteuid,
    system::{sysinfo, uname},
    thread::ClockId,
    time::{clock_gettime, Timespec},
};

use owo_colors::{DynColors, OwoColorize};

mod passwd;

use crate::passwd::getpwuid;

const ARCH_LOGO: &str = r#"
                   -`
                  .o+`
                 `ooo/
                `+oooo:
               `+oooooo:
               -+oooooo+:
             `/:-:++oooo+:
            `/++++/+++++++:
           `/++++++++++++++:
          `/+++ooooooooooooo/`
         ./ooosssso++osssssso+`
        .oossssso-````/ossssss+`
       -osssssso.      :ssssssso.
      :osssssss/        osssso+++.
     /ossssssss/        +ssssooo/-
   `/ossssso+/:-        -:/+osssso+-
  `+sso+:-`                 `.-/+oso:
 `++:.                           `-/+/
 .`                                 `
"#;

const TRANSGENDER_FLAG: [DynColors; 5] = [
    DynColors::Rgb(91, 206, 250),
    DynColors::Rgb(245, 169, 184),
    DynColors::Rgb(255, 255, 255),
    DynColors::Rgb(91, 206, 250),
    DynColors::Rgb(245, 169, 184),
];

const NONBINARY_FLAG: [DynColors; 4] = [
    DynColors::Rgb(252, 244, 52),
    DynColors::Rgb(255, 255, 255),
    DynColors::Rgb(156, 89, 209),
    DynColors::Rgb(44, 44, 44),
];

const RAINBOW_FLAG: [DynColors; 6] = [
    DynColors::Rgb(252, 244, 52),
    DynColors::Rgb(255, 255, 255),
    DynColors::Rgb(156, 89, 209),
    DynColors::Rgb(252, 244, 52),
    DynColors::Rgb(255, 255, 255),
    DynColors::Rgb(156, 89, 209),
];

fn main() -> anyhow::Result<()> {
    // enable_raw_mode()?;
    // let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // terminal.draw(|frame| {
    //     let frame_size = frame.size();
    //     frame.render_widget(
    //         Paragraph::new("Hello Ratatui! (press 'q' to quit)")
    //             .white()
    //             .on_blue(),
    //         Rect::new(frame_size.x, frame_size.y, frame_size.width, 2),
    //     );
    // })?;

    // disable_raw_mode()?;
    // return Ok(());
    // let mut backend = buffer::create_backend();
    // let mut backend =    CrosstermBackend::new(io::stdout());
    // let mut buffer = Buffer::empty(Rect::new(0, 0, 500, 50));

    let os_release = parse_os_release();
    // // let sysinfo = sysinfo();
    // // dbg!(sysinfo);
    // // dbg!(&os_release);

    let uname = uname();

    print_flag(ARCH_LOGO, &TRANSGENDER_FLAG);

    let packages = Command::new("pacman")
        .arg("-Q")
        .output()?
        .stdout
        .lines()
        .count();

    println!("{}@{}", get_user(), uname.nodename().to_string_lossy());
    println!("--------------");
    println!("OS: {}", os_release.get("PRETTY_NAME").unwrap());
    println!("Arch: {}", std::env::consts::ARCH);
    println!("Kernel: {}", uname.release().to_string_lossy());
    println!("Uptime: {}", format_time(clock_gettime(ClockId::Boottime)));
    println!("Packages: {} (pacman)", packages);

    Ok(())
}

fn print_flag(logo: &str, flag: &[DynColors]) {
    let stripe_size = logo.lines().count() / flag.len();
    // TODO: Dynamically adjust line distribution based on extra lines
    let _extra_lines = logo.lines().count() % flag.len();

    for (index, line) in logo.lines().enumerate() {
        let mut color_index = index / stripe_size;

        if color_index >= flag.len() {
            color_index = flag.len() - 1
        }

        println!("{}", line.color(flag[color_index]));
    }
}

fn get_user() -> String {
    if let Ok(user) = std::env::var("USER") {
        return user;
    }

    if let Some(user) = getpwuid(geteuid()) {
        return user.name;
    }

    return "unknown".to_string();
}

fn format_time(timespec: Timespec) -> String {
    let total = timespec.tv_sec;

    let days = total / 86400;
    let hours = (total % 86400) / 3600;
    let minutes = (total % 86400 % 3600) / 60;
    let _secs = total % 86400 % 3600 % 60;

    let days = if days == 0 {
        Cow::Borrowed("")
    } else {
        Cow::Owned(format!("{days} days, "))
    };

    let hours = if hours == 0 {
        Cow::Borrowed("")
    } else {
        Cow::Owned(format!("{hours} hours, "))
    };

    format!("{days}{hours}{minutes} mins")
}

fn parse_os_release() -> BTreeMap<String, String> {
    let os_release_file =
        fs::read_to_string("/etc/os-release").expect("Failed to read /etc/os-release");

    let mut entries = BTreeMap::new();

    for line in os_release_file.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let (name, content) = line.split_once('=').expect("Invalid config");

        let name = name.trim();
        let content = content.trim_matches('"');

        entries.insert(name.to_string(), content.to_string());
    }

    entries
}
