// Copyright (C) 2019-2021 Aleo Systems Inc.
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

//! Evaluates a formatted string in a compiled Leo program.

use crate::{errors::ConsoleError, program::ConstrainedProgram, GroupType};
use leo_asg::{CharValue, ConsoleArgs};
use snarkvm_fields::PrimeField;
use snarkvm_r1cs::ConstraintSystem;

impl<'a, F: PrimeField, G: GroupType<F>> ConstrainedProgram<'a, F, G> {
    pub fn format<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        args: &ConsoleArgs<'a>,
    ) -> Result<String, ConsoleError> {
        let mut out = Vec::new();
        let mut in_container = false;
        let mut substring = String::new();
        let mut arg_index = 0;
        let mut escape_right_bracket = false;
        for (index, character) in args.string.iter().enumerate() {
            match character {
                _ if escape_right_bracket => {
                    escape_right_bracket = false;
                    continue;
                }
                CharValue::Scalar(scalar) => match scalar {
                    '{' if !in_container => {
                        out.push(substring.clone());
                        substring.clear();
                        in_container = true;
                    }
                    '{' if in_container => {
                        substring.push('{');
                        in_container = false;
                    }
                    '}' if in_container => {
                        in_container = false;
                        let parameter = match args.parameters.get(arg_index) {
                            Some(index) => index,
                            None => return Err(ConsoleError::length(arg_index + 1, args.parameters.len(), &args.span)),
                        };
                        out.push(self.enforce_expression(cs, parameter.get())?.to_string());
                        arg_index += 1;
                    }
                    '}' if !in_container => {
                        if let Some(CharValue::Scalar(next)) = args.string.get(index + 1) {
                            if *next == '}' {
                                substring.push('}');
                                escape_right_bracket = true;
                            } else {
                                return Err(ConsoleError::expected_escaped_right_brace(&args.span));
                            }
                        }
                    }
                    _ if in_container => {
                        return Err(ConsoleError::expected_left_or_right_brace(&args.span));
                    }
                    _ => substring.push(*scalar),
                },
                CharValue::NonScalar(non_scalar) => {
                    substring.push_str(format!("\\u{{{:x}}}", non_scalar).as_str());
                    in_container = false;
                }
            }
        }
        out.push(substring);

        // Check that containers and parameters match
        if arg_index != args.parameters.len() {
            return Err(ConsoleError::length(arg_index, args.parameters.len(), &args.span));
        }

        Ok(out.join(""))
    }
}
