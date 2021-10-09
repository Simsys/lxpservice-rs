mod clidef;
mod logger;
mod lxpapi;
mod lxpcommands;
mod lxpconfig;
mod lxptypes;

const APP_NAME: &str = "lxp";
const VERSION: &str = "0.1.1";

#[tokio::main]
async fn main() {
    // Defenition of the command line interface
    let matches = clidef::cli_definition(APP_NAME, VERSION); 

    let verbose_level = matches.occurrences_of("verbose");

    let (log_dir, config_dir) = match matches.subcommand_matches("daemonize") {
        Some(matches) => {
            let log_dir = std::fs::canonicalize(matches.value_of("directory")
                .unwrap()) // CLAP ensures that
                .expect("Couldn't determine log_dir");
            let config_dir = std::path::PathBuf::from("/etc").join(APP_NAME);
            (log_dir, config_dir)
        },
        None => {
            let log_dir = std::env::current_dir()
                .expect("Couldn't determine log_dir");
            let config_dir = dirs::config_dir()
                .expect("Couldn't determine config_dir")
                .join(APP_NAME);
                (log_dir, config_dir)
            },
    };
    println!("log_dir {:?}", log_dir);
    println!("config_dir {:?}", config_dir);

    logger::init(APP_NAME, VERSION, verbose_level);
    let mut lxp_cmds = lxpcommands::LxpCommands::new(APP_NAME);

    // handle subcommand daemonize
    if let Some(matches) = matches.subcommand_matches("daemonize") {
        let color = match matches.is_present("black_and_white") {
            true => lxptypes::ColorPrint::BlackAndWhite,
            false => lxptypes::ColorPrint::Color, 
        };
        let mode = match matches.is_present("international") {
            true => lxptypes::Mode::Duplex,
            false => lxptypes::Mode::Simplex,
        };
        let ship = match matches.is_present("duplex") {
            true => lxptypes::Ship::International,
            false => lxptypes::Ship::National,
        };
        let dir_name = &matches.value_of("directory").unwrap().to_string();
        lxp_cmds
            .watch_dir(&dir_name, color, mode, ship)
            .await;
    }

    // handle subcommand profile
    if let Some(matches) = matches.subcommand_matches("profile") {
        if matches.is_present("new") {
            lxp_cmds.profile_new(
                matches.value_of("profile").unwrap(), // unwrap is ok, arg reqired
                matches.value_of("user").unwrap(),    // ...
                matches.value_of("url").unwrap(),
                matches.value_of("api_key").unwrap(),
            );
        }
        if matches.is_present("delete") {
            // unwrap is ok, arg reqired
            lxp_cmds.profile_delete(matches.value_of("profile").unwrap());
        }
        if matches.is_present("delete_all") {
            lxp_cmds.profile_delete_all();
        }
        if matches.is_present("switch") {
            // unwrap is ok, arg reqired
            lxp_cmds.profile_switch(matches.value_of("profile").unwrap());
        }
        if matches.is_present("overview") {
            lxp_cmds.profile_show();
        }
    }

    // handle subcommand invoice
    if let Some(matches) = matches.subcommand_matches("invoice") {
        if matches.is_present("list") {
            lxp_cmds.invoice_list().await;
        }
        if matches.is_present("current") {
            lxp_cmds.invoice_get_last().await;
        }
        if matches.is_present("id") {
            lxp_cmds
                .invoice_get_by_id(matches.value_of("id").unwrap())
                .await;
        };
    }

    // handle subcommand job
    if let Some(matches) = matches.subcommand_matches("job") {
        // show overview
        if matches.is_present("overview") {
            lxp_cmds.job_overview().await;
        }

        // delete job(s)
        if matches.is_present("delete") {
            if matches.is_present("all") {
                lxp_cmds.job_delete_all().await;
            } else {
                lxp_cmds
                    .job_delete_by_id(matches.value_of("id").unwrap())
                    .await;
            }
        }
    }

    // handle subcommand set
    if let Some(matches) = matches.subcommand_matches("set") {
        let color = match matches.is_present("black_and_white") {
            true => lxptypes::ColorPrint::BlackAndWhite,
            false => lxptypes::ColorPrint::Color, 
        };
        let mode = match matches.is_present("international") {
            true => lxptypes::Mode::Duplex,
            false => lxptypes::Mode::Simplex,
        };
        let ship = match matches.is_present("duplex") {
            true => lxptypes::Ship::International,
            false => lxptypes::Ship::National,
        };
        let file_or_dir_name = matches.value_of("file_or_dir").unwrap().to_string();
        lxp_cmds
            .job_set_file_or_dir(&file_or_dir_name, color, mode, ship)
            .await;
    }
}
