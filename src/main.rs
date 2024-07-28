use std::{borrow::Cow, collections::BTreeMap, fmt::Write, fs, io::BufRead, process::Command};

use anyhow::Result;
use clap::Parser;
use itertools::{EitherOrBoth, Itertools};
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use rustix::{
    process::geteuid,
    system::{sysinfo, uname},
    thread::ClockId,
    time::{clock_gettime, Timespec},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional distro name, defaults to autodetected distro
    #[arg(short, long)]
    distro: Option<Distro>,
    /// Sets colors for the flag (allowed values: "transgender"/"trans"/"t",
    /// "non-binary"/"nonbinary"/"enby"/"nb", "rainbow"/"r")
    #[arg(short, long)]
    colors: Option<String>,
}

mod passwd;

use crate::passwd::getpwuid;

const ARCH_LOGO: &str = r#"
                   -`
                  .o+`
                 `ooo/
                `+oooo:
               `+ooooolockedo:
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

const NIXOS_LOGO: &str = r###"
           ,##.      #####.    ###.
          '####\      #####.  #####
            ####\      `####.####'
       ,,,,,,####;,,,,,.`#######'
      ##################.`#####.      #`
     ####################.`#####.   ,###`
           ,MMMM;           '####. ,#####
         ,#####               ###',#####
 ,############'                #',##########.
<###########',                  #############>
 ''''''####',##               ,#####'''''''''
      ####',####.            ,#####
    ,####'  #####.          'WWWW'
     ###'    '####.`####################'
      `'      #####.`##################'
            ,########`''''''#####''''''
           ,####'`####.      #####.
          ,####'  `####.      #####
           "##'    `####'      `##'
"###;

const LINUX_LOGO: &str = r#"
              a8888b.
             d888888b.
             8P"YP"Y88
             8|o||o|88
             8'    .88
             8`._.' Y8.
            d/      `8b.
          .dP   .     Y8b.
         d8:'   "   `::88b.
        d8"           `Y88b
       :8P     '       :888
        8a.    :      _a88P
      ._/"Yaa_ :    .| 88P|
      \    YP"      `| 8P  `.
      /     \._____.d|    .'
      `--..__)888888P`._.'"#;

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
    DynColors::Rgb(230, 0, 0),
    DynColors::Rgb(255, 142, 0),
    DynColors::Rgb(255, 239, 0),
    DynColors::Rgb(0, 130, 27),
    DynColors::Rgb(0, 75, 255),
    DynColors::Rgb(120, 0, 137),
];

const DISTANCE: usize = 3;

fn main() -> Result<()> {
    let _sysinfo = sysinfo();
    // dbg!(sysinfo);
    // dbg!(&os_release);

    let args = Args::parse();

    let flag = get_logo(args.distro.unwrap_or_else(detect_distro));
    let colors = get_colors(args.colors.unwrap_or("".into()));
    let info = get_info_string()?;

    let stripe_size = ARCH_LOGO.lines().count() / colors.len();

    let longest_line = flag.lines().map(|l| l.len()).max().unwrap();

    let zipped = flag.lines().enumerate().zip_longest(info.lines());

    for line in zipped {
        match line {
            EitherOrBoth::Both((index, flag), info) => {
                let spacing = " ".repeat(longest_line - flag.len() + DISTANCE);

                let mut color_index = index / stripe_size;

                if color_index >= colors.len() {
                    color_index = colors.len() - 1
                }

                println!(
                    "{flag}{spacing}{info}",
                    flag = flag.color(colors[color_index])
                )
            }
            EitherOrBoth::Left((index, flag)) => {
                let mut color_index = index / stripe_size;

                if color_index >= colors.len() {
                    color_index = colors.len() - 1
                }

                println!("{flag}", flag = flag.color(colors[color_index]))
            }
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
    let distro_name = os_release.get("NAME").unwrap();

    let packages = match distro_name.as_ref() {
        "Arch Linux" => Some(format!("{} (pacman)", Command::new("pacman")
            .arg("-Q")
            .output()?
            .stdout
            .lines()
            .count())),
        "NixOS" => Some(format!("{} (nix)", Command::new("sh")
            .args(["-c", "nix-store --query --requisites /run/current-system | cut -d- -f2- | grep -o '.\\+[0-9]\\+\\.[0-9a-z.]\\+' | sort | uniq"])
            .output()?
            .stdout
            .lines()
            .count())),
        _ => None,
    };

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
    if let Some(packages) = packages {
        writeln!(
            &mut info_string,
            "{}: {}",
            "Packages".color(AnsiColors::Cyan).bold(),
            packages
        )?;
    }

    Ok(info_string)
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
    let mut entries = BTreeMap::new();

    if let Ok(os_release_file) = fs::read_to_string("/etc/os-release") {
        for line in os_release_file.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some((name, content)) = line.split_once('=') {
                let name = name.trim();
                let content = content.trim_matches('"');

                entries.insert(name.to_string(), content.to_string());
            }
        }
    }

    entries
}

#[derive(Debug, Clone)]
enum Distro {
    Arch,
    Nix,
    Unknown,
}

impl From<&str> for Distro {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "arch" | "arch linux" => Self::Arch,
            "nix" | "nixos" => Self::Nix,
            _ => Self::Unknown,
        }
    }
}

fn detect_distro() -> Distro {
    parse_os_release()
        .get("NAME")
        .map(|s| s.as_str())
        .unwrap_or("")
        .into()
}

fn get_logo(distro: Distro) -> &'static str {
    match distro {
        Distro::Arch => ARCH_LOGO,
        Distro::Nix => NIXOS_LOGO,
        _ => LINUX_LOGO,
    }
}

fn get_colors(colors: impl AsRef<str>) -> &'static [DynColors] {
    match colors.as_ref() {
        "transgender" | "trans" | "t" => &TRANSGENDER_FLAG,
        "non-binary" | "nonbinary" | "enby" | "nb" => &NONBINARY_FLAG,
        "rainbow" | "r" => &RAINBOW_FLAG,
        _ => &TRANSGENDER_FLAG,
    }
}
