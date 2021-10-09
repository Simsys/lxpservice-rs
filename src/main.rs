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
    let matches = clidef::cli_definition(APP_NAME, VERSION); // Defenition of the command line interface
    let verbose_level = matches.occurrences_of("verbose");
    logger::init(APP_NAME, VERSION, verbose_level);
    let mut lxp_cmds = lxpcommands::LxpCommands::new(APP_NAME);

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
        let file_or_dir_name = &matches.value_of("file_or_dir").unwrap().to_string();
        lxp_cmds
            .job_set_file_or_dir(&file_or_dir_name, color, mode, ship)
            .await;
    }

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
}
