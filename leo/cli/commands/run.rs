// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

use snarkvm::cli::Run as AleoRun;

/// Build, Prove and Run Leo program with inputs
#[derive(Parser, Debug)]
pub struct Run {
    #[clap(name = "NAME", help = "The name of the program to run.", default_value = "main")]
    name: String,

    #[clap(name = "INPUTS", help = "The inputs to the program. If none are provided, the input file is used.")]
    inputs: Vec<String>,

    #[clap(flatten)]
    pub(crate) compiler_options: BuildOptions,
}

impl Command for Run {
    type Input = <Build as Command>::Output;
    type Output = ();

    fn log_span(&self) -> Span {
        tracing::span!(tracing::Level::INFO, "Leo")
    }

    fn prelude(&self, context: Context) -> Result<Self::Input> {
        (Build { options: self.compiler_options.clone() }).execute(context)
    }

    fn apply(self, context: Context, input: Self::Input) -> Result<Self::Output> {
        // If input values are provided, then run the program with those inputs.
        // Otherwise, use the input file.
        let mut inputs = match self.inputs.is_empty() {
            true => match input {
                (Some(input_ast), circuits) => input_ast.program_inputs(&self.name, circuits),
                _ => Vec::new(),
            },
            false => self.inputs,
        };

        // Compose the `aleo run` command.
        let mut arguments = vec![ALEO_CLI_COMMAND.to_string(), self.name];
        arguments.append(&mut inputs);

        // Open the Leo build/ directory
        let path = context.dir()?;
        let build_directory = BuildDirectory::open(&path)?;

        // Change the cwd to the Leo build/ directory to compile aleo files.
        std::env::set_current_dir(&build_directory)
            .map_err(|err| PackageError::failed_to_set_cwd(build_directory.display(), err))?;

        // Call the `aleo run` command from the Aleo SDK.
        if self.compiler_options.offline {
            arguments.push(String::from("--offline"));
        }

        // Unset the Leo panic hook
        let _ = std::panic::take_hook();

        println!();
        let command = AleoRun::try_parse_from(&arguments).map_err(CliError::failed_to_parse_aleo_run)?;
        let res = command.parse().map_err(CliError::failed_to_execute_aleo_run)?;

        // Log the output of the `aleo run` command.
        tracing::info!("{}", res);

        Ok(())
    }
}
