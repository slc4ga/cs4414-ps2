//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::io::signal::{Listener, Interrupt};
use std::{io, run, os, str, vec, clone};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use std::io::fs::File;
use extra::getopts;

struct Shell {
	cmd_prompt: ~str,
	history: ~[~str],
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
            history: ~[],
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        
        loop {
            let cwd = std::os::getcwd();
            let cwd2 = format!("{}", cwd.display());
            let mut cwdS : ~[&str] = cwd2.split('/').collect();
            let last = cwdS.pop();
            if(last.len() > 0) {
                print!("{} : {}", last, self.cmd_prompt);
            } else {
                print!("Home : {}", self.cmd_prompt);
            }
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            if (cmd_line == ~"") {continue;}
            self.history.push(cmd_line.clone());
            let decomposed = self.decompose_Cmdline(cmd_line.clone());
            let mut error : bool = false;
            for j in range(0, decomposed.len()) {
            	if(Shell::checkForError(Some(~decomposed.clone()[j]))) {error = true;}
            }
            if error {continue;}
            	for j in range(0, decomposed.len()) {
            		let check = self.runDecomposed(Some(~decomposed.clone()[j]), ~[]);
            		if check==~"exit" { return; }
            }
        }
    }

	fn checkForError(cmd : Option<~DecomposedCmd>) -> bool {
		match cmd {
			Some(comm) 	=> {
				if comm.error {return true;}
				else {return Shell::checkForError(comm.pipeToNext);}
			}
			None		=> {
				return false;
			}
		}
	}
	
	fn runDecomposed (&mut self, cmd : Option<~DecomposedCmd>, input : ~[u8]) -> ~str {
		match cmd {
			Some(useCmd) => {
			if useCmd.background {
				let mut shellCopy = ~(Shell::new("gash > "));
				shellCopy.history = self.history.clone();
				let (portSelf, chanSelf): (Port<~Shell>, Chan<~Shell>) = Chan::new();
		
				chanSelf.send(shellCopy);
				
				do spawn { 
					let mut output : ~[u8] = ~[];
					let mut shellCopy : ~Shell = portSelf.recv();
					match useCmd.program {
						~""		=>  { }
						~"exit"		=>  { }
						~"cd"		=>  {
							if(useCmd.args.len() == 1) {
								os::change_dir(&Path::new(useCmd.args[0]));
							}
							else {
								println("Usage: cd <directory>");
							}
						}
						~"history"	=>  {
							match useCmd.outputFile.clone() {
								Some(fileName)	=> {
									let mut newStdOut = File::create(&Path::new(fileName));
									for z in range (0, shellCopy.history.len()) {
										newStdOut.write_str(shellCopy.history[z]);
										newStdOut.write_char('\n');
									}
								}
								_		=> {
									match useCmd.pipeToNext.clone() {
										Some(pipTo)	=> {
											for z in range (0, shellCopy.history.len()) {
												let temp = (shellCopy.history[z] + "\n");
												let tempBytes = temp.as_bytes();
												output = vec::append(output, tempBytes);
											}
										}
										_		=> { shellCopy.run_history(useCmd.cmd_line); }
									}
								}
							}
						}
						_		=>  { output = shellCopy.runDecomposedUnit(*useCmd, input); }
						
                			}
				
					shellCopy.runDecomposed(useCmd.pipeToNext.clone(), match useCmd.pipeToNext {Some(x) => {output} _ => {~[]} });
				}
			}
			else {
				let mut output : ~[u8] = ~[];
				match  useCmd.program {
					~""		=>  { }
					~"exit"		=>  { return ~"exit"; }
					~"cd"		=>  {
						if(useCmd.args.len() == 1) {
							os::change_dir(&Path::new(useCmd.args.clone()[0]));
						}
						else {
							println("Usage: cd <directory>");
						}
					}
					~"history"	=>  { 
						match useCmd.outputFile.clone() {
							Some(fileName)	=> {
								let mut newStdOut = File::create(&Path::new(fileName));
								for z in range (0, self.history.len()) {
									newStdOut.write_str(self.history[z]);
									newStdOut.write_char('\n');
								}
							}
							_		=> {
								match useCmd.pipeToNext.clone() {
									Some(pipTo)	=> {
										for z in range (0, self.history.len()) {
											let temp = (self.history[z] + "\n");
											let tempBytes = temp.as_bytes();
											output = vec::append(output, tempBytes);
										}
									}
									_		=> { self.run_history(useCmd.cmd_line); }
								}
							}
						}
					}
					_		=>  { output = self.runDecomposedUnit(*useCmd.clone(), input); }
				}
				self.runDecomposed(useCmd.pipeToNext.clone(), match useCmd.pipeToNext {Some(x) => {output} _ => {~[]} });
			}
			}
			_	=> {}
		}
		~""
	}

	fn runDecomposedUnit (&mut self, cmd : DecomposedCmd, input : ~[u8]) -> ~[u8] {
		match (cmd.inputFile.clone(), cmd.outputFile.clone()) {
			(Some(input), Some(output))	=> {
				let path = &Path::new(input.clone());
				let newStdOut = File::create(&Path::new(output));
				if (path.exists()) {
					let inputFile = File::open(path);
					match inputFile {
						Some(mut x) => {
							let inputBytes = x.read_to_end();
							let process = run::Process::new(cmd.program, cmd.args, run::ProcessOptions::new());
							match(process) {
								Some(mut toWrite) => {
									toWrite.input().write(inputBytes);
									let output = toWrite.finish_with_output();
									newStdOut.unwrap().write(output.output);
									return output.output;
								}
								None => {}
							}
						}
						None => { println!("gash: {:s}: No such file or directory", input);}
					}
				}
				else {
					println!("gash: {:s}: No such file or directory", input);
				}
			}
			(Some(input), _)		=> {
				let path = &Path::new(input.clone());
				if (path.exists()) {
					let inputFile = File::open(path);
					match inputFile {
						Some(mut x) => {
							let inputBytes = x.read_to_end();
							let process = run::Process::new(cmd.program, cmd.args, run::ProcessOptions::new());
							match(process) {
								Some(mut toWrite) => {
									toWrite.input().write(inputBytes);
									let output = toWrite.finish_with_output();
									match cmd.pipeToNext { None => {print!("{:s}", str::from_utf8(output.output));} _ => {}}
									return output.output;
								}
								None => {}
							}
						}
						None => { println!("gash: {:s}: No such file or directory", input);}
					}
				}
				else {
					println!("gash: {:s}: No such file or directory", input);
				}
			}
			(_, Some(output))		=> {
				let newStdOut = File::create(&Path::new(output));
				match newStdOut {
					Some(mut x) => {
						let mut options = run::ProcessOptions::new();
						if(input == ~[]) { options.in_fd = Some(0); }
						let process = run::Process::new(cmd.program, cmd.args, options);
						match process {
							Some(mut pros)	=> {
								if(input != ~[]) { pros.input().write(input); }
								let output = pros.finish_with_output();
								x.write(output.output);
								return output.output;
							}
							_		=> {}
						}
					}
					None => { println("shouldnt be here");}
				}
			}
			(_, _)				=> {
				let mut processOptions = run::ProcessOptions::new();
				match cmd.pipeToNext { None => { processOptions.out_fd = Some(1); } _ => {}}
				if(input == ~[]) { processOptions.in_fd = Some(0); }
				let process = run::Process::new(cmd.program, cmd.args, processOptions);
				match process {
					Some(mut pros)	=> {
						if(input != ~[]) { pros.input().write(input); }
						let output = pros.finish_with_output();
						return output.output;
					}
					_		=> {}
				}
			}
		}
		~[]
	}

	fn decompose_Cmdline (&mut self, mut cmd_line: ~str) -> ~[DecomposedCmd]{
		let mut decomposed = ~[];
		let mut decomposedCmd = DecomposedCmd {
			cmd_line: ~"",
			program: ~"",
			args: ~[],
			background: false,
			inputFile: None,
			outputFile: None,
			pipeToNext: None,
			error: false,
		};
		
		let background = cmd_line.find_str("&");
		match background {
			Some(index)	=> {
				if(index==0) {
					println("Syntax error near '&'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd1 = cmd_line.slice(0, index).trim().to_owned();
				let cmd2 = if(index < cmd_line.len()) {cmd_line.slice_from(index+1).trim().to_owned()} else {~""};
				decomposed = self.decompose_Cmdline(cmd1);
				decomposed[0].background = true;
				if(cmd2!=~"") { 
					decomposed = vec::append(decomposed, self.decompose_Cmdline(cmd2));
				}
				return decomposed;
			}
			_		=> {
			}
		}

		let pipe = cmd_line.find_str("|");
		match pipe {
			Some(index)	=> {
				if(index==0 || index == cmd_line.len()-1) {
					println("Syntax error near '|'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let cmd1 = cmd_line.slice(0, index).trim().to_owned();
				let cmd2 = cmd_line.slice_from(index+1).trim().to_owned();
				decomposed = self.decompose_Cmdline(cmd1);
				decomposed[decomposed.len() - 1].pipeToNext = Some(~self.decompose_Cmdline(cmd2)[0]);
				return decomposed;
			}
			_		=> {
			}
		}

		if !self.cmd_exists(cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned()) {
			let first = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
			if first != ~"exit" && first != ~"cd" && first != ~"history" {
				println!("{:s}: command not found", first);
				decomposedCmd.error = true;
				decomposed.push(decomposedCmd);
				return decomposed;
			}
		}

		let writeRedirect = cmd_line.find_str(">");
		let readRedirect = cmd_line.find_str("<");
		match (readRedirect, writeRedirect) {
			(Some(read), Some(write))	=> {
				if(write==0 || write == cmd_line.len()-1) {
					println("Syntax error near '>'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				if(read==0 || read == cmd_line.len()-1) {
					println("Syntax error near '<'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				if(read>write) {
					let mut cmd = cmd_line.slice(0, write).trim().to_owned();
					cmd = Shell::handleHistory(cmd.clone(), self.history.clone());
					if cmd == ~"error!" {
						decomposedCmd.error = true;
						decomposed.push(decomposedCmd);
						return decomposed;
					}

					let output = cmd_line.slice(write+1, read).trim().to_owned();
					let input = cmd_line.slice_from(read+1).to_owned();
					decomposedCmd.outputFile = Some(output);
					decomposedCmd.inputFile = Some(input);
					decomposedCmd.cmd_line = cmd.clone();
					decomposedCmd.program = cmd.splitn(' ', 1).nth(0).expect("no program").to_owned();
					let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
					args.remove(0);
				}
				else {
					let mut cmd = cmd_line.slice(0, read).trim().to_owned();
					cmd = Shell::handleHistory(cmd.clone(), self.history.clone());
					if cmd == ~"error!" {
						decomposedCmd.error = true;
						decomposed.push(decomposedCmd);
						return decomposed;
					}

					let input = cmd_line.slice(read+1, write).trim().to_owned();
					let output = cmd_line.slice_from(write+1).trim().to_owned();
					decomposedCmd.inputFile = Some(input);
					decomposedCmd.outputFile = Some(output);
					decomposedCmd.cmd_line = cmd.clone();
					decomposedCmd.program = cmd.splitn(' ', 1).nth(0).expect("no program").to_owned();
					let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
					args.remove(0);
					decomposedCmd.args = args;
				}
			}
			(Some(read), _)			=> {
				if(read==0 || read == cmd_line.len()-1) {
					println("Syntax error near '<'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let mut cmd = cmd_line.slice(0, read).trim().to_owned();
				cmd = Shell::handleHistory(cmd.clone(), self.history.clone());
				if cmd == ~"error!" {
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}

				let input = cmd_line.slice_from(read+1).trim().to_owned();
				decomposedCmd.inputFile = Some(input);
				decomposedCmd.cmd_line = cmd.clone();
				decomposedCmd.program = cmd.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
			(_, Some(write))		=> {
				if(write==0 || write == cmd_line.len()-1) {
					println("Syntax error near '>'");
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}
				let mut cmd = cmd_line.slice(0, write).trim().to_owned();
				cmd = Shell::handleHistory(cmd.clone(), self.history.clone());
				if cmd == ~"error!" {
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}

				let output = cmd_line.slice_from(write+1).trim().to_owned();
				decomposedCmd.outputFile = Some(output);
				decomposedCmd.cmd_line = cmd.clone();
				decomposedCmd.program = cmd.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
			(_, _)				=> {
				cmd_line = Shell::handleHistory(cmd_line.clone(), self.history.clone());
				if cmd_line == ~"error!" {
					decomposedCmd.error = true;
					decomposed.push(decomposedCmd);
					return decomposed;
				}

				decomposedCmd.cmd_line = cmd_line.to_owned();
				decomposedCmd.program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();
				let mut args : ~[~str] = cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
				args.remove(0);
				decomposedCmd.args = args;
			}
		}
		
		decomposed.push(decomposedCmd);
		return decomposed;
	}

	fn handleHistory(mut cmd_line : ~str, history : ~[~str]) -> ~str {
		let mut errorInHistory = false;
		let program = cmd_line.splitn(' ', 1).nth(0).expect("no program").to_owned();

		if (program == ~"history") {
			let split : ~[&str] = cmd_line.split(' ').collect();
			if split.len()>2 {
				println("gash: history: too many arguments");
				errorInHistory = true;
			}
			else if split.len()==2 {
				let num = from_str::<uint>(split[1]);
				match num {
					Some(index)	=> {
						if (index > history.len()) {
							println("You haven't entered that many commands yet - try a smaller number");
							errorInHistory = true;
						} else {
							cmd_line = history[history.len() - index - 1];
						}
					}
					_		=> {
						println!("gash: history: {:s}: positive numeric argument required", split[1]);
						errorInHistory = true;
					}
				}
			}
		}
		if errorInHistory {cmd_line = ~"error!"}
 
		return cmd_line;
	}
    
    fn run_cmdline(&mut self, cmd_line: &str) -> ~[u8]{
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            return self.run_cmd(program, argv);
        }
	return ~[];
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) -> ~[u8] {
        if self.cmd_exists(program) {
		let mut output : ~[u8] = ~[];
		{
			let out = run::process_output(program, argv);
			match out {
				Some(procOut)	=> {
					output = procOut.output;
				}
				None		=> {}
			}
		}
		return output;
        } else {
		println!("{:s}: command not found", program);
		return ~[];
        }
    }

    fn run_history(&mut self, program: &str) -> Option<~[DecomposedCmd]> {
        let histArgs : ~[&str] = program.split(' ').collect();
        if(histArgs.len() == 2) {
            if(from_str::<int>(histArgs[1]).unwrap() >= 0 && from_str::<uint>(histArgs[1]).unwrap() < self.history.len()) {
                    let num = self.history.len() - from_str::<uint>(histArgs[1]).unwrap() - 1;
                    println!("running: {:s}", self.history[num]);
                    let cmd = self.history[num].to_owned();
                    self.history.push(cmd.clone());
                    let decomposed = self.decompose_Cmdline(cmd);
                    return Some(decomposed);
            } else if(from_str::<int>(histArgs[1]).unwrap() < 0) {
                println("You can't run the command with a negative number!");
            } else {
                println("You haven't entered that many commands yet - try a smaller number");
            }
        } else {
            for c in range(0, self.history.len()) {
                println!("{:s}", self.history[c]);
            }
        }
	None
    }

    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    
    let opts = ~[
        getopts::optopt("c")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    let mut listener = Listener::new();
    listener.register(Interrupt);

    /*let (portSelf, chanSelf): (Port<Listener>, Chan<Listener>) = Chan::new();    
    chanSelf.send(listener);

    do spawn{
        loop {
            let listener = portSelf.recv();
            match listener.port.recv() {
                Interrupt => { unsafe { posix88::signal::kill(std::libc::getpid() , std::libc::SIGINT); }
                            }
                _ => (),
            }
        }
    };*/
    
    match opt_cmd_line {
        Some(cmd_line) => {Shell::new("").run_cmdline(cmd_line);},
        None           => Shell::new("gash > ").run()
    }
}

struct DecomposedCmd {
	cmd_line: ~str,
	program: ~str,
	args: ~[~str],
	background: bool,
	inputFile: Option<~str>,
	outputFile: Option<~str>,
	pipeToNext: Option<~DecomposedCmd>,
	error: bool,
}

impl clone::Clone for DecomposedCmd {
	fn clone(&self) -> DecomposedCmd {
		DecomposedCmd {
			cmd_line: self.cmd_line.clone(),
			program: self.program.clone(),
			args: self.args.clone(),
			background : self.background,
			inputFile: self.inputFile.clone(),
			outputFile: self.outputFile.clone(),
			pipeToNext: self.pipeToNext.clone(),
			error: self.error,
		}
	}
}

impl DecomposedCmd {
	fn print(&self) {
		print!("cmd_line: {:s}\nprogram: {:s}\nargs:", self.cmd_line, self.program);
		if self.args.len() > 0 {
			for i in range(0, self.args.len()) {
				print!("\n\t{:s}", self.args[i]);
			}
		}
		println!("\nbackground: {:b}\ninputFile: {:s}\noutputFile: {:s}\nerror: {:b}", self.background, 
			match self.clone().inputFile { Some(name) => {name} _ => {~""} }, match self.clone().outputFile { Some(name) => {name} _ => {~""} }, self.error);
		match self.clone().pipeToNext {
			Some(next) => {
				println("Pipe to:");
				next.print();
			}
			_ => {println("Pipe to: None");}
		}
	}
}
