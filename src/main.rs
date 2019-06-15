use chrono;
use clap;
use clap::{App, Arg};

use regex::Regex;
use rpm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
     let matches = App::new("rpm-builder")
          .version("0.1.0")
          .author("Rene R. <richterrettich@gmail.com>")
          .about("Build rpms with ease")
          .arg(Arg::with_name("version")
               .long("version")
               .value_name("VERSION")
               .help("Specify a version")
               .default_value("1.0.0")
               .takes_value(true))
          .arg(Arg::with_name("license")
               .long("license")
               .value_name("LICENSE")
               .help("Specify a license")
               .default_value("MIT")
               .takes_value(true))
          .arg(Arg::with_name("arch")
               .long("arch")
               .value_name("ARCH")
               .help("Specify the target architecture")
               .default_value("x86_64")
               .takes_value(true))
          .arg(Arg::with_name("release")
               .long("release")
               .value_name("RELEASE")
               .help("Specify release number of the package")
               .default_value("1")
               .takes_value(true))
          .arg(Arg::with_name("desc")
               .long("desc")
               .value_name("DESC")
               .help("Give a description of the package")
               .default_value("")
               .takes_value(true))
          .arg(Arg::with_name("exec-file")
               .long("exec-file")
               .value_name("EXEC_FILE")
               .help("add a executable-file to the rpm")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("doc-file")
               .long("doc-file")
               .value_name("DOC_FILE")
               .help("add a documentation-file to the rpm")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("config-file")
               .long("config-file")
               .value_name("CONFIG_FILE")
               .help("add a config-file to the rpm")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
           .arg(Arg::with_name("changelog")
               .long("changelog")
               .value_name("CHANGELOG_ENTRY")
               .help("add a changelog entry to the rpm. The entry has the form <author>:<content>:<yyyy-mm-dd> (time is in utc)")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("requires")
               .long("requires")
               .value_name("REQUIRES")
               .help("indicates that the rpm requires another package. Use the format '<name> [>|>=|=|<=|< version]'")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("provides")
               .long("provides")
               .value_name("PROVIDES")
               .help("indicates that the rpm provides another package. Use the format '<name> [>|>=|=|<=|< version]'")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("obsoletes")
               .long("obsoletes")
               .value_name("OBSOLETES")
               .help("indicates that the rpm obsoletes another package. Use the format '<name> [>|>=|=|<=|< version]'")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("conflicts")
               .long("conflicts")
               .value_name("CONFLICTS")
               .help("indicates that the rpm conflicts with another package. Use the format '<name> [>|>=|=|<=|< version]'")
               .takes_value(true)
               .multiple(true)
               .number_of_values(1))
          .arg(Arg::with_name("name")
               .help("Specify the name of your package")
               .required(true))
          .get_matches();


     let name = matches.value_of("name").unwrap();
     let version = matches.value_of("version").unwrap();
     let license = matches.value_of("license").unwrap();
     let arch = matches.value_of("arch").unwrap();
     let description = matches.value_of("desc").unwrap();

     let mut builder = rpm::RPMBuilder::new(name, version, license, arch, description);
     let files = matches
          .values_of("exec-file")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for f in files {
          let parts: Vec<&str> = f.split(":").collect();
          if parts.len() != 2 {
               return Err(Box::new(AppError::new(format!(
                    "invalid file argument:{} it needs to be of the form <source-path>:<dest-path>",
                    &f
               ))));
          }
          builder =
               builder.with_file(parts[0], rpm::RPMFileOptions::new(parts[1]).mode(0o100755))?;
     }

     let files = matches
          .values_of("config-file")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for f in files {
          let parts: Vec<&str> = f.split(":").collect();
          if parts.len() != 2 {
               return Err(Box::new(AppError::new(format!(
                    "invalid file argument:{} it needs to be of the form <source-path>:<dest-path>",
                    &f
               ))));
          }
          builder = builder.with_file(parts[0], rpm::RPMFileOptions::new(parts[1]).is_config())?;
     }

     let files = matches
          .values_of("doc-file")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for f in files {
          let parts: Vec<&str> = f.split(":").collect();
          if parts.len() != 2 {
               return Err(Box::new(AppError::new(format!(
                    "invalid file argument:{} it needs to be of the form <source-path>:<dest-path>",
                    &f
               ))));
          }
          builder = builder.with_file(parts[0], rpm::RPMFileOptions::new(parts[1]).is_doc())?;
     }

     let raw_changelog = matches
          .values_of("changelog")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for raw_entry in raw_changelog {
          let parts: Vec<&str> = raw_entry.split(":").collect();
          if parts.len() != 3 {
               return Err(Box::new(AppError::new(format!(
                    "invalid file argument:{} it needs to be of the form <author>:<content>:<yyyy-mm-dd>",
                    &raw_entry
               ))));
          }
          let name = parts[0];
          let content = parts[1];
          let raw_time = parts[2];
          let parse_result = chrono::NaiveDate::parse_from_str(raw_time, "%Y-%m-%d");
          if parse_result.is_err() {
               return Err(Box::new(AppError::new(format!(
                    "error while parsing date time: {}",
                    parse_result.err().unwrap()
               ))));
          }
          let seconds = parse_result
               .unwrap()
               .and_time(chrono::NaiveTime::from_hms(0, 0, 0))
               .timestamp();
          builder = builder.add_changelog_entry(name, content, seconds as i32);
     }

     let re = Regex::new(r"^([a-zA-Z0-9\-]+)(\s*(>=|>|=|<=|<)(.+))?$").unwrap();

     let requires = matches
          .values_of("requires")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for req in requires {
          let dependency = parse_dependency(&re, req)?;
          builder = builder.requires(dependency);
     }


     let obsoletes = matches
          .values_of("obsoletes")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for item in obsoletes {
          let dependency = parse_dependency(&re, item)?;
          builder = builder.obsoletes(dependency);
     }

     let conflicts = matches
          .values_of("conflicts")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for item in conflicts {
          let dependency = parse_dependency(&re, item)?;
          builder = builder.conflicts(dependency);
     }

     let provides = matches
          .values_of("provides")
          .map(|v| v.collect())
          .unwrap_or(Vec::new());

     for item in provides {
          let dependency = parse_dependency(&re, item)?;
          builder = builder.provides(dependency);
     }

     let pkg = builder.build()?;
     let mut out_file = std::fs::File::create(format!("./{}.rpm", name))?;
     pkg.write(&mut out_file)?;
     Ok(())
}

fn parse_dependency(re: &Regex, line: &str) -> Result<rpm::Dependency, AppError> {
     let captures = re.captures(line).ok_or(AppError::new(format!(
          "invalid pattern in dependency block {}",
          line
     )))?;
     if captures.len() == 2 {
          Ok(rpm::Dependency::any(&captures[1]))
     } else {
          let dep = match &captures[3] {
               "=" => rpm::Dependency::eq(&captures[1], &captures[4]),
               "<" => rpm::Dependency::less(&captures[1], &captures[4]),
               "<=" => rpm::Dependency::less_eq(&captures[1], &captures[4]),
               ">=" => rpm::Dependency::greater_eq(&captures[1], &captures[4]),
               ">" => rpm::Dependency::greater(&captures[1], &captures[4]),
               _ => {
                    return Err(AppError::new(format!(
                         "regex is invalid here, got unknown match {}",
                         &captures[3]
                    )))
               }
          };
          Ok(dep)
     }
}


struct AppError {
     cause: String,
}

impl AppError {
     fn new<T: Into<String>>(cause: T) -> Self {
          return AppError {
               cause: cause.into(),
          };
     }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
          write!(f, "{}", self.cause)
     }
}

impl std::fmt::Debug for AppError {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
          write!(f, "{}", self.cause)
     }
}
