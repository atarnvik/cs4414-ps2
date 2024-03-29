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

use std::{io, run, os};
use std::io::buffered::BufferedReader;
use std::io::signal::{Listener, Interrupt};
use std::run::Process;
use std::io::stdin;
use std::str;
use std::io::File;
use std::io::Reader;
use std::io::Writer;
use extra::getopts;

struct Shell {
    cmd_prompt: ~str,
}

impl Shell {

    fn new(prompt_str: &str) -> Shell {
        Shell {
            cmd_prompt: prompt_str.to_owned(),
        }
    }
    
    fn run(&mut self) {
       
        spawn(proc() {
            let mut listener = Listener::new();
            listener.register(Interrupt);
            loop {
                match listener.port.recv() {
                    Interrupt => (),
                    _ => (),
                }
            }
        });

        let mut stdin = BufferedReader::new(stdin());

        let mut cmdHist = ~[];
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();

            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            //add command to command history
            cmdHist.push(cmd_line.clone());
            
            
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            
            match program {
                ""      =>  { continue; }
                "!!" => { self.last_command(cmdHist.clone()); }
                "exit"  =>  { return; }
                "cd"    =>  { self.run_cd(cmd_line); }
                "history" =>   { self.run_history(cmdHist.clone()); }
                _       =>  { self.run_cmdline(cmd_line); }
            }
            
        }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str) {

        //Get commands between redirectors
        let mut commands: ~[~str] = cmd_line.split(|c: char| c=='>' || c=='<' || c=='|').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        //Create an array that describes redirects
        let mut elements: ~[~str] = cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        let mut state: ~[int] = ~[];
        for i in range(0, elements.len()) { 
            if (elements[i] == ~"<") {
                state.push(0);
            } else if (elements[i] == ~">") {
                state.push(1);
            } else if (elements[i] == ~"|") {
                state.push(2);
            }
        }
        //checks to see if command is to be run in the foreground or background
        let mut background: int = 0;
        let elements_size = elements.len();
        if (elements.last() == &~"&") {
            background = 1;
            elements.remove(elements_size-1);

        }
        //calls process using the commands, the state, and the background flag
        process(commands, state, background);
        //old stuff
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            self.run_cmd(program, argv);
        }
    }

    //maybe return a pointer to a buffer?


    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if(argv.len() > 0){
            //redirect output to text file
            if(argv[argv.len()-1] == ~"&") {
                let tempProgram: ~str = program.clone().to_owned();
                let tempArgv: ~[~str] = argv.clone().to_owned();
                spawn(proc() { 
                    let ret = run::process_output("which", [tempProgram.to_owned()]);
                    let ifExists: bool = ret.expect("exit code error.").status.success();
                    if ifExists {
                        let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None, out_fd: None, err_fd: None};
                        let process  = run::Process::new(tempProgram,tempArgv,Process_Options);
                        match(process) {
                            Some(mut p)  => {
                                let r = p.finish_with_output();
                            },
                            None => ()

                        }

                    } else {
                        println!("{:s}: command not found", tempProgram);
                    }
                });
            } else {

            }
        }
        //if it only has one argument, just run the command
        else{
            if self.cmd_exists(program) {
                let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None, out_fd: None, err_fd: None};
                let process  = run::Process::new(program,argv,Process_Options);
                match(process) {
                    Some(mut p)  => {
                        let r = p.finish_with_output().output;

                    },
                    None => ()

                }

                spawn(proc() {
                    let mut listener = Listener::new();
                    listener.register(Interrupt);
                    loop {
                        match listener.port.recv() {
 //                           Interrupt => run::process_output(program, argv).finish(),
                            _ => (),
                        }
                    }
                });
                let output_options = run::process_status(program, argv);

                // loop {
                //     let output_bytes: &~[u8] = &output_options.unwrap().output;
                //     //let output_bytes_new = str::from_utf8(output_bytes.to_owned());
                //     //let s = str::from_utf8(output_bytes.clone().to_owned());
                //     println(output_bytes.to_str());
                // }
                
                //println!("Output from command that was run in the background:\n{:s}",s);
            } else {
                println!("{:s}: command not found", program);
            }
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }

    fn run_cd(&mut self, cmd_line: &str) {
        let arguments: ~[&str] = cmd_line.split(' ').collect(); 
        if(arguments.len() == 2) {
            let dir = arguments[1].clone();
            let path = Path::new(dir.clone());
            if(os::change_dir(&path)){
                let cwd = os::getcwd();
                println!("{}", cwd.display());
            }
            else{
                println("Directory does not exist.");
            }
        }
        else {
            println("Please input only one argument.");
        }
    }

    fn last_command(&mut self, cmdHist: ~[~str]) {
        if(cmdHist.len() > 1) {
            let cmd_line = cmdHist[cmdHist.len()-2];
            println(cmd_line);
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
            
            match program {
                ""      =>  { }
                "exit"  =>  { return; }
                "cd"    =>  { self.run_cd(cmd_line); }
                _       =>  { self.run_cmdline(cmd_line); }
            }
        }
        else {
            println("History is not large enough!");
        }
    }

    fn run_history(&mut self, cmd_hist: ~[~str]) {
        for i in range(0, cmd_hist.len()) { 
            println!("{}", cmd_hist[i]);
        }
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

fn redirect_output(cmd_line: &str, background: int, filename: &str) {
        if (background == 1) {

            let mut argv: ~[~str] =
                cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
            let argcount = argv.len();
            if argcount > 0 {
            let program: ~str = argv.remove(0);
            let tempProgram: ~str = program.clone().to_owned();
            let tempArgv: ~[~str] = argv.clone().to_owned();
            let tempFilename: ~str = filename.clone().to_owned();
            spawn(proc() { 
                let ret = run::process_output("which", [tempProgram.to_owned()]);
                let ifExists: bool = ret.expect("exit code error.").status.success();
                if ifExists {
                    let mut output_file = File::create(&Path::new(tempFilename));
                    let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None, out_fd: None, err_fd: None};
                    let process  = run::Process::new(tempProgram,tempArgv,Process_Options);
                    match(process) {
                        Some(mut p)  => {
                            let reader = p.output();
                            while(true) {
                                match reader.read_byte(){
                                    Some (byte) => output_file.write_u8(byte),
                                    None => break
                                }
                            }
                            //let r = p.finish_with_output().output;
                            //println!("output = {:s}", str::from_utf8(bytes)); 

                        },
                        None => ()
                    }
                } else {
                    println!("{:s}: command not found", tempProgram);
                }
            });
         } else {
            let mut argv: ~[~str] =
                cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
            let program: ~str = argv.remove(0);
            let rightArrowIndex = vector_contains(">",argv);
            let mut filename = argv[rightArrowIndex + 1].to_owned();

            let mut output_file = File::create(&Path::new(filename));
            argv.remove(rightArrowIndex);
            argv.remove(rightArrowIndex);
            let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None, out_fd: None, err_fd: None};
            let process  = run::Process::new(program,argv,Process_Options);
            match(process) {
                Some(mut p)  => {
                    let reader = p.output();
                    let bytes = reader.read_to_end();
                    //let r = p.finish_with_output().output;

                    output_file.write(bytes);

                },
                None => ()

            }
        }
    }
}

    fn process(cmds: &[~str], state: &[int], background: int) {
        let commands = cmds.clone();
        let mut buffer: ~[u8] = ~[];

        //for loop for iterating through commands
        //if (state.len() == 0) {
        //    Shell::new("").run_cmdline(commands[0]);
        //}
        for i in range (0,state.len()) {
            if (state[i] == 0) {
                let input_filename = commands[i+1].trim().clone();
                let input_file_path = Path::new(input_filename.clone());
                let input_file = File::open(&input_file_path);
                match input_file {
                    Some(mut file) => {
                         buffer = file.read_to_end();
                         let temp_buffer: ~[u8] = redirect_input(commands[i],buffer);
                         buffer = temp_buffer.clone();
                    } ,
                    _ => {
                        fail!("Error opening input file!");
                        return;
                    } 
                }
            } else if state[i] == 2 {
                let temp_buffer: ~[u8] = redirect_input(commands[i+1],buffer);
                buffer = temp_buffer.clone();
            } else if (background == 1 && state[i] == 1) {
                //run this last process in the background
                redirect_output(commands[i],1,commands[i+1]);
                break;
            } else if state[i] == 1 {
                write_to_file(commands[i+1],buffer.clone());
            } else {
                Shell::new("").run_cmdline(commands[i]);
            }
        }

        //print out final output
        println!("{:s}",str::from_utf8(buffer));

    }

    fn run_in_background(program: ~str, argv: ~[~str], filename: ~str) {
        let tempProgram: ~str = program.clone().to_owned();
        let tempArgv: ~[~str] = argv.clone().to_owned();
        let tempFilename: ~str = filename.clone().to_owned();
        spawn(proc() { 
            let ret = run::process_output("which", [tempProgram.to_owned()]);
            let ifExists: bool = ret.expect("exit code error.").status.success();
            if ifExists {
                let mut output_file = File::create(&Path::new(tempFilename));
                let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None, out_fd: None, err_fd: None};
                let process  = run::Process::new(tempProgram,tempArgv,Process_Options);
                match(process) {
                    Some(mut p)  => {
                        let reader = p.output();
                        while(true) {
                            match reader.read_byte(){
                                Some (byte) => output_file.write_u8(byte),
                                None => break
                            }
                        }
                        //let r = p.finish_with_output().output;
                        //println!("output = {:s}", str::from_utf8(bytes)); 

                    },
                    None => ()
                }
            } else {
                println!("{:s}: command not found", tempProgram);
            }
        });
    }

    fn redirect_input(cmd_line: &str, input: ~[u8]) -> ~[u8]{
        //Get command line
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        let program: ~str = argv.remove(0);
        let Process_Options = run::ProcessOptions {env: None, dir: None, in_fd: None , out_fd: None, err_fd: None};
        let process  = run::Process::new(program,argv,Process_Options);
        let mut bytes: ~[u8] = ~[];
        match(process) {
            Some(mut p)  => {        
                for i in range (0, input.len()) {      
                    p.input().write_u8(input[i]);
                }
                //let reader = p.output();
                //bytes = reader.read_to_end();

                bytes = p.finish_with_output().output;
            },
            None => ()
        }
        
        return bytes;
    }

    fn write_to_file(filename: &str, output: ~[u8]) {
        let mut output_file = File::create(&Path::new(filename));
        output_file.write(output);
    }
//takes a vector and a string to look for. Checks to see any of the elements of the vector
// match exactly. returns the integer of the first element that matches. Otherwise, returns -1.
fn vector_contains(keyword: &str, argv: &[~str]) -> uint {
    for i in range (0,argv.len()) {
        if (argv[i] == keyword.to_owned()) {
            return i;
        }
    }
    return -1;
}


fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
