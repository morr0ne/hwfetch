use std::{borrow::Cow, collections::BTreeMap, fmt::Write, fs, io::BufRead, process::Command};

use anyhow::Result;
use itertools::{EitherOrBoth, Itertools};
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use rustix::{
    process::geteuid,
    system::{sysinfo, uname},
    thread::ClockId,
    time::{clock_gettime, Timespec},
};

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

const DISTANCE: usize = 3;

fn main() -> Result<()> {
    // let sysinfo = sysinfo();
    // dbg!(sysinfo);
    // dbg!(&os_release);

    let flag = get_flag_string(ARCH_LOGO, &TRANSGENDER_FLAG)?;
    let info = get_info_string()?;

    let longest_line = flag.lines().map(|l| l.len()).max().unwrap();

    let info = info.repeat(3);

    let zipped = flag.lines().zip_longest(info.lines());

    for line in zipped {
        match line {
            EitherOrBoth::Both(flag, info) => {
                let spacing = " ".repeat(longest_line - flag.len() + DISTANCE);
                println!("{flag}{spacing}{info}")
            }
            EitherOrBoth::Left(flag) => println!("{flag}"),
            EitherOrBoth::Right(info) => {
                let spacing = " ".repeat(longest_line + DISTANCE);
                println!("{spacing}{info}",)
            }
        }
    }

    Ok(())
}

fn get_info_string() -> Result<String> {
    let mut info_string = String::new();

    let os_release = parse_os_release();
    let uname = uname();

    let packages = Command::new("pacman")
        .arg("-Q")
        .output()?
        .stdout
        .lines()
        .count();

    writeln!(
        &mut info_string,
        "{}@{}",
        get_user().color(AnsiColors::Cyan).bold(),
        uname
            .nodename()
            .to_string_lossy()
            .color(AnsiColors::Cyan)
            .bold()
    )?;
    writeln!(&mut info_string, "--------------")?;
    writeln!(
        &mut info_string,
        "{}: {}",
        "OS".color(AnsiColors::Cyan).bold(),
        os_release.get("PRETTY_NAME").unwrap()
    )?;
    writeln!(
        &mut info_string,
        "{}: {}",
        "Arch".color(AnsiColors::Cyan).bold(),
        std::env::consts::ARCH
    )?;
    writeln!(
        &mut info_string,
        "{}: {}",
        "Kernel".color(AnsiColors::Cyan).bold(),
        uname.release().to_string_lossy()
    )?;
    writeln!(
        &mut info_string,
        "{}: {}",
        "Uptime".color(AnsiColors::Cyan).bold(),
        format_time(clock_gettime(ClockId::Boottime))
    )?;
    writeln!(
        &mut info_string,
        "{}: {} (pacman)",
        "Packages".color(AnsiColors::Cyan).bold(),
        packages
    )?;

    Ok(info_string)
}

fn get_flag_string(logo: &str, flag: &[DynColors]) -> Result<String> {
    let mut flag_string = String::new();

    let stripe_size = logo.lines().count() / flag.len();
    // TODO: Dynamically adjust line distribution based on extra lines
    let _extra_lines = logo.lines().count() % flag.len();

    for (index, line) in logo.lines().enumerate() {
        let mut color_index = index / stripe_size;

        if color_index >= flag.len() {
            color_index = flag.len() - 1
        }

        writeln!(&mut flag_string, "{}", line.color(flag[color_index]))?;
    }

    Ok(flag_string)
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
