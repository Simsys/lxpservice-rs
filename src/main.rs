mod clidef;
mod logger;
mod lxpapi;
mod lxpcommands;
mod lxpconfig;
mod lxptypes;

const APP_NAME: &str = "lxpservice";

#[tokio::main]
async fn main() {
    let matches = clidef::cli_definition(); // Defenition of the command line interface

    let mut lxp_cmds = lxpcommands::LxpCommands::new(APP_NAME, matches.occurrences_of("verbose"));

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
            lxp_cmds.invoide_list().await;
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
        lxp_cmds
            .job_set_fil_or_dir(
                &matches.value_of("file_or_dir").unwrap().to_string(),
                matches.is_present("black_and_white"),
                matches.is_present("international"),
                matches.is_present("duplex"),
            )
            .await;
    }
}
