use std::{borrow::Cow, collections::BTreeMap, fmt::Write, fs, io::BufRead, process::Command};

use acumen::{getpwuid, Cpuinfo, Meminfo, OsRelease};
use anyhow::Result;
use clap::Parser;
use humansize::{format_size, BINARY};
use itertools::{EitherOrBoth, Itertools};
use owo_colors::{AnsiColors, DynColors, OwoColorize};
use rustix::{
    process::geteuid,
    system::uname,
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

const ARCH_LOGO: &str = include_str!("logos/arch.txt");
const NIXOS_LOGO: &str = include_str!("logos/nixos.txt");
const LINUX_LOGO: &str = include_str!("logos/linux.txt");

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
    let args = Args::parse();

    let os_release = OsRelease::new();
    let distro = args.distro.unwrap_or(detect_distro(&os_release));

    let flag = get_logo(&distro);
    let colors = get_colors(args.colors.unwrap_or("".into()));
    let info = get_info_string(&distro, &os_release)?;

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

fn get_info_string(distro: &Distro, os_release: &OsRelease) -> Result<String> {
    let mut info_string = String::new();
    let meminfo = Meminfo::new();

    let totalram = meminfo.mem_total().unwrap();
    let freeram = meminfo.get("MemFree").unwrap();

    let total = format_size(totalram, BINARY);
    let used = format_size(totalram - freeram, BINARY);

    let cpuinfo = Cpuinfo::new();

    let uname = uname();

    let packages = match distro {
        Distro::Arch => Some(format!("{} (pacman)", Command::new("pacman")
            .arg("-Q")
            .output()?
            .stdout
            .lines()
            .count())),
        Distro::Nix=> Some(format!(
            "{} (nix system), {} (nix user)",
            Command::new("sh")
                .args(["-c", "nix-store -qR /run/current-system | cut -d- -f2- | grep -o '.\\+[0-9]\\+\\.[0-9a-z.]\\+' | sort | uniq"])
                .output()?
                .stdout
                .lines()
                .count(),
            Command::new("sh")
                .args(["-c", "nix-store -qR $HOME/.nix-profile | cut -d- -f2- | grep -o '.\\+[0-9]\\+\\.[0-9a-z.]\\+' | sort | uniq"])
                .output()?
                .stdout
                .lines()
                .count(),
        )),
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
    writeln!(
        &mut info_string,
        "{}: {used} / {total}",
        "Memory".color(AnsiColors::Cyan).bold(),
    )?;

    writeln!(
        &mut info_string,
        "{}: {}",
        "CPU".color(AnsiColors::Cyan).bold(),
        cpuinfo.cpus()[0].model_name().unwrap()
    )?;

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

fn detect_distro(os_release: &OsRelease) -> Distro {
    os_release
        .id()
        .unwrap_or(os_release.name().unwrap_or(""))
        .into()
}

fn get_logo(distro: &Distro) -> &'static str {
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
